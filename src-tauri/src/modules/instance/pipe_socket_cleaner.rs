//! Stale Unix and named pipe socket cleaner.
//! Cleans orphaned socket handles, IPC lock files, and socket descriptors
//! left over in instance temporary folders after abnormal process crashes.

use crate::error::AppError;
use std::path::Path;

pub struct PipeSocketCleaner;

impl PipeSocketCleaner {
    /// Scans the instance runtime directory and removes orphaned `.sock` and `.lock` socket files.
    pub fn clean_stale_sockets(runtime_dir: &Path) -> Result<usize, AppError> {
        if !runtime_dir.is_dir() {
            return Ok(0);
        }

        let mut cleaned_count = 0usize;

        let entries = std::fs::read_dir(runtime_dir)
            .map_err(AppError::Io)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

            let is_stale_socket = file_name.ends_with(".sock")
                || file_name.ends_with(".socket")
                || file_name.starts_with("antigravity-ipc-")
                || (file_name.starts_with("vscode-") && file_name.ends_with(".pipe"));

            if is_stale_socket {
                if std::fs::remove_file(&path).is_ok() {
                    cleaned_count += 1;
                }
            }
        }

        Ok(cleaned_count)
    }
}
