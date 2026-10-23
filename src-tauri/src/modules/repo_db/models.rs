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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media_thumbnails: Vec<String>,
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
    pub is_error_or_quota: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectAuditRationaleType {
    ActiveInFlightTasks,
    InstanceProcessDead,
    IdleNoActiveTasks,
    ActivePromptMemory,
    ActiveWorkerMatched,
    ActivePromptDbRunning,
    ConversationSummaryActiveTurn,
    PromptDispatchMatched,
    PromptDispatchRejected,
}

impl ProjectAuditRationaleType {
    #[inline]
    pub const fn from_liveness(is_project_active: bool, is_instance_active: bool) -> Self {
        if is_project_active {
            Self::ActiveInFlightTasks
        } else if !is_instance_active {
            Self::InstanceProcessDead
        } else {
            Self::IdleNoActiveTasks
        }
    }

    #[inline]
    pub const fn from_dispatch(is_match: bool) -> Self {
        if is_match {
            Self::PromptDispatchMatched
        } else {
            Self::PromptDispatchRejected
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ActiveInFlightTasks => "ACTIVE_IN_FLIGHT_TASKS",
            Self::InstanceProcessDead => "INSTANCE_PROCESS_DEAD",
            Self::IdleNoActiveTasks => "IDLE_NO_ACTIVE_TASKS",
            Self::ActivePromptMemory => "ACTIVE_PROMPT_MEMORY",
            Self::ActiveWorkerMatched => "ACTIVE_WORKER_MATCHED",
            Self::ActivePromptDbRunning => "ACTIVE_PROMPT_DB_RUNNING",
            Self::ConversationSummaryActiveTurn => "CONVERSATION_SUMMARY_ACTIVE_TURN",
            Self::PromptDispatchMatched => "PROMPT_DISPATCH_MATCHED",
            Self::PromptDispatchRejected => "PROMPT_DISPATCH_REJECTED",
        }
    }

    #[inline]
    pub const fn display_label(&self) -> &'static str {
        self.as_str()
    }

    #[inline]
    pub const fn is_active(&self) -> bool {
        matches!(
            self,
            Self::ActiveInFlightTasks
                | Self::ActivePromptMemory
                | Self::ActiveWorkerMatched
                | Self::ActivePromptDbRunning
                | Self::ConversationSummaryActiveTurn
        )
    }

    #[inline]
    pub const fn is_dead(&self) -> bool {
        matches!(self, Self::InstanceProcessDead)
    }

    #[inline]
    pub const fn is_idle(&self) -> bool {
        matches!(self, Self::IdleNoActiveTasks)
    }
}

impl fmt::Display for ProjectAuditRationaleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProjectAuditRationaleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "ACTIVE_IN_FLIGHT_TASKS" => Ok(Self::ActiveInFlightTasks),
            "INSTANCE_PROCESS_DEAD" => Ok(Self::InstanceProcessDead),
            "IDLE_NO_ACTIVE_TASKS" => Ok(Self::IdleNoActiveTasks),
            "ACTIVE_PROMPT_MEMORY" => Ok(Self::ActivePromptMemory),
            "ACTIVE_WORKER_MATCHED" => Ok(Self::ActiveWorkerMatched),
            "ACTIVE_PROMPT_DB_RUNNING" => Ok(Self::ActivePromptDbRunning),
            "CONVERSATION_SUMMARY_ACTIVE_TURN" => Ok(Self::ConversationSummaryActiveTurn),
            "PROMPT_DISPATCH_MATCHED" => Ok(Self::PromptDispatchMatched),
            "PROMPT_DISPATCH_REJECTED" => Ok(Self::PromptDispatchRejected),
            other => Err(format!("Unknown ProjectAuditRationaleType: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromptAuditCriteriaType {
    WorkspaceStorageScanAndProcessLiveness,
    PromptDispatcherInstanceMatching,
    Gate0HostProcessLiveness,
    Gate1MemoryPromptsMap,
    Gate2ActiveAgyWorkers,
    Gate3ActivePromptsSqlite,
    Gate4ConversationSummariesLiveTurn,
    Gate0To4AllEvaluationsCompleted,
    ProjectConversationTreeLiveness,
}

impl PromptAuditCriteriaType {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::WorkspaceStorageScanAndProcessLiveness => {
                "WorkspaceStorageScan + ProcessLiveness"
            }
            Self::PromptDispatcherInstanceMatching => "PromptDispatcher:InstanceMatching",
            Self::Gate0HostProcessLiveness => "Gate0:HostProcessLiveness",
            Self::Gate1MemoryPromptsMap => "Gate1:MemoryPromptsMap",
            Self::Gate2ActiveAgyWorkers => "Gate2:ActiveAgyWorkers",
            Self::Gate3ActivePromptsSqlite => "Gate3:ActivePromptsSQLite",
            Self::Gate4ConversationSummariesLiveTurn => "Gate4:ConversationSummariesLiveTurn",
            Self::Gate0To4AllEvaluationsCompleted => "Gate0-4:AllEvaluationsCompleted",
            Self::ProjectConversationTreeLiveness => "ProjectConversationTreeLiveness",
        }
    }

    #[inline]
    pub const fn display_label(&self) -> &'static str {
        self.as_str()
    }
}

