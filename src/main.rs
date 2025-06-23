mod commands;
mod crypto;
mod io;
mod message;
mod network;

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
        Commands::SendMessage {
            sender,
            receiver_public_key,
            message,
        } => {
            user::send_message(&sender, &receiver_public_key, &message);
        }
        Commands::ReceiveMessage {
            receiver,
            sender_ed25519_pub,
            sender_x25519_pub,
            envelope_json,
        } => {
            user::receive_message(
                &receiver,
                &sender_ed25519_pub,
                &sender_x25519_pub,
                &envelope_json,
            );
        }
        Commands::Connect { url } => {
            network::vaultsyn_ws_client(&url, |msg| {
                println!("📨 Received: {}", msg);
            })
            .await;
        }
        Commands::Chat {
            url,
            sender,
            receiver_x25519_pub,
        } => {
            network::vaultsyn_secure_chat(&url, &sender, &receiver_x25519_pub).await;
        }
    }
}
