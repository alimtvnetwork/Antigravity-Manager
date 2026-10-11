use serde::{Deserialize, Serialize};

/// Top-level payload for agm/compact-export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactExportData {
    pub meta: CompactMetadata,
    #[serde(default)]
    pub pinned_projects: Vec<CompactPinnedProject>,
    #[serde(default)]
    pub conversations: Vec<CompactConversation>,
    #[serde(default)]
    pub running_prompts: Vec<CompactRunningPrompt>,
    #[serde(default)]
    pub prompt_history: Vec<CompactPromptHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactMetadata {
    pub schema_version: String,
    pub created_at: String,
    pub source_machine: String,
    pub source_work_dir: String,
    pub is_portable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactPinnedProject {
    pub project_key: String,
    pub repo_name: String,
    pub relative_path: String,
    pub pinned_at: String,
    pub is_active: bool,
    pub display_order: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactConversation {
    pub conversation_id: String,
    pub project_key: String,
    pub seq_id: i64,
    pub title: String,
    pub relative_path: String,
    pub last_active_at: String,
    pub message_count: usize,
    pub is_archived: bool,
    pub has_active_prompt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactRunningPrompt {
    pub prompt_id: String,
    pub project_key: String,
    pub conversation_id: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_payload: Option<String>,
    pub is_restored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactPromptHistory {
    pub history_id: String,
    pub project_key: String,
    pub conversation_id: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub role: String,
    pub executed_at: String,
    pub exit_code: Option<i32>,
    pub has_completed: bool,
    pub duration_ms: u64,
}

/// Command-line export options.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub file: Option<String>,
    pub work_dir: Option<String>,
    pub is_pinned_included: bool,
    pub is_relative: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            file: None,
            work_dir: None,
            is_pinned_included: true,
            is_relative: true,
        }
    }
}

/// Command-line import options.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub file: String,
    pub work_dir: Option<String>,
    pub is_auto_confirmed: bool,
}

/// Export outcome metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub output_path: String,
    pub is_sqlite: bool,
    pub pinned_count: usize,
    pub conversations_count: usize,
    pub running_count: usize,
    pub history_count: usize,
    pub duration_ms: u64,
}

/// Import outcome metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub source_path: String,
    pub target_work_dir: String,
    pub pinned_restored: usize,
    pub conversations_restored: usize,
    pub running_restored: usize,
    pub history_restored: usize,
    pub duration_ms: u64,
}
