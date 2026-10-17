use crate::modules::audit_action::{resolve_action, AuditAction};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::*;

pub(crate) const SPLIT_ROW_CAP: i64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditTaskStatus {
    Queued,
    Running,
    #[serde(rename = "ok")]
    Succeeded,
    #[serde(rename = "fail")]
    Failed,
}

impl AuditTaskStatus {
    /// Return the wire string format persisted in SQLite and communicated over IPC.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "ok",
            Self::Failed => "fail",
        }
    }

    /// Check if the task has concluded execution.
    #[inline]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }

    /// Check if the task is actively queued or running.
    #[inline]
    pub const fn is_in_flight(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

impl fmt::Display for AuditTaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AuditTaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "ok" | "succeeded" | "success" => Ok(Self::Succeeded),
            "fail" | "failed" | "error" => Ok(Self::Failed),
            other => Err(format!("Unknown AuditTaskStatus: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SplitInfo {
    pub id: String,
    pub file_path: String,
    pub created_at: i64,
    pub closed_at: Option<i64>,
    pub row_count: i64,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskRecord {
    pub id: String,
    pub action_code: i32,
    pub action: String,
    pub action_label: String,
    pub status: String,
    pub subject: String,
    pub detail: String,
    pub instance_id: String,
    pub split_path: String,
    pub from_email: String,
    pub to_email: String,
    pub created_at: i64,
    pub finished_at: Option<i64>,
}

impl TaskRecord {
    pub fn task_status(&self) -> Result<AuditTaskStatus, String> {
        self.status.parse()
    }
}

/// Backwards-compatible alias for TaskRecord
pub type TaskItemRecord = TaskRecord;

/// Typed insert payload for audit task recording
#[derive(Debug, Clone)]
pub struct TaskItemInsert {
    pub action: AuditAction,
    pub subject: String,
    pub status: AuditTaskStatus,
    pub detail: String,
    pub instance_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskDetail {
    pub id: String,
    pub action_code: i32,
    pub action: String,
    pub action_label: String,
    pub status: String,
    pub subject: String,
    pub detail: String,
    pub instance_id: String,
    pub split_path: String,
    pub created_at: i64,
    pub finished_at: Option<i64>,
    pub payload_json: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskHistoryPage {
    pub total: i64,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<TaskRecord>,
    pub splits: Vec<SplitInfo>,
}

pub struct AuditTask {
    pub(crate) id: String,
    pub(crate) open: bool,
}

impl AuditTask {
    pub fn start(action: AuditAction, subject: &str, instance_id: Option<&str>) -> Self {
        match enqueue(
            action,
            subject,
            AuditTaskStatus::Queued.as_str(),
            instance_id,
        ) {
            Ok(id) => Self { id, open: true },
            Err(err) => {
                crate::modules::logger::log_warn(&format!("[History] enqueue failed: {}", err));
                Self {
                    id: String::new(),
                    open: false,
                }
            }
        }
    }

    pub fn succeed(&mut self, detail: &str) {
        self.finish(AuditTaskStatus::Succeeded, detail, None);
    }

    pub fn succeed_with_payload(&mut self, detail: &str, payload_json: &str) {
        self.finish(AuditTaskStatus::Succeeded, detail, Some(payload_json));
    }

    pub fn fail(&mut self, detail: &str) {
        self.finish(AuditTaskStatus::Failed, detail, None);
    }

    pub(crate) fn finish(
        &mut self,
        status: AuditTaskStatus,
        detail: &str,
        payload_json: Option<&str>,
    ) {
        if !self.open {
            return;
        }
        if let Err(err) = complete(&self.id, status.as_str(), detail, payload_json) {
            crate::modules::logger::log_warn(&format!("[History] complete failed: {}", err));
        }
        self.open = false;
    }
}

impl Drop for AuditTask {
    fn drop(&mut self) {
        if self.open {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                complete(
                    &self.id,
                    AuditTaskStatus::Failed.as_str(),
                    "stopped before the task finished",
                    None,
                ),
                "complete",
            );
        }
    }
}
