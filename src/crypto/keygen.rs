use base64::{prelude::BASE64_STANDARD as base64Standard, Engine};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

#[derive(Clone, Serialize, Deserialize)]
pub struct Identity {
    pub username: String,
    pub ed25519_public: String,
    pub ed25519_private: String,
    pub x25519_public: String,
    pub x25519_private: String,
}

impl Identity {
    pub fn public_info_json(&self) -> String {
        serde_json::json!({
            "username": self.username,
            "ed25519_public": self.ed25519_public,
            "x25519_public": self.x25519_public
        })
        .to_string()
    }
}

pub fn generate_identity(username: String) -> Identity {
    let mut csprng = OsRng;

    // ✅ Ed25519 Keypair (newer API)
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key = VerifyingKey::from(&signing_key);

    // ✅ X25519 Keypair
    let x_secret = StaticSecret::random_from_rng(&mut csprng);
    let x_public = X25519Public::from(&x_secret);

    Identity {
        username,
        ed25519_public: base64Standard.encode(verifying_key.as_bytes()),
        ed25519_private: base64Standard.encode(signing_key.to_bytes()),
        x25519_public: base64Standard.encode(x_public.as_bytes()),
        x25519_private: base64Standard.encode(x_secret.to_bytes()),
    }
}
