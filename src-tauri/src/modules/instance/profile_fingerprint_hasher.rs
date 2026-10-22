//! Profile Structural Fingerprint Hasher
//!
//! Generates a deterministic SHA-256 fingerprint hash of an instance profile's
//! configuration directory (settings.json, keybindings.json, extensions metadata),
//! allowing Antigravity Manager to detect profile drift across multi-instance environments.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::debug;

/// Files that contribute to the configuration fingerprint.
pub const FINGERPRINT_TARGET_FILES: &[&str] = &[
    "User/settings.json",
    "User/keybindings.json",
];

/// Computes a structural fingerprint hash for an instance profile directory.
pub fn compute_profile_fingerprint(profile_dir: &Path) -> Result<String, std::io::Error> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 4096];

    for rel_path in FINGERPRINT_TARGET_FILES {
        let full_path = profile_dir.join(rel_path);
        if !full_path.exists() || !full_path.is_file() {
            hasher.update(b"MISSING:");
            hasher.update(rel_path.as_bytes());
            continue;
        }

        hasher.update(b"FILE:");
        hasher.update(rel_path.as_bytes());

        let file = File::open(&full_path)?;
        let mut reader = BufReader::new(file);

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
    }

    let hash_hex = format!("{:x}", hasher.finalize());
    debug!(
        dir = ?profile_dir,
        hash = %hash_hex,
        "Computed instance profile structural fingerprint"
    );

    Ok(hash_hex)
}
