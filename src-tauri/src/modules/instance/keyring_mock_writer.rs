//! Sandboxed keyring mock marker writer.
//! Atomically generates keyring bypass marker files in instance profile directories
//! to force Electron/Chromium to skip OS credential store prompts and use sandboxed in-memory storage.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const KEYRING_MARKER_FILENAME: &str = "antigravity-keyring-unavailable";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyringMarkerStatus {
    pub profile_dir: String,
    pub marker_path: String,
    pub is_present: bool,
    pub created_new: bool,
}

pub struct KeyringMockWriter;

impl KeyringMockWriter {
    /// Writes the keyring bypass marker file in the specified profile directory if missing.
    pub fn ensure_keyring_marker(profile_dir: &Path) -> Result<KeyringMarkerStatus, AppError> {
        let marker_path = profile_dir.join(KEYRING_MARKER_FILENAME);

        if marker_path.is_file() {
            return Ok(KeyringMarkerStatus {
                profile_dir: profile_dir.to_string_lossy().to_string(),
                marker_path: marker_path.to_string_lossy().to_string(),
                is_present: true,
                created_new: false,
            });
        }

        // Ensure parent directory exists
        if !profile_dir.is_dir() {
            std::fs::create_dir_all(profile_dir).map_err(AppError::Io)?;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let content = format!("mock-keyring-enabled=true\ncreated_epoch={}\n", timestamp);

        std::fs::write(&marker_path, content).map_err(AppError::Io)?;

        Ok(KeyringMarkerStatus {
            profile_dir: profile_dir.to_string_lossy().to_string(),
            marker_path: marker_path.to_string_lossy().to_string(),
            is_present: true,
            created_new: true,
        })
    }
}
