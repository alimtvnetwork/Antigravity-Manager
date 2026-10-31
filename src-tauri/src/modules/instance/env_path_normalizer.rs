// Sandbox Toolchain PATH Normalizer (Imp-529)
// Normalizes the system PATH environment variable for child IDE instances,
// prepending isolated sandbox directories and GitMap tool paths while eliminating duplicates.

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

pub struct EnvPathNormalizer;

impl EnvPathNormalizer {
    /// Constructs a clean, prioritized PATH string with priority_paths placed first.
    pub fn build_sandboxed_path(priority_paths: &[PathBuf]) -> String {
        let separator = if cfg!(windows) { ";" } else { ":" };
        let mut seen = HashSet::new();
        let mut ordered_paths = Vec::new();

        // 1. Prepend priority paths (sandboxed bin, gitmap, etc.)
        for path in priority_paths {
            let normalized = path.to_string_lossy().to_string();
            if !normalized.is_empty() && seen.insert(normalized.clone()) {
                ordered_paths.push(normalized);
            }
        }

        // 2. Append existing system PATH entries, omitting duplicates
        if let Ok(current_path) = env::var("PATH") {
            for entry in current_path.split(if cfg!(windows) { ';' } else { ':' }) {
                let trimmed = entry.trim();
                if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
                    ordered_paths.push(trimmed.to_string());
                }
            }
        }

        ordered_paths.join(separator)
    }

    /// Verifies if a given binary name exists within the computed path string.
    pub fn exists_in_path(binary_name: &str, path_str: &str) -> bool {
        let separator = if cfg!(windows) { ';' } else { ':' };
        for entry in path_str.split(separator) {
            let candidate = Path::new(entry).join(binary_name);
            if candidate.exists() {
                return true;
            }
            #[cfg(windows)]
            {
                let candidate_exe = Path::new(entry).join(format!("{}.exe", binary_name));
                if candidate_exe.exists() {
                    return true;
                }
            }
        }
        false
    }
}
