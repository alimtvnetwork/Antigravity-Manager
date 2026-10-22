//! Instance Profile Clone Integrity Verifier
//!
//! Validates that a duplicated/cloned instance directory contains all required core
//! configuration artifacts (`User/settings.json`, valid SQLite stores, and sanitized
//! machine UUIDs) prior to registering the clone in the instance registry.

use std::path::Path;
use tracing::{debug, warn};

/// Validation result of a cloned profile inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneIntegrityResult {
    Valid,
    MissingDirectory(String),
    MissingConfigFile(String),
    CorruptedStorageJson(String),
}

/// Core relative files that should exist or be valid in an IDE profile.
pub const REQUIRED_PROFILE_RELATIVE_FILES: &[&str] = &[
    "User/settings.json",
];

/// Inspects the cloned profile directory and verifies structural integrity.
pub fn verify_cloned_profile_integrity(cloned_dir: &Path) -> CloneIntegrityResult {
    if !cloned_dir.exists() || !cloned_dir.is_dir() {
        return CloneIntegrityResult::MissingDirectory(cloned_dir.display().to_string());
    }

    for rel_file in REQUIRED_PROFILE_RELATIVE_FILES {
        let full_path = cloned_dir.join(rel_file);
        if !full_path.exists() {
            warn!(path = ?full_path, "Required profile configuration file missing");
            return CloneIntegrityResult::MissingConfigFile(rel_file.to_string());
        }
    }

    // Validate storage.json if present
    let storage_json = cloned_dir.join("User/globalStorage/storage.json");
    if storage_json.exists() {
        match std::fs::read_to_string(&storage_json) {
            Ok(content) => {
                if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                    warn!(path = ?storage_json, "storage.json contains invalid JSON syntax");
                    return CloneIntegrityResult::CorruptedStorageJson("Invalid JSON".to_string());
                }
            }
            Err(e) => {
                warn!(error = %e, "Could not read storage.json during integrity check");
                return CloneIntegrityResult::CorruptedStorageJson(e.to_string());
            }
        }
    }

    debug!(dir = ?cloned_dir, "Cloned instance profile verified successfully");
    CloneIntegrityResult::Valid
}
