use crate::modules::audit_action::{resolve_action, AuditAction};
use serde::{Deserialize, Serialize};

use super::*;

pub(crate) const SPLIT_ROW_CAP: i64 = 500;

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
        match enqueue(action, subject, "queued", instance_id) {
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
        self.finish("ok", detail, None);
    }

    pub fn succeed_with_payload(&mut self, detail: &str, payload_json: &str) {
        self.finish("ok", detail, Some(payload_json));
    }

    pub fn fail(&mut self, detail: &str) {
        self.finish("fail", detail, None);
    }

    pub(crate) fn finish(&mut self, status: &str, detail: &str, payload_json: Option<&str>) {
        if !self.open {
            return;
        }
        if let Err(err) = complete(&self.id, status, detail, payload_json) {
            crate::modules::logger::log_warn(&format!("[History] complete failed: {}", err));
        }
        self.open = false;
    }
}

impl Drop for AuditTask {
    pub(crate) fn drop(&mut self) {
        if self.open {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                complete(&self.id, "fail", "stopped before the task finished", None),
                "complete",
            );
        }
    }
}
