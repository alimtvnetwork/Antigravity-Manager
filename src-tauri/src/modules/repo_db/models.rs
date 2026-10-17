//! Repo DB: models

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivePromptStatus {
    Running,
    BackedUp,
    Dispatched,
    Queued,
    Pending,
    Completed,
}

impl ActivePromptStatus {
    /// Return the canonical database string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::BackedUp => "backed_up",
            Self::Dispatched => "dispatched",
            Self::Queued => "queued",
            Self::Pending => "pending",
            Self::Completed => "completed",
        }
    }

    /// Check if the prompt is actively executing or dispatched.
    #[inline]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Dispatched)
    }

    /// Check if the prompt is waiting in the dispatch queue.
    #[inline]
    pub const fn is_queued_or_pending(&self) -> bool {
        matches!(self, Self::Queued | Self::Pending)
    }

    /// Check if the prompt reached terminal completion.
    #[inline]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

impl fmt::Display for ActivePromptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ActivePromptStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "running" => Ok(Self::Running),
            "backed_up" | "backedup" => Ok(Self::BackedUp),
            "dispatched" | "in_flight" => Ok(Self::Dispatched),
            "queued" => Ok(Self::Queued),
            "pending" => Ok(Self::Pending),
            "completed" | "done" => Ok(Self::Completed),
            other => Err(format!("Unknown ActivePromptStatus: '{other}'")),
        }
    }
}

/// Common status subsets for SQL queries across repo_db
pub const ACTIVE_PROMPT_IN_FLIGHT_STATUSES: &[&str] = &["running", "in_flight", "dispatched"];
pub const ACTIVE_PROMPT_QUEUED_STATUSES: &[&str] = &["queued", "pending"];

/// Represents an active or running project workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningProject {
    pub id: String,
    pub instance_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub workspace_storage_path: Option<String>,
    pub is_running: bool,
    pub last_detected_at: i64,
}

/// Represents detailed live execution state for a project workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExecutionInfo {
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub is_running: bool,
    pub is_idle: bool,
    pub status: String,
    pub active_prompt: Option<String>,
    pub last_detected_at: i64,
}

/// Represents a prompt captured from a running project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePrompt {
    pub id: String,
    pub project_id: String,
    pub instance_id: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub status: String, // "running", "backed_up", "dispatched", "completed"
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub image_payload: Option<String>,
}

impl ActivePrompt {
    pub fn prompt_status(&self) -> Result<ActivePromptStatus, String> {
        self.status.parse()
    }
}

/// Metadata for an auto-resumed prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResumePromptInfo {
    pub project_id: String,
    pub repo_path: String,
    pub prompt_preview: String,
    pub has_image: bool,
}

/// Outcome of the fast-forward auto-resume operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResumeResult {
    pub instance_id: String,
    pub account_email: String,
    pub resumed_project_count: usize,
    pub skipped_project_count: usize,
    pub resumed_prompts: Vec<AutoResumePromptInfo>,
}

/// Telemetry record for instance prompt liveness probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptLivenessProbe {
    pub target_instance_id: String,
    pub target_instance_name: String,
    pub data_dir: String,
    pub is_instance_alive: bool,
    pub matched_pids: Vec<u32>,
    pub workspace_folders_count: usize,
    pub evaluated_projects_count: usize,
    pub running_projects_count: usize,
}

/// Detailed evaluation record for an individual project's liveness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLivenessEvaluation {
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub is_running: bool,
    pub active_tasks_count: usize,
    pub non_idle_convs_count: usize,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedCommandRecord {
    pub id: i64,
    pub command: String,
    pub full_args: String,
    pub domain: String,
    pub error_code: String,
    pub message: String,
    pub suggestions: String,
    pub hit_count: i64,
    pub working_dir: String,
    pub agm_version: String,
    pub is_resolved: bool,
    pub created_at: String,
    pub last_seen_at: String,
}

pub struct SwitchPromptSnap {
    pub prompt_id: String,
    pub prompt_text: String,
    pub conversation_id: String,
}

impl SwitchPromptSnap {
    pub(crate) fn empty() -> Self {
        Self {
            prompt_id: String::new(),
            prompt_text: String::new(),
            conversation_id: String::new(),
        }
    }
}

/// Conversation node inside an AGM Project Tree (with dual AGM `C001` & GitMap `GM:<short_id>` sequence codes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmConversationNode {
    pub seq_id: i64,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub conversation_id: String,
    pub short_id: String,
    pub title: String,
    pub status: String,
    pub is_running: bool,
    pub step_count: usize,
    pub instance_id: String,
    #[serde(default)]
    pub instance_seq_num: Option<u32>,
    #[serde(default)]
    pub instance_name: String,
    #[serde(default)]
    pub instance_exe_name: String,
    pub prompt_preview_200w: String,
    #[serde(default)]
    pub prompt_tail_snippet: String,
    pub prompt_word_count: usize,
    pub last_modified: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_badge: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_runs: Vec<AgmConversationNode>,
    #[serde(default)]
    pub prompt_category: String,
    #[serde(default)]
    pub is_queued: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_step_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_response: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_results: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_prompt_text: Option<String>,
    /// Tier 3: Chronological list of all prompt turns dispatched to this conversation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prompts: Vec<AgmPromptTurnNode>,
}

/// Individual prompt turn within a conversation (Tier 3 of Prompt Tree)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmPromptTurnNode {
    pub turn_number: usize,
    pub step_index: usize,
    pub prompt_text: String,
    pub preview_200w: String,
    pub status: String,
    pub is_running: bool,
    pub is_queued: bool,
    pub timestamp: String,
    pub created_at: i64,
    pub media_paths: Vec<String>,
    pub media_thumbnails: Vec<String>,
}

/// Project node in the AGM Tree View (Project -> Conversation -> 200-Word Prompt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmProjectTreeNode {
    pub seq_id: i64,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub instance_id: String,
    pub instance_seq_num: Option<u32>,
    pub instance_name: String,
    #[serde(default)]
    pub instance_exe_name: String,
    pub bound_email: Option<String>,
    pub is_running: bool,
    #[serde(default)]
    pub running_count: usize,
    #[serde(default)]
    pub queued_count: usize,
    pub conversations: Vec<AgmConversationNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_badge: Option<String>,
}

pub type ProjectConversationItem = AgmProjectTreeNode;
pub type ConversationItem = AgmConversationNode;

/// Resolved target from an AGM or GitMap Sequence ID (`P001`, `AGM:P001`, `GM:#1`, `C001`, `AGM:C001`, `GM:<cid>`, or conversation UUID prefix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmSequenceResolution {
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub instance_id: String,
    pub conversation_id: Option<String>,
    pub conversation_title: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TranscriptInspection {
    pub step_count: usize,
    pub latest_prompt: Option<String>,
    pub latest_response: Option<String>,
    pub execution_results: Option<String>,
    pub tool_calls_summary: Option<String>,
    pub latest_step_summary: Option<String>,
    pub is_subagent: bool,
    pub is_recent_active: bool,
    pub is_non_prompt: bool,
    pub is_terminal_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGoalHeartbeatConfig {
    pub prompt_id: String,
    pub instance_id: String,
    pub data_dir: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub heartbeat_file: String,
    pub interval_secs: u64,
    pub account_email: String,
    pub worker_pid: Option<u32>,
    pub started_at: i64,
    pub last_heartbeat_at: Option<i64>,
}
