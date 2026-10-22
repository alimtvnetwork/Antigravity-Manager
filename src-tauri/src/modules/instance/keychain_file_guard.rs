//! Sandboxed Keychain File Guard.
//! Verifies and ensures that mock keyring bypass markers (`antigravity-keyring-unavailable`, etc.)
//! exist with valid non-zero content (`1\n`) within the target instance's directory before launch.

use std::fs;
use std::path::Path;

/// Expected marker filenames checked by the Antigravity language server / credentials module.
pub const KEYRING_BYPASS_MARKERS: &[&str] = &[
    "antigravity-keyring-unavailable",
    "antigravity-ide-keyring-unavailable",
    "antigravity-cli-keyring-unavailable",
];

/// Verifies that all required keyring bypass marker files are present and valid.
pub fn ensure_keychain_markers_present(data_dir: &Path) -> Result<usize, std::io::Error> {
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
    }

    let mut written = 0;
    for marker in KEYRING_BYPASS_MARKERS {
        let marker_path = data_dir.join(marker);
        let needs_write = match fs::metadata(&marker_path) {
            Ok(meta) => meta.len() == 0,
            Err(_) => true,
        };

        if needs_write {
            fs::write(&marker_path, b"1\n")?;
            written += 1;
        }
    }

    Ok(written)
}

/// Checks whether all keyring markers are present without modifying filesystem.
pub fn check_keychain_markers_status(data_dir: &Path) -> bool {
    KEYRING_BYPASS_MARKERS.iter().all(|marker| {
        let path = data_dir.join(marker);
        fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false)
    })
}
