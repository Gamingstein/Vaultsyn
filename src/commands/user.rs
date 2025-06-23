use crate::crypto::keygen::{generate_identity, Identity};
use crate::io::storage::save_identity;

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
