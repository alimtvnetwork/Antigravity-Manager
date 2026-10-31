//! Active Project Folder URI Formatter.
//! Normalizes filesystem paths into canonical `file:///...` URIs for `workspaceStorage/*/workspace.json`.

use std::path::{Path, PathBuf};

/// Formats a local path into a canonical `file:///...` URI with forward slashes.
pub fn format_project_folder_uri(path: &Path) -> String {
    let path_str = path.to_string_lossy().to_string();
    let normalized = path_str.replace('\\', "/");

    if normalized.starts_with('/') {
        format!("file://{}", normalized)
    } else {
        format!("file:///{}", normalized)
    }
}

/// Parses a `file:///...` URI back into a local PathBuf.
pub fn parse_project_folder_uri(uri: &str) -> Option<PathBuf> {
    let stripped = uri.strip_prefix("file://")?;
    let path_str =
        if cfg!(windows) && stripped.starts_with('/') && stripped.chars().nth(2) == Some(':') {
            &stripped[1..]
        } else {
            stripped
        };

    Some(PathBuf::from(path_str))
}

/// Generates valid JSON payload for `workspace.json` pointing to a workspace folder URI.
pub fn generate_workspace_json_content(folder_uri: &str) -> String {
    serde_json::json!({
        "folder": folder_uri
    })
    .to_string()
}
