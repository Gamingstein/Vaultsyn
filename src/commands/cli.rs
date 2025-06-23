use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vaultsyn", version = "0.1", author = "Gamingstein")]
pub struct VaultsynCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new user identity
    CreateUser {
        username: String,
    },
    /// Export a public key
    ExportPublicKey {
        username: String,
    },
    /// Send Message
    SendMessage {
        sender: String,
        receiver_public_key: String,
        message: String,
    },
    /// Receive Message
    ReceiveMessage {
        receiver: String,
        sender_ed25519_pub: String,
        sender_x25519_pub: String,
        envelope_json: String,
    },
    Connect {
        url: String,
    },
    Chat {
        url: String,
        sender: String,
        receiver_x25519_pub: String,
    },
}
