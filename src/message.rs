use crate::crypto::keygen::Identity;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce}; // GCM = AES-GCM
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

// use crate::message::MessageEnvelope;
use base64::{prelude::BASE64_STANDARD as standard, Engine};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub from: String,
    pub to: String,
    pub ciphertext: String,
    pub nonce: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct VaultsynTransport {
    pub envelope: MessageEnvelope,
    pub sender_ed25519_pub: String,
    pub sender_x25519_pub: String,
}

pub fn encrypt_and_sign_message(
    sender: &Identity,
    receiver_public: &str,
    message: &str,
) -> MessageEnvelope {
    // Decode receiver’s x25519 public key
    let receiver_key_bytes = standard
        .decode(receiver_public)
        .expect("Invalid base64 public key");
    let receiver_public = X25519Public::from(<[u8; 32]>::try_from(receiver_key_bytes).unwrap());

    // Decode sender’s x25519 private key
    let sender_secret_bytes = standard.decode(&sender.x25519_private).unwrap();
    let sender_secret = StaticSecret::from(<[u8; 32]>::try_from(sender_secret_bytes).unwrap());

    // Shared key via ECDH
    let shared_secret = sender_secret.diffie_hellman(&receiver_public);
    let shared_bytes = shared_secret.as_bytes();

    // Derive AES key from shared secret
    let aes_key = Sha256::digest(shared_bytes);
    let key = Key::<Aes256Gcm>::from_slice(&aes_key[..32]);

    // Create cipher
    let cipher = Aes256Gcm::new(key);

    // Random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, message.as_bytes())
        .expect("encryption failed");

    // Create Ed25519 keypair
    let ed_bytes = standard.decode(&sender.ed25519_private).unwrap();
    let ed_keypair = SigningKey::from_bytes(&<[u8; 32]>::try_from(ed_bytes).unwrap());

    // Sign the ciphertext
    let signature = ed_keypair.sign(&ciphertext);

    // Return envelope
    MessageEnvelope {
        from: sender.username.clone(),
        to: String::from("receiver"),
        ciphertext: standard.encode(ciphertext),
        nonce: standard.encode(nonce_bytes),
        signature: standard.encode(signature.to_bytes()),
    }
}

pub fn decrypt_and_verify_message(
    envelope: &MessageEnvelope,
    receiver: &Identity,
    sender_ed25519_pub: &str,
    sender_x25519_pub: &str,
) -> Result<String, String> {
    // Decode keys
    let receiver_secret_bytes = standard.decode(&receiver.x25519_private).unwrap();
    let receiver_secret = StaticSecret::from(<[u8; 32]>::try_from(receiver_secret_bytes).unwrap());

    let sender_pub_bytes = standard.decode(sender_x25519_pub).unwrap();
    let sender_public = X25519Public::from(<[u8; 32]>::try_from(sender_pub_bytes).unwrap());

    // Perform X25519 ECDH
    let shared_secret = receiver_secret.diffie_hellman(&sender_public);
    let aes_key = Sha256::digest(shared_secret.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&aes_key);

    let cipher = Aes256Gcm::new(key);

    // Fix: avoid temporary value dropped while borrowed
    let nonce_bytes = standard.decode(&envelope.nonce).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt
    let decrypted = cipher
        .decrypt(
            nonce,
            standard.decode(&envelope.ciphertext).unwrap().as_ref(),
        )
        .map_err(|_| "❌ Decryption failed")?;

    // Verify signature
    let verifying_key_bytes = standard.decode(sender_ed25519_pub).unwrap();
    let verifying_key =
        VerifyingKey::from_bytes(&<[u8; 32]>::try_from(verifying_key_bytes).unwrap())
            .map_err(|_| "❌ Invalid public key")?;

    let signature_bytes = standard.decode(&envelope.signature).unwrap();
    let signature = Signature::from_bytes(&<[u8; 64]>::try_from(signature_bytes).unwrap());
    // .map_err(|_| "❌ Invalid signature format")?;

    verifying_key
        .verify(&standard.decode(&envelope.ciphertext).unwrap(), &signature)
        .map_err(|_| "❌ Signature verification failed")?;

    Ok(String::from_utf8(decrypted).unwrap())
}
