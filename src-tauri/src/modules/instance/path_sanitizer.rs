//! Cloned Profile Recent Paths Sanitizer.
//! Sanitizes `openedPathsList` in `storage.json` to filter out non-existent directories.

use std::path::Path;

/// Outcome of recent paths sanitization.
#[derive(Debug, Clone, Default)]
pub struct SanitizedPathsReport {
    pub retained_paths: Vec<String>,
    pub purged_nonexistent_paths: Vec<String>,
}

/// Cleans a list of workspace path strings, retaining only folders that exist on disk.
pub fn sanitize_workspace_path_entries(paths: &[String]) -> SanitizedPathsReport {
    let mut retained = Vec::new();
    let mut purged = Vec::new();

    for p in paths {
        // Strip file:/// prefix if URI
        let cleaned = if let Some(stripped) = p.strip_prefix("file:///") {
            stripped.to_string()
        } else if let Some(stripped) = p.strip_prefix("file://") {
            stripped.to_string()
        } else {
            p.clone()
        };

        if Path::new(&cleaned).exists() {
            retained.push(p.clone());
        } else {
            purged.push(p.clone());
        }
    }

    SanitizedPathsReport {
        retained_paths: retained,
        purged_nonexistent_paths: purged,
    }
}