impl fmt::Display for PromptAuditCriteriaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PromptAuditCriteriaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "WorkspaceStorageScan + ProcessLiveness" => {
                Ok(Self::WorkspaceStorageScanAndProcessLiveness)
            }
            "PromptDispatcher:InstanceMatching" => Ok(Self::PromptDispatcherInstanceMatching),
            "Gate0:HostProcessLiveness" => Ok(Self::Gate0HostProcessLiveness),
            "Gate1:MemoryPromptsMap" => Ok(Self::Gate1MemoryPromptsMap),
            "Gate2:ActiveAgyWorkers" => Ok(Self::Gate2ActiveAgyWorkers),
            "Gate3:ActivePromptsSQLite" => Ok(Self::Gate3ActivePromptsSqlite),
            "Gate4:ConversationSummariesLiveTurn" => Ok(Self::Gate4ConversationSummariesLiveTurn),
            "Gate0-4:AllEvaluationsCompleted" => Ok(Self::Gate0To4AllEvaluationsCompleted),
            "ProjectConversationTreeLiveness" => Ok(Self::ProjectConversationTreeLiveness),
            other => Err(format!("Unknown PromptAuditCriteriaType: {other}")),
        }
    }
}

/// Represents the execution run status of a conversation or turn from Antigravity Cascade engine summaries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CascadeRunStatusType {
    Running,
    Idle,
    Completed,
    Failed,
    Cancelled,
    Aborted,
    Unknown,
}

impl CascadeRunStatusType {
    /// Parse from raw Antigravity/Cascade status string (case-insensitive substring/prefix matching)
    pub fn from_status_str(status: &str) -> Self {
        let upper = status.trim().to_uppercase();
        if upper.contains("CASCADE_RUN_STATUS_IDLE") || upper == "IDLE" {
            Self::Idle
        } else if upper.contains("CASCADE_RUN_STATUS_COMPLETED")
            || upper.contains("COMPLETED")
            || upper == "DONE"
        {
            Self::Completed
        } else if upper.contains("CASCADE_RUN_STATUS_FAILED")
            || upper.contains("FAILED")
            || upper == "ERROR"
        {
            Self::Failed
        } else if upper.contains("CASCADE_RUN_STATUS_CANCELLED")
            || upper.contains("CANCELLED")
            || upper.contains("CANCELED")
        {
            Self::Cancelled
        } else if upper.contains("CASCADE_RUN_STATUS_ABORTED") || upper.contains("ABORTED") {
            Self::Aborted
        } else if upper.contains("CASCADE_RUN_STATUS_RUNNING")
            || upper.contains("RUNNING")
            || upper == "IN_FLIGHT"
        {
            Self::Running
        } else if upper.contains("IDLE") {
            Self::Idle
        } else {
            Self::Unknown
        }
    }

    /// Check if the status indicates an explicit terminal completion.
    #[inline]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Aborted
        )
    }

    /// Check if the status indicates an idle or terminal state (Explicit Idle Supremacy).
    #[inline]
    pub const fn is_idle_or_terminal(&self) -> bool {
        matches!(
            self,
            Self::Idle | Self::Completed | Self::Failed | Self::Cancelled | Self::Aborted
        )
    }

    /// Check if the status indicates active execution.
    #[inline]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Canonical string identifier
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "CASCADE_RUN_STATUS_RUNNING",
            Self::Idle => "CASCADE_RUN_STATUS_IDLE",
            Self::Completed => "CASCADE_RUN_STATUS_COMPLETED",
            Self::Failed => "CASCADE_RUN_STATUS_FAILED",
            Self::Cancelled => "CASCADE_RUN_STATUS_CANCELLED",
            Self::Aborted => "CASCADE_RUN_STATUS_ABORTED",
            Self::Unknown => "CASCADE_RUN_STATUS_UNKNOWN",
        }
    }
}

