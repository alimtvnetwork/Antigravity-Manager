//! VS Code IPC Socket Cleaner.
//! Sweeps abandoned UNIX domain socket files, temporary lock pipes, and named socket handles
//! left behind by crashed Electron/Chromium instances prior to relaunching.

use std::fs;
use std::path::Path;

/// Sweeper metrics summary.
#[derive(Debug, Clone, Default)]
pub struct SocketCleanSummary {
    pub files_scanned: usize,
    pub sockets_removed: usize,
    pub errors_encountered: usize,
}

/// Identifies if a filename matches standard VS Code / Electron IPC socket patterns.
pub fn is_stale_ipc_socket_name(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.starts_with("vscode-ipc-")
        || lower.starts_with("antigravity-ipc-")
        || lower.ends_with(".sock")
        || lower.starts_with(".org.chromium.chrom")
}

/// Cleans stale IPC socket handles within a specified directory.
pub fn clean_stale_ipc_sockets(dir: &Path) -> SocketCleanSummary {
    let mut summary = SocketCleanSummary::default();

    if !dir.exists() || !dir.is_dir() {
        return summary;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            summary.errors_encountered += 1;
            return summary;
        }
    };

    for entry in entries.flatten() {
        summary.files_scanned += 1;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if is_stale_ipc_socket_name(&file_name) {
            let path = entry.path();
            if fs::remove_file(&path).is_ok() {
                summary.sockets_removed += 1;
            } else {
                summary.errors_encountered += 1;
            }
        }
    }

    summary
}
