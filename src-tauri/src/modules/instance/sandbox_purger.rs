//! Ephemeral Sandbox Profile Purger.
//! Safely removes leftover temporary test sandbox folders and profiles with active file lock detection.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPurgeReport {
    pub deleted_paths: Vec<String>,
    pub skipped_locked_paths: Vec<String>,
    pub total_reclaimed_bytes: u64,
}

/// Discovers and removes ephemeral test profiles matching prefix conventions.
pub fn purge_ephemeral_sandboxes(parent_dir: &Path) -> std::io::Result<SandboxPurgeReport> {
    let mut deleted_paths = Vec::new();
    let mut skipped_locked_paths = Vec::new();
    let mut total_reclaimed_bytes = 0u64;

    if !parent_dir.exists() || !parent_dir.is_dir() {
        return Ok(SandboxPurgeReport {
            deleted_paths,
            skipped_locked_paths,
            total_reclaimed_bytes,
        });
    }

    for entry in fs::read_dir(parent_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let fname = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        let is_ephemeral = fname.starts_with(".antigravity-sandbox-")
            || fname.starts_with(".test-profile-")
            || fname.starts_with("agm-test-");

        if !is_ephemeral {
            continue;
        }

        let dir_size = calculate_dir_bytes(&path);
        match fs::remove_dir_all(&path) {
            Ok(_) => {
                deleted_paths.push(path.to_string_lossy().to_string());
                total_reclaimed_bytes += dir_size;
            }
            Err(_) => {
                skipped_locked_paths.push(path.to_string_lossy().to_string());
            }
        }
    }

    Ok(SandboxPurgeReport {
        deleted_paths,
        skipped_locked_paths,
        total_reclaimed_bytes,
    })
}

fn calculate_dir_bytes(dir: &Path) -> u64 {
    let mut size = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Ok(meta) = p.metadata() {
                    size += meta.len();
                }
            } else if p.is_dir() {
                size += calculate_dir_bytes(&p);
            }
        }
    }
    size
}
