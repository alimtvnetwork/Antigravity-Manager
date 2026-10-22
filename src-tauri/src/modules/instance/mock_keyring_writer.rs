//! Mock Keyring Marker Writer
//! Writes filesystem markers to signal to child Antigravity IDE and language server
//! binaries that the OS keyring service is unavailable, forcing isolated file credential mode.

use std::fs;
use std::path::Path;
use tracing::{debug, info};

pub struct MockKeyringWriter;

impl MockKeyringWriter {
    pub const MARKER_FILENAMES: &'static [&'static str] = &[
        "antigravity-keyring-unavailable",
        "antigravity-ide-keyring-unavailable",
        "antigravity-cli-keyring-unavailable",
    ];

    /// Writes standard keyring bypass markers into the target directory.
    pub fn write_markers(target_dir: &Path) -> std::io::Result<usize> {
        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }

        let mut written_count = 0;
        for &filename in Self::MARKER_FILENAMES {
            let marker_path = target_dir.join(filename);
            if let Err(e) = fs::write(&marker_path, b"1\n") {
                debug!("Failed to write keyring marker {:?}: {}", marker_path, e);
            } else {
                written_count += 1;
            }
        }

        info!("Wrote {} keyring bypass markers to {:?}", written_count, target_dir);
        Ok(written_count)
    }
}
