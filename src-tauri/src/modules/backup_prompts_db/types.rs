use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBackupRecord {
    pub id: String,
    pub backup_batch_id: String,
    pub prompt_id: String,
    pub project_name: String,
    pub project_path: String,
    pub project_id: String,
    pub conversation_id: String,
    pub conversation_name: Option<String>,
    pub sequence_id: i64,
    pub prompt_text: String,
    pub has_images: bool,
    pub images_payload: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub is_restored: bool,
    pub restored_at: Option<i64>,
    pub instance_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupBatchInfo {
    pub id: String,
    pub created_at: i64,
    pub prompts_count: usize,
    pub file_path: String,
    pub retention_days: i64,
    pub is_fully_restored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenProjectRecord {
    pub id: String,
    pub project_identifier: String,
    pub project_path: String,
    pub added_at: i64,
    pub status: String,
    pub last_checked_at: Option<i64>,
}

/// Helper to resolve a human-readable friendly project name from path or ID without raw UUIDs
pub fn resolve_friendly_project_name(project_path: &str, project_id: &str) -> String {
    let clean_path = project_path.trim().replace('\\', "/");
    let trimmed_path = clean_path.trim_end_matches('/');
    if let Some(pos) = trimmed_path.rfind('/') {
        let name = &trimmed_path[pos + 1..];
        if !name.is_empty() && name != "." {
            return name.to_string();
        }
    } else if !trimmed_path.is_empty() && trimmed_path != "." {
        return trimmed_path.to_string();
    }

    let is_raw_uuid_or_hash = project_id.is_empty()
        || (project_id.len() >= 32
            && project_id
                .chars()
                .all(|c| c.is_ascii_hexdigit() || c == '-'));

    if !is_raw_uuid_or_hash {
        return project_id.to_string();
    }

    "Antigravity-Manager".to_string()
}
