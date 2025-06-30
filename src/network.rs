use crate::crypto::keygen::Identity;
use crate::io::storage::load_identity;
use crate::message::{encrypt_and_sign_message, VaultsynTransport};

use crossterm::style::*;
use futures::{SinkExt, StreamExt};
use rustyline::Editor;
use tokio::task;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};

use chrono::Local;

pub async fn vaultsyn_secure_chat(uri: &str, sender_id: &str, receiver_pub_x25519: &str) {
    let url = Url::parse(uri).expect("Invalid WebSocket URL");
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("📡 Connected securely to Vaultsyn at {}", uri);
    let (mut write, mut read) = ws_stream.split();

    let identity: Identity = load_identity(sender_id).expect("Could not load sender identity");

    // Shared stdout for sync printing
    let stdout = Arc::new(Mutex::new(stdout()));

    // Read incoming messages
    let sender_id_reader = Arc::new(sender_id.to_string());
    let stdout_reader = stdout.clone();
    let reader_task = {
        let sender_id_reader = sender_id_reader.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<VaultsynTransport>(&text) {
                        Ok(envelope) => {
                            let identity =
                                load_identity(&sender_id_reader).expect("Missing identity");

                            if envelope.sender_ed25519_pub == identity.ed25519_public {
                                continue; // skip self-echo
                            }

                            match crate::message::decrypt_and_verify_message(
                                &envelope.envelope,
                                &identity,
                                &envelope.sender_ed25519_pub,
                                &envelope.sender_x25519_pub,
                            ) {
                                Ok(decrypted) => {
                                    let _now = Local::now().format("%H:%M:%S");
                                    let mut out = stdout_reader.lock().unwrap();
                                    writeln!(
                                        &mut *out,
                                        "\r{} {}: {}\n{} ❯ ",
                                        "📨".yellow(),
                                        envelope.envelope.from.green(),
                                        decrypted,
                                        sender_id_reader.as_str().blue()
                                    )
                                    .unwrap();
                                }
                                Err(e) => {
                                    let mut out = stdout_reader.lock().unwrap();
                                    writeln!(
                                        &mut *out,
                                        "\r{} {}\n{} ❯ ",
                                        "⚠️  Decryption failed:".red(),
                                        e,
                                        sender_id_reader.as_str().blue()
                                    )
                                    .unwrap();
                                }
                            }
                        }
                        Err(_) => {
                            let mut out = stdout_reader.lock().unwrap();
                            writeln!(
                                &mut *out,
                                "\r{} {}\n{} ❯ ",
                                "⚠️  Invalid message format.".red(),
                                text,
                                sender_id_reader.as_str().blue()
                            )
                            .unwrap();
                        }
                    }
                }
            }
        })
    };

    // Write outgoing messages
    let identity_writer = identity.clone();
    let receiver_pub_x25519_writer = receiver_pub_x25519.to_string();
    let sender_id_writer = sender_id.to_string();
    let stdout_writer = stdout.clone();

    let writer_task = task::spawn_blocking(move || {
        let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
        let prompt = format!("{} ❯ ", sender_id_writer.clone().blue());

        println!("💬 Encrypted chat ready. Type /exit to quit.");

        while let Ok(line) = rl.readline(&prompt) {
            if line.trim() == "/exit" {
                break;
            }

            if !line.trim().is_empty() {
                let _ = rl.add_history_entry(line.as_str());

                let envelope =
                    encrypt_and_sign_message(&identity_writer, &receiver_pub_x25519_writer, &line);

                let transport = VaultsynTransport {
                    envelope,
                    sender_ed25519_pub: identity_writer.ed25519_public.clone(),
                    sender_x25519_pub: identity_writer.x25519_public.clone(),
                };

                let json = serde_json::to_string(&transport).unwrap();
                if let Err(e) = futures::executor::block_on(write.send(Message::Text(json))) {
                    let mut out = stdout_writer.lock().unwrap();
                    writeln!(&mut *out, "{} {}", "⚠️  Failed to send message:".red(), e).unwrap();
                    break;
                }

                let _now = Local::now().format("%H:%M:%S");
                let mut out = stdout_writer.lock().unwrap();
                writeln!(
                    &mut *out,
                    "{} {}: {}",
                    "🕒".dim(),
                    sender_id_writer.clone().green(),
                    line.trim()
                )
                .unwrap();
            }
        }
    });

    let _ = tokio::join!(reader_task, writer_task);
    println!("{}", "🔌 Disconnected.".dark_grey());
}

