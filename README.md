# Vaultsyn

**Vaultsyn** is a secure, end-to-end encrypted messaging and identity system written in Rust. It provides cryptographically secure user identities, encrypted messaging, and a simple WebSocket-based relay server for real-time communication.

---

## Features

- **User Identity Management**: Generate and store Ed25519/X25519 keypairs for each user.
- **Public Key Export**: Share your public keys for others to send you encrypted messages.
- **End-to-End Encrypted Messaging**: Messages are encrypted with X25519/AES-GCM and signed with Ed25519.
- **WebSocket Relay Server**: Simple relay for real-time message delivery.
- **Command-Line Interface**: Manage users, keys, and chat securely from the terminal.
- **Interactive Secure Chat**: Encrypted chat sessions over WebSocket.

---

## Quick Start

### 1. Build

```sh
cargo build --release
```

### 2. Run the WebSocket Server

```sh
cargo run --bin server
```

The server will listen on `ws://localhost:9001`.

### 3. Create Users

```sh
vaultsyn create-user alice
vaultsyn create-user bob
```

### 4. Export Public Keys

```sh
vaultsyn export-public-key alice
vaultsyn export-public-key bob
```

Share the output with your chat partners.

### 5. Send an Encrypted Message

Suppose Alice wants to send Bob a message. She needs Bob's X25519 public key:

```sh
vaultsyn send-message \
  --sender alice \
  --receiver-public-key <bob_x25519_public> \
  --message "Hello, Bob!"
```

This outputs a JSON envelope.

### 6. Receive and Decrypt a Message

Bob receives the envelope and Alice's public keys:

```sh
vaultsyn receive-message \
  --receiver bob \
  --sender-ed25519-pub <alice_ed25519_public> \
  --sender-x25519-pub <alice_x25519_public> \
  --envelope-json '<json_envelope>'
```

If the message is authentic and decrypts, it will be displayed.

### 7. Real-Time Secure Chat

Start the server:

```sh
cargo run --bin server
```

Start a secure chat session (Alice):

```sh
vaultsyn chat \
  --url ws://localhost:9001 \
  --sender alice \
  --receiver-x25519-pub <bob_x25519_public>
```

Bob does the same, using Alice's public key.

---

## Command-Line Usage

```sh
vaultsyn --help
```

**Commands:**

- `create-user <username>`: Create a new user identity.
- `export-public-key <username>`: Export a user's public keys as JSON.
- `send-message --sender <username> --receiver-public-key <base64> --message <text>`: Encrypt and sign a message.
- `receive-message --receiver <username> --sender-ed25519-pub <base64> --sender-x25519-pub <base64> --envelope-json <json>`: Decrypt and verify a message.
- `connect --url <ws_url>`: Connect to the WebSocket relay (plaintext).
- `chat --url <ws_url> --sender <username> --receiver-x25519-pub <base64>`: Start an encrypted chat session.

---

## How It Works

- **Identities**: Each user has an Ed25519 keypair (for signatures) and an X25519 keypair (for ECDH key exchange).
- **Encryption**: Messages are encrypted using a shared AES-GCM key derived from X25519 ECDH.
- **Signing**: The ciphertext is signed with the sender's Ed25519 private key.
- **Transport**: Messages are sent as JSON envelopes, optionally over a WebSocket relay.

---

## Directory Structure

```
src/
  bin/server.rs      # WebSocket relay server
  main.rs            # CLI entry point
  commands/          # CLI command implementations
  crypto/            # Key generation and cryptography
  io/                # Identity storage
  message.rs         # Message encryption/decryption
  network.rs         # WebSocket client and chat logic
```

---

## Security Notes

- **Private keys** are stored locally in your OS user data directory (see `directories` crate).
- **Never share your private keys**. Only share public keys.
- The relay server does **not** see plaintext messages; all encryption is end-to-end.

---

## Dependencies

- [tokio](https://crates.io/crates/tokio)
- [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite)
- [ed25519-dalek](https://crates.io/crates/ed25519-dalek)
- [x25519-dalek](https://crates.io/crates/x25519-dalek)
- [aes-gcm](https://crates.io/crates/aes-gcm)
- [clap](https://crates.io/crates/clap)
- [serde](https://crates.io/crates/serde)
- [directories](https://crates.io/crates/directories)
- [crossterm](https://crates.io/crates/crossterm)
- [rustyline](https://crates.io/crates/rustyline)

---

## License

MIT

---

## Author

Gamingstein

---
