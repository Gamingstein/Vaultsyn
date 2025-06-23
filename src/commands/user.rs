use crate::crypto::keygen::{generate_identity, Identity};
use crate::io::storage::{load_identity, save_identity};
use crate::message::{decrypt_and_verify_message, encrypt_and_sign_message, MessageEnvelope};

pub fn create_user(username: &str) {
    let identity = generate_identity(username.to_string());
    save_identity(&identity).expect("Failed to save identity");
    println!("✅ Identity for '{}' created.", username);
}

pub fn export_public_key(username: &str) {
    let identity: Identity = crate::io::storage::load_identity(username).expect("User not found");
    println!(
        "🔑 Public key for {}:\n{}",
        username,
        identity.public_info_json()
    );
}

pub fn send_message(sender: &str, receiver_pub: &str, content: &str) {
    let identity = load_identity(sender).expect("Sender not found");
    let envelope = encrypt_and_sign_message(&identity, receiver_pub, content);
    println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}

pub fn receive_message(receiver_username: &str, sender_ed: &str, sender_x25519: &str, json: &str) {
    let receiver =
        crate::io::storage::load_identity(receiver_username).expect("Receiver not found");

    let envelope: MessageEnvelope = serde_json::from_str(json).expect("Invalid JSON message");

    match decrypt_and_verify_message(&envelope, &receiver, sender_ed, sender_x25519) {
        Ok(msg) => {
            println!("✅ Verified message:\n{}", msg);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
