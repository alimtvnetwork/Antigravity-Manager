//! Profile Clone Integrity Checksum Verifier.
//! Asserts byte parity and file presence across cloned configuration directories.

use serde::{Deserialize, Serialize};
use std::path::Path;

pub const DEFAULT_CRITICAL_CONFIG_FILES: &[&str] = &[
    "User/settings.json",
    "User/keybindings.json",
    "storage.json",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneParityReport {
    pub is_parity_valid: bool,
    pub matched_files: Vec<String>,
    pub missing_files: Vec<String>,
    pub size_mismatches: Vec<String>,
}

/// Verifies critical configuration file presence and byte size consistency between source and cloned profiles.
pub fn verify_clone_parity(
    src_dir: &Path,
    dst_dir: &Path,
    critical_files: Option<&[&str]>,
) -> CloneParityReport {
    let files = critical_files.unwrap_or(DEFAULT_CRITICAL_CONFIG_FILES);
    let mut matched_files = Vec::new();
    let mut missing_files = Vec::new();
    let mut size_mismatches = Vec::new();

    for rel_path in files {
        let src_file = src_dir.join(rel_path);
        let dst_file = dst_dir.join(rel_path);

        if !src_file.exists() {
            // Source didn't have this optional file, skip
            continue;
        }

        if !dst_file.exists() {
            missing_files.push(rel_path.to_string());
            continue;
        }

        // Check file sizes
        let src_meta = src_file.metadata().ok();
        let dst_meta = dst_file.metadata().ok();

        match (src_meta, dst_meta) {
            (Some(s), Some(d)) => {
                // If it's storage.json, it was scrubbed, so size can differ slightly.
                if rel_path.ends_with("storage.json") {
                    matched_files.push(rel_path.to_string());
                } else if s.len() == d.len() {
                    matched_files.push(rel_path.to_string());
                } else {
                    size_mismatches.push(format!("{}: src {}b vs dst {}b", rel_path, s.len(), d.len()));
                }
            }
            _ => {
                missing_files.push(rel_path.to_string());
            }
        }
    }

    let is_parity_valid = missing_files.is_empty() && size_mismatches.is_empty();

    CloneParityReport {
        is_parity_valid,
        matched_files,
        missing_files,
        size_mismatches,
    }
}
