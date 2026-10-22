//! Instance Workspace Continuity Detector.
//! Validates that cloned instance profiles retain working project references in workspaceStorage.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContinuityCheck {
    pub has_continuity: bool,
    pub target_workspace_path: Option<String>,
    pub storage_entry_count: usize,
}

/// Inspects an instance's workspaceStorage folder to ensure project continuity across switches.
pub fn verify_workspace_continuity(data_dir: &Path) -> WorkspaceContinuityCheck {
    let ws_dir = data_dir.join("User").join("workspaceStorage");
    if !ws_dir.exists() || !ws_dir.is_dir() {
        return WorkspaceContinuityCheck {
            has_continuity: false,
            target_workspace_path: None,
            storage_entry_count: 0,
        };
    }

    let mut entry_count = 0usize;
    let mut detected_target = None;

    if let Ok(entries) = fs::read_dir(&ws_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                entry_count += 1;
                let ws_json = entry.path().join("workspace.json");
                if ws_json.exists() && detected_target.is_none() {
                    if let Ok(content) = fs::read_to_string(&ws_json) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(folder) = val.get("folder").and_then(|f| f.as_str()) {
                                detected_target = Some(folder.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    WorkspaceContinuityCheck {
        has_continuity: entry_count > 0,
        target_workspace_path: detected_target,
        storage_entry_count: entry_count,
    }
}
