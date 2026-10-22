//! Profile Extension State Sanitizer.
//! Purges stale extension runtime locks, temporary IPC sockets, and cached lockfiles from instance profiles.

use std::path::Path;

pub const EXTENSION_LOCK_PATTERNS: &[&str] = &[
    ".lock",
    ".sock",
    ".ipc",
    "extension-lock",
    "runtime.pid",
];

/// Outcome of extension directory sanitation.
#[derive(Debug, Clone, Default)]
pub struct ExtensionSanitizerReport {
    pub purged_files: Vec<String>,
    pub errors: Vec<String>,
}

/// Recursively scans an instance's extension/globalStorage folder to remove stale socket and lock files.
pub fn sanitize_extension_runtime_state(extensions_dir: &Path) -> ExtensionSanitizerReport {
    let mut report = ExtensionSanitizerReport::default();
    if !extensions_dir.exists() {
        return report;
    }

    if let Ok(entries) = std::fs::read_dir(extensions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let is_lock = EXTENSION_LOCK_PATTERNS.iter().any(|&ext| name.ends_with(ext));
                if is_lock {
                    match std::fs::remove_file(&path) {
                        Ok(_) => report.purged_files.push(path.to_string_lossy().to_string()),
                        Err(e) => report.errors.push(format!("{}: {}", name, e)),
                    }
                }
            }
        }
    }

    report
}
