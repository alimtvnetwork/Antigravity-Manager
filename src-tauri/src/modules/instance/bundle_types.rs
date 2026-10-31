//! IDE Profile Bundle Schema & Data Models (`.agmide` / `.json`).
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const AGM_IDE_BUNDLE_SCHEMA_VERSION: &str = "1.0";
pub const DEFAULT_BUNDLE_EXTENSION: &str = "agmide";

/// Full container for an exported IDE instance profile bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmIdeProfileBundle {
    pub agm_bundle_version: String,
    pub exported_at: String,
    pub source_environment: SourceEnvironment,
    pub settings: BundleSettings,
    pub projects: BundleProjects,
    pub prompts: BundlePrompts,
    pub metadata: BundleMetadata,
}

/// Metadata describing the originating machine and AGM installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEnvironment {
    pub instance_id: String,
    pub instance_name: String,
    pub agm_version: String,
    pub platform: String,
    pub arch: String,
}

/// Serialized settings, keybindings, and snippet assets.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleSettings {
    pub editor_settings: serde_json::Value,
    pub keybindings: Vec<serde_json::Value>,
    pub snippets: HashMap<String, String>,
}

/// Serialized workspace URIs, pinned project sequences, and active projects.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleProjects {
    pub workspace_uris: Vec<String>,
    pub pinned_project_sequences: Vec<BundlePinnedProject>,
    pub running_projects: Vec<BundleRunningProject>,
}

/// Pinned project sequence record corresponding to `agm_project_sequences`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePinnedProject {
    pub project_key: String,
    pub seq_id: i64,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
}

/// Running project descriptor corresponding to `running_projects`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRunningProject {
    pub id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub workspace_storage_path: Option<String>,
}

/// Prompt queue snapshot, templates, and system instruction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundlePrompts {
    pub active_prompts: Vec<BundleActivePrompt>,
    pub suffix_templates: Vec<BundleSuffixTemplate>,
    pub resend_as_templates: Vec<BundleResendAsTemplate>,
    pub global_system_prompt: Option<String>,
}

/// Prompt row corresponding to `active_prompts` in strict FIFO ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleActivePrompt {
    pub id: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Suffix template record corresponding to `prompt_suffix_templates`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSuffixTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub template: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Resend As template record corresponding to `prompt_resend_as_templates`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResendAsTemplate {
    pub id: String,
    pub name: String,
    pub title_pattern: String,
    pub target_mode: String,
    pub suffix_id: Option<String>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Integrity checksum and summary counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    pub bundle_checksum: String,
    pub item_counts: HashMap<String, usize>,
}

/// Parameter options provided when importing an IDE bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub target_instance_id: Option<String>,
    pub new_instance_name: Option<String>,
    pub force: bool,
}

/// Execution summary returned upon successful profile bundle import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub instance_id: String,
    pub instance_name: String,
    pub settings_applied: usize,
    pub projects_restored: usize,
    pub prompts_queued: usize,
}
