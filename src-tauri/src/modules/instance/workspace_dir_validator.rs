//! Instance workspace directory validator.
//! Validates target workspace directories before launching Antigravity IDE instances,
//! verifying filesystem existence, write accessibility, and project markers.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceValidationReport {
    pub workspace_path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub is_writable: bool,
    pub detected_project_markers: Vec<String>,
}

pub struct WorkspaceDirValidator;

impl WorkspaceDirValidator {
    /// Validates that a workspace path is valid and launchable.
    pub fn validate_workspace(workspace_dir: &Path) -> Result<WorkspaceValidationReport, AppError> {
        let path_str = workspace_dir.to_string_lossy().to_string();

        if !workspace_dir.exists() {
            return Ok(WorkspaceValidationReport {
                workspace_path: path_str,
                exists: false,
                is_directory: false,
                is_writable: false,
                detected_project_markers: Vec::new(),
            });
        }

        let is_dir = workspace_dir.is_dir();
        let is_writable = Self::test_write_permission(workspace_dir);

        let mut markers = Vec::new();
        let candidate_markers = [".git", "package.json", "Cargo.toml", "go.mod", "pyproject.toml"];

        for marker in candidate_markers {
            if workspace_dir.join(marker).exists() {
                markers.push(marker.to_string());
            }
        }

        Ok(WorkspaceValidationReport {
            workspace_path: path_str,
            exists: true,
            is_directory: is_dir,
            is_writable,
            detected_project_markers: markers,
        })
    }

    fn test_write_permission(dir: &Path) -> bool {
        let test_file = dir.join(".agm_write_test_tmp");
        if std::fs::write(&test_file, b"test").is_ok() {
            let _ = std::fs::remove_file(test_file);
            true
        } else {
            false
        }
    }
}
