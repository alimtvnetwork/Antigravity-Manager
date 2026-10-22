//! Profile Clone Cryptographic Manifest Generator
//! Hashes critical profile configuration files during cloning to produce a verifiable
//! cryptographic integrity manifest, detecting incomplete or corrupted duplicate profiles.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChecksumEntry {
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha256_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCloneManifest {
    pub instance_id: String,
    pub generated_at_epoch_sec: i64,
    pub files: HashMap<String, FileChecksumEntry>,
}

pub struct ProfileCloneManifestGenerator;

impl ProfileCloneManifestGenerator {
    /// List of critical relative configuration files inspected for clone manifest.
    pub const CRITICAL_FILES: &'static [&'static str] = &[
        "settings.json",
        "storage.json",
        "keybindings.json",
        "state.vscdb",
    ];

    /// Generates a checksum manifest for an instance profile directory.
    pub fn generate_manifest(instance_dir: &Path, instance_id: &str) -> ProfileCloneManifest {
        let mut files = HashMap::new();

        for &rel in Self::CRITICAL_FILES {
            let target = instance_dir.join(rel);
            if target.exists() && target.is_file() {
                if let Ok(bytes) = std::fs::read(&target) {
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let sha256_hex = format!("{:x}", hasher.finalize());

                    files.insert(
                        rel.to_string(),
                        FileChecksumEntry {
                            relative_path: rel.to_string(),
                            size_bytes: bytes.len() as u64,
                            sha256_hex,
                        },
                    );
                }
            }
        }

        ProfileCloneManifest {
            instance_id: instance_id.to_string(),
            generated_at_epoch_sec: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            files,
        }
    }

    /// Verifies that destination profile has matching file checksums with source manifest.
    pub fn verify_manifest(
        destination_dir: &Path,
        source_manifest: &ProfileCloneManifest,
    ) -> (bool, Vec<String>) {
        let mut mismatches = Vec::new();

        for (rel_path, entry) in &source_manifest.files {
            let dest_file = destination_dir.join(rel_path);
            if !dest_file.exists() {
                mismatches.push(format!("Missing file in clone: {}", rel_path));
                continue;
            }

            match std::fs::read(&dest_file) {
                Ok(bytes) => {
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let dest_hex = format!("{:x}", hasher.finalize());

                    if dest_hex != entry.sha256_hex {
                        mismatches.push(format!(
                            "Checksum mismatch for {}: expected {}, got {}",
                            rel_path, entry.sha256_hex, dest_hex
                        ));
                    }
                }
                Err(e) => {
                    mismatches.push(format!("Failed to read clone file {}: {}", rel_path, e));
                }
            }
        }

        let is_valid = mismatches.is_empty();
        (is_valid, mismatches)
    }
}
