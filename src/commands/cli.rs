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
    CreateUser { username: String },
    /// Export a public key
    ExportPublicKey { username: String },
}