impl fmt::Display for CascadeRunStatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CascadeRunStatusType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_status_str(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_audit_rationale_type_variants_and_roundtrip() {
        let variants = [
            (
                ProjectAuditRationaleType::ActiveInFlightTasks,
                "ACTIVE_IN_FLIGHT_TASKS",
            ),
            (
                ProjectAuditRationaleType::InstanceProcessDead,
                "INSTANCE_PROCESS_DEAD",
            ),
            (
                ProjectAuditRationaleType::IdleNoActiveTasks,
                "IDLE_NO_ACTIVE_TASKS",
            ),
            (
                ProjectAuditRationaleType::ActivePromptMemory,
                "ACTIVE_PROMPT_MEMORY",
            ),
            (
                ProjectAuditRationaleType::ActiveWorkerMatched,
                "ACTIVE_WORKER_MATCHED",
            ),
            (
                ProjectAuditRationaleType::ActivePromptDbRunning,
                "ACTIVE_PROMPT_DB_RUNNING",
            ),
            (
                ProjectAuditRationaleType::ConversationSummaryActiveTurn,
                "CONVERSATION_SUMMARY_ACTIVE_TURN",
            ),
            (
                ProjectAuditRationaleType::PromptDispatchMatched,
                "PROMPT_DISPATCH_MATCHED",
            ),
            (
                ProjectAuditRationaleType::PromptDispatchRejected,
                "PROMPT_DISPATCH_REJECTED",
            ),
        ];

        for (variant, expected_str) in variants {
            assert_eq!(variant.as_str(), expected_str);
            assert_eq!(variant.display_label(), expected_str);
            assert_eq!(format!("{variant}"), expected_str);

            let parsed: ProjectAuditRationaleType = expected_str.parse().unwrap();
            assert_eq!(parsed, variant);

            // Test serde round-trip
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, format!("\"{expected_str}\""));
            let deserialized: ProjectAuditRationaleType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }

        assert!("UNKNOWN_RATIONALE"
            .parse::<ProjectAuditRationaleType>()
            .is_err());
    }

    #[test]
    fn test_project_audit_rationale_from_liveness() {
        assert_eq!(
            ProjectAuditRationaleType::from_liveness(true, true),
            ProjectAuditRationaleType::ActiveInFlightTasks
        );
        assert_eq!(
            ProjectAuditRationaleType::from_liveness(true, false),
            ProjectAuditRationaleType::ActiveInFlightTasks
        );
        assert_eq!(
            ProjectAuditRationaleType::from_liveness(false, false),
            ProjectAuditRationaleType::InstanceProcessDead
        );
        assert_eq!(
            ProjectAuditRationaleType::from_liveness(false, true),
            ProjectAuditRationaleType::IdleNoActiveTasks
        );
    }

    #[test]
    fn test_project_audit_rationale_from_dispatch() {
        assert_eq!(
            ProjectAuditRationaleType::from_dispatch(true),
            ProjectAuditRationaleType::PromptDispatchMatched
        );
        assert_eq!(
            ProjectAuditRationaleType::from_dispatch(false),
            ProjectAuditRationaleType::PromptDispatchRejected
        );
    }

    #[test]
    fn test_project_audit_rationale_predicates() {
        let active_variants = [
            ProjectAuditRationaleType::ActiveInFlightTasks,
            ProjectAuditRationaleType::ActivePromptMemory,
            ProjectAuditRationaleType::ActiveWorkerMatched,
            ProjectAuditRationaleType::ActivePromptDbRunning,
            ProjectAuditRationaleType::ConversationSummaryActiveTurn,
        ];
        for v in active_variants {
            assert!(v.is_active());
            assert!(!v.is_dead());
            assert!(!v.is_idle());
        }

        assert!(ProjectAuditRationaleType::InstanceProcessDead.is_dead());
        assert!(!ProjectAuditRationaleType::InstanceProcessDead.is_active());
        assert!(!ProjectAuditRationaleType::InstanceProcessDead.is_idle());

        assert!(ProjectAuditRationaleType::IdleNoActiveTasks.is_idle());
        assert!(!ProjectAuditRationaleType::IdleNoActiveTasks.is_active());
        assert!(!ProjectAuditRationaleType::IdleNoActiveTasks.is_dead());

        assert!(!ProjectAuditRationaleType::PromptDispatchMatched.is_active());
        assert!(!ProjectAuditRationaleType::PromptDispatchMatched.is_dead());
        assert!(!ProjectAuditRationaleType::PromptDispatchMatched.is_idle());

        assert!(!ProjectAuditRationaleType::PromptDispatchRejected.is_active());
        assert!(!ProjectAuditRationaleType::PromptDispatchRejected.is_dead());
        assert!(!ProjectAuditRationaleType::PromptDispatchRejected.is_idle());
    }

    #[test]
    fn test_prompt_audit_criteria_type_variants_and_roundtrip() {
        let variants = [
            (
                PromptAuditCriteriaType::WorkspaceStorageScanAndProcessLiveness,
                "WorkspaceStorageScan + ProcessLiveness",
            ),
            (
                PromptAuditCriteriaType::PromptDispatcherInstanceMatching,
                "PromptDispatcher:InstanceMatching",
            ),
            (
                PromptAuditCriteriaType::Gate0HostProcessLiveness,
                "Gate0:HostProcessLiveness",
            ),
            (
                PromptAuditCriteriaType::Gate1MemoryPromptsMap,
                "Gate1:MemoryPromptsMap",
            ),
            (
                PromptAuditCriteriaType::Gate2ActiveAgyWorkers,
                "Gate2:ActiveAgyWorkers",
            ),
            (
                PromptAuditCriteriaType::Gate3ActivePromptsSqlite,
                "Gate3:ActivePromptsSQLite",
            ),
            (
                PromptAuditCriteriaType::Gate4ConversationSummariesLiveTurn,
                "Gate4:ConversationSummariesLiveTurn",
            ),
            (
                PromptAuditCriteriaType::Gate0To4AllEvaluationsCompleted,
                "Gate0-4:AllEvaluationsCompleted",
            ),
            (
                PromptAuditCriteriaType::ProjectConversationTreeLiveness,
                "ProjectConversationTreeLiveness",
            ),
        ];

        for (variant, expected_str) in variants {
            assert_eq!(variant.as_str(), expected_str);
            assert_eq!(variant.display_label(), expected_str);
            assert_eq!(format!("{variant}"), expected_str);

            let parsed: PromptAuditCriteriaType = expected_str.parse().unwrap();
            assert_eq!(parsed, variant);

            // Test serde round-trip
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: PromptAuditCriteriaType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, variant);
        }

        assert!("Unknown Criteria"
            .parse::<PromptAuditCriteriaType>()
            .is_err());
    }

    #[test]
    fn test_cascade_run_status_type_variants_and_predicates() {
        assert!(
            CascadeRunStatusType::from_status_str("CASCADE_RUN_STATUS_IDLE").is_idle_or_terminal()
        );
        assert!(!CascadeRunStatusType::from_status_str("CASCADE_RUN_STATUS_IDLE").is_running());
        assert!(!CascadeRunStatusType::from_status_str("CASCADE_RUN_STATUS_IDLE").is_terminal());

        assert!(CascadeRunStatusType::from_status_str("IDLE").is_idle_or_terminal());
        assert!(
            CascadeRunStatusType::from_status_str("CASCADE_RUN_STATUS_COMPLETED").is_terminal()
        );
        assert!(
            CascadeRunStatusType::from_status_str("CASCADE_RUN_STATUS_COMPLETED")
                .is_idle_or_terminal()
        );
        assert!(CascadeRunStatusType::from_status_str("COMPLETED").is_terminal());

        assert!(CascadeRunStatusType::from_status_str("FAILED").is_terminal());
        assert!(CascadeRunStatusType::from_status_str("CANCELLED").is_terminal());
        assert!(CascadeRunStatusType::from_status_str("ABORTED").is_terminal());

        assert!(CascadeRunStatusType::from_status_str("CASCADE_RUN_STATUS_RUNNING").is_running());
        assert!(
            !CascadeRunStatusType::from_status_str("CASCADE_RUN_STATUS_RUNNING")
                .is_idle_or_terminal()
        );
        assert!(CascadeRunStatusType::from_status_str("RUNNING").is_running());

        assert_eq!(
            CascadeRunStatusType::from_status_str("something_else"),
            CascadeRunStatusType::Unknown
        );
    }
}
