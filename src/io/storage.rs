use crate::crypto::keygen::Identity;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

fn identity_path(username: &str) -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "vaultsyn", "vaultsyn").unwrap();
    let dir = proj_dirs.data_local_dir().join("users");
    fs::create_dir_all(&dir).unwrap();
    dir.join(format!("{}.json", username))
}

pub fn save_identity(identity: &Identity) -> std::io::Result<()> {
    let path = identity_path(&identity.username);
    let json = serde_json::to_string_pretty(identity)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_identity(username: &str) -> Option<Identity> {
    let path = identity_path(username);
    let json = fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}
