use crate::modules::audit_action::{resolve_action, AuditAction};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerFacts {
    pub scheduler_run_id: String,
    pub project_name: String,
    pub repo_path: String,
    pub prompt_id: String,
    pub prompt_preview: String,
    pub conversation_id: String,
    pub action_taken: String,
    pub reason: String,
    pub idle_check_passed: bool,
    pub instance_id: String,
    pub timestamp: i64,
}

pub fn scheduler_payload(facts: &SchedulerFacts) -> String {
    serde_json::to_string(facts).unwrap_or_else(|_| "{}".to_string())
}

pub fn record_scheduler_event(facts: &SchedulerFacts) -> Result<String, String> {
    let mut task = AuditTask::start(
        AuditAction::SchedulePrompt,
        &facts.project_name,
        Some(&facts.instance_id),
    );
    let payload = scheduler_payload(facts);
    task.succeed_with_payload(&facts.reason, &payload);
    Ok(task.id.clone())
}

pub fn record_requeue_event(
    project_name: &str,
    instance_id: &str,
    conversation_id: &str,
    reason: &str,
    prompt_preview: &str,
) -> Result<String, String> {
    let mut task = AuditTask::start(
        AuditAction::RequeueConversation,
        project_name,
        Some(instance_id),
    );
    let payload = serde_json::json!({
        "project_name": project_name,
        "instance_id": instance_id,
        "conversation_id": conversation_id,
        "prompt_preview": prompt_preview,
        "reason": reason,
        "requeued_at": Utc::now().timestamp(),
    })
    .to_string();
    task.succeed_with_payload(reason, &payload);
    Ok(task.id.clone())
}