pub async fn vaultsyn_ws_client(uri: &str, on_msg: impl Fn(String) + Send + Sync + 'static) {
    let url = Url::parse(uri).expect("Invalid WebSocket URL");
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("📡 Connected to Vaultsyn network at {}", uri);
    let (mut write, mut read) = ws_stream.split();

    // Spawn task to listen
    let on_msg = std::sync::Arc::new(on_msg);
    let _reader = {
        let on_msg = on_msg.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    on_msg(text);
                }
            }
        })
    };

    // REPL loop: send messages
    let stdin = std::io::stdin();
    let mut input = String::new();
    println!("💬 Type to send messages:");

    loop {
        input.clear();
        if stdin.read_line(&mut input).is_ok() {
            if input.trim() == "/exit" {
                break;
            }
            write
                .send(Message::Text(input.trim().to_string()))
                .await
                .unwrap();
        }
    }

    println!("🔌 Disconnected.");
}

// pub async fn vaultsyn_secure_chat(uri: &str, sender_id: &str, receiver_pub_x25519: &str) {
//     let url = Url::parse(uri).expect("Invalid WebSocket URL");
//     let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

//     println!("📡 Connected securely to Vaultsyn at {}", uri);
//     let (mut write, mut read) = ws_stream.split();

//     let identity: Identity = load_identity(sender_id).expect("Could not load sender identity");

//     // Read incoming messages
//     let sender_id_reader = sender_id.to_string();
//     let reader_task = tokio::spawn(async move {
//         while let Some(Ok(msg)) = read.next().await {
//             if let Message::Text(text) = msg {
//                 match serde_json::from_str::<VaultsynTransport>(&text) {
//                     Ok(envelope) => {
//                         let identity = load_identity(&sender_id_reader).expect("Missing identity");

//                         if envelope.sender_ed25519_pub == identity.ed25519_public {
//                             continue; // skip echo of self message
//                         }

//                         match crate::message::decrypt_and_verify_message(
//                             &envelope.envelope,
//                             &identity,
//                             &envelope.sender_ed25519_pub,
//                             &envelope.sender_x25519_pub,
//                         ) {
//                             Ok(decrypted) => {
//                                 println!(
//                                     "{} {}: {}",
//                                     "📨".yellow(),
//                                     envelope.envelope.from.green(),
//                                     decrypted
//                                 );
//                             }
//                             Err(e) => println!("{} {}", "⚠️  Decryption failed:".red(), e),
//                         }
//                     }
//                     Err(_) => {
//                         println!("{}", "⚠️  Invalid message format.".red());
//                     }
//                 }
//             }
//         }
//     });

//     // Write outgoing messages
//     let identity_writer = identity.clone();
//     let receiver_pub_x25519_writer = receiver_pub_x25519.to_string();
//     let sender_id_writer = sender_id.to_string();
//     let writer_task = task::spawn_blocking(move || {
//         let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
//         let prompt = format!("{} ❯ ", sender_id_writer.clone().blue());

//         println!("💬 Encrypted chat ready. Type /exit to quit.");

//         while let Ok(line) = rl.readline(&prompt) {
//             if line.trim() == "/exit" {
//                 break;
//             }

//             if !line.trim().is_empty() {
//                 let _ = rl.add_history_entry(line.as_str());

//                 let envelope =
//                     encrypt_and_sign_message(&identity_writer, &receiver_pub_x25519_writer, &line);

//                 let transport = VaultsynTransport {
//                     envelope,
//                     sender_ed25519_pub: identity_writer.ed25519_public.clone(),
//                     sender_x25519_pub: identity_writer.x25519_public.clone(),
//                 };

//                 let json = serde_json::to_string(&transport).unwrap();
//                 if let Err(e) = futures::executor::block_on(write.send(Message::Text(json))) {
//                     println!("{} {}", "⚠️  Failed to send message:".red(), e);
//                     break;
//                 }

//                 println!(
//                     "{} {}: {}",
//                     "🕒".dim(),
//                     sender_id_writer.clone().green(),
//                     line.trim()
//                 );
//             }
//         }
//     });

//     let _ = tokio::join!(reader_task, writer_task);
//     println!("{}", "🔌 Disconnected.".dark_grey());
// }
