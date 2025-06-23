mod commands;
mod crypto;
mod io;

use clap::Parser;
use commands::cli::{Commands, VaultsynCli};
use commands::user;

#[tokio::main]
async fn main() {
    let args = VaultsynCli::parse();

    match args.command {
        Commands::CreateUser { username } => {
            user::create_user(&username);
        }
        Commands::ExportPublicKey { username } => {
            user::export_public_key(&username);
        }
    }
}
