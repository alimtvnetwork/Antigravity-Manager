use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::audit_action::{resolve_action, AuditAction};

const SPLIT_ROW_CAP: i64 = 500;

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
    id: String,
    open: bool,
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

    fn finish(&mut self, status: &str, detail: &str, payload_json: Option<&str>) {
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
    fn drop(&mut self) {
        if self.open {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                complete(&self.id, "fail", "stopped before the task finished", None),
                "complete",
            );
        }
    }
}

pub fn record(
    action: AuditAction,
    subject: &str,
    status: &str,
    detail: &str,
    instance_id: Option<&str>,
) {
    match enqueue(action, subject, status, instance_id) {
        Ok(id) => {
            if status != "queued" && status != "running" {
                // Justification: best-effort call; failure logged without changing control flow
                crate::error::record_ignored(complete(&id, status, detail, None), "complete");
            }
        }
        Err(err) => {
            crate::modules::logger::log_warn(&format!("[History] record failed: {}", err));
        }
    }
}

pub fn history_dir() -> Result<PathBuf, String> {
    let local = PathBuf::from("data").join("task-history");
    if local.exists() {
        return Ok(local);
    }
    let data_dir = crate::modules::account::get_data_dir()?;
    let dir = data_dir.join("task-history");
    fs::create_dir_all(&dir)
        .map_err(|err| format!("Failed to create task-history folder: {}", err))?;
    Ok(dir)
}

pub fn list_page(offset: u32, limit: u32) -> Result<TaskHistoryPage, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    let total: i64 = splits.iter().map(|split| split.row_count).sum();

    // Fast path: if offset == 0 && limit <= 200, query hot_tasks_cache directly for sub-millisecond response
    if offset == 0 && limit <= 200 {
        let mut stmt = index
            .prepare(
                "SELECT id, action, action_code, status, subject, detail, instance_id, split_path, created_at, finished_at,
                        COALESCE(from_email, ''), COALESCE(to_email, '')
                 FROM hot_tasks_cache ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map(params![limit.max(1)], |row| {
                let legacy: String = row.get(1)?;
                let code: Option<i32> = row.get(2)?;
                let (action_code, action, action_label) = resolve_action(code, &legacy);
                Ok(TaskRecord {
                    id: row.get(0)?,
                    action_code,
                    action,
                    action_label,
                    status: row.get(3)?,
                    subject: row.get(4)?,
                    detail: row.get(5)?,
                    instance_id: row.get(6)?,
                    split_path: row.get(7)?,
                    created_at: row.get(8)?,
                    finished_at: row.get(9)?,
                    from_email: row.get(10)?,
                    to_email: row.get(11)?,
                })
            })
            .map_err(|err| err.to_string())?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| err.to_string())?);
        }
        if !items.is_empty() || total == 0 {
            return Ok(TaskHistoryPage {
                total,
                offset,
                limit,
                items,
                splits,
            });
        }
    }

    let mut items = Vec::new();
    let mut skip = offset as i64;
    let mut need = limit.max(1) as i64;
    for split in &splits {
        if need <= 0 {
            break;
        }
        if skip >= split.row_count {
            skip -= split.row_count;
            continue;
        }
        let take = need.min(split.row_count - skip);
        let conn = open_split(Path::new(&split.file_path))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, action, action_code, status, subject, detail, instance_id, created_at, finished_at,
                        COALESCE(from_email, ''), COALESCE(to_email, '')
                 FROM tasks ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map(params![take, skip], |row| {
                let legacy: String = row.get(1)?;
                let code: Option<i32> = row.get(2)?;
                let (action_code, action, action_label) = resolve_action(code, &legacy);
                Ok(TaskRecord {
                    id: row.get(0)?,
                    action_code,
                    action,
                    action_label,
                    status: row.get(3)?,
                    subject: row.get(4)?,
                    detail: row.get(5)?,
                    instance_id: row.get(6)?,
                    split_path: split.file_path.clone(),
                    created_at: row.get(7)?,
                    finished_at: row.get(8)?,
                    from_email: row.get(9)?,
                    to_email: row.get(10)?,
                })
            })
            .map_err(|err| err.to_string())?;
        for row in rows {
            items.push(row.map_err(|err| err.to_string())?);
        }
        skip = 0;
        need -= take;
    }
    Ok(TaskHistoryPage {
        total,
        offset,
        limit,
        items,
        splits,
    })
}

fn enqueue(
    action: AuditAction,
    subject: &str,
    status: &str,
    instance_id: Option<&str>,
) -> Result<String, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let (split_id, split_path) = current_split(&index, &dir)?;
    let split = open_split(&split_path)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    split
        .execute(
            "INSERT INTO tasks (id, action, action_code, status, subject, detail, instance_id, payload_json, created_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, '', ?7, NULL)",
            params![
                id,
                action.pascal(),
                action.code(),
                status,
                subject,
                instance_id.unwrap_or(""),
                now
            ],
        )
        .map_err(|err| err.to_string())?;
    index
        .execute(
            "UPDATE split_catalog SET row_count = row_count + 1 WHERE id = ?1",
            params![split_id],
        )
        .map_err(|err| err.to_string())?;
    let count: i64 = index
        .query_row(
            "SELECT row_count FROM split_catalog WHERE id = ?1",
            params![split_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if count >= SPLIT_ROW_CAP {
        index
            .execute(
                "UPDATE split_catalog SET is_current = 0, closed_at = ?1 WHERE id = ?2",
                params![now, split_id],
            )
            .map_err(|err| err.to_string())?;
    }

    // Synchronously update hot_tasks_cache and cap at 200 entries
    let split_path_str = split_path.to_string_lossy().to_string();
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "INSERT OR REPLACE INTO hot_tasks_cache (id, action, action_code, status, subject, detail, instance_id, split_path, created_at, finished_at, from_email, to_email)
         VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, ?7, ?8, NULL, '', '')",
        params![
            id,
            action.pascal(),
            action.code(),
            status,
            subject,
            instance_id.unwrap_or(""),
            split_path_str,
            now
        ],
    ), "db execute");
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "DELETE FROM hot_tasks_cache WHERE id NOT IN (SELECT id FROM hot_tasks_cache ORDER BY created_at DESC LIMIT 200)",
        [],
    ), "db execute");

    Ok(id)
}

pub fn get_detail(id: &str) -> Result<TaskDetail, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    for split in splits {
        let conn = open_split(Path::new(&split.file_path))?;
        let row = conn
            .query_row(
                "SELECT id, action, action_code, status, subject, detail, instance_id, created_at, finished_at, payload_json
                 FROM tasks WHERE id = ?1",
                params![id],
                |row| {
                    let legacy: String = row.get(1)?;
                    let code: Option<i32> = row.get(2)?;
                    let (action_code, action, action_label) = resolve_action(code, &legacy);
                    Ok(TaskDetail {
                        id: row.get(0)?,
                        action_code,
                        action,
                        action_label,
                        status: row.get(3)?,
                        subject: row.get(4)?,
                        detail: row.get(5)?,
                        instance_id: row.get(6)?,
                        split_path: split.file_path.clone(),
                        created_at: row.get(7)?,
                        finished_at: row.get(8)?,
                        payload_json: row.get(9)?,
                    })
                },
            )
            .optional()
            .map_err(|err| err.to_string())?;
        if let Some(detail) = row {
            return Ok(detail);
        }
    }
    Err(format!("history task '{}' was not found", id))
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchBackupStep {
    #[serde(default)]
    pub prompt_count: usize,
    #[serde(default)]
    pub project_names: Vec<String>,
    #[serde(default)]
    pub project_paths: Vec<String>,
    #[serde(default)]
    pub backup_batch_id: String,
    #[serde(default)]
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchResetStep {
    #[serde(default)]
    pub terminated_pids: Vec<u32>,
    #[serde(default)]
    pub auth_swapped: bool,
    #[serde(default)]
    pub credentials_injected: bool,
    #[serde(default)]
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchRestoreStep {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub restored_count: usize,
    #[serde(default)]
    pub dispatched_count: usize,
    #[serde(default)]
    pub prompt_channel_waited: bool,
    #[serde(default)]
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchVerificationStep {
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchAuditSteps {
    #[serde(default)]
    pub backup: Option<SwitchBackupStep>,
    #[serde(default)]
    pub reset: Option<SwitchResetStep>,
    #[serde(default)]
    pub restore: Option<SwitchRestoreStep>,
    #[serde(default)]
    pub verification: Option<SwitchVerificationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceSwitchRecord {
    pub id: String,
    pub action_code: i32,
    pub action: String,
    pub action_label: String,
    pub status: String,
    pub subject: String,
    pub detail: String,
    pub instance_id: String,
    #[serde(default)]
    pub instance_name: String,
    pub from_email: String,
    pub to_email: String,
    #[serde(default)]
    pub from_account_email: String,
    #[serde(default)]
    pub to_account_email: String,
    #[serde(default)]
    pub switch_reason: String,
    pub created_at: i64,
    pub finished_at: Option<i64>,
    #[serde(default)]
    pub duration_ms: i64,
    #[serde(default)]
    pub steps: Option<SwitchAuditSteps>,
    #[serde(default)]
    pub payload_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceSwitchHistoryResponse {
    pub total: usize,
    pub instance_id: String,
    pub records: Vec<InstanceSwitchRecord>,
    #[serde(default)]
    pub items: Vec<InstanceSwitchRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchFacts {
    pub from_email: String,
    pub to_email: String,
    pub reason: String,
    pub how: String,
    pub prompt_id: String,
    pub prompt_text: String,
    pub conversation_id: String,
    pub prompt_reinjected: bool,
    pub switch_ok: bool,
    #[serde(default)]
    pub instance_id: String,
    #[serde(default)]
    pub ide_type: String,
    #[serde(default)]
    pub idc_machine_alias: String,
    #[serde(default)]
    pub ide_path: String,
    #[serde(default)]
    pub switch_reason: String,
    #[serde(default)]
    pub steps: Option<SwitchAuditSteps>,
}

pub fn switch_payload(facts: &SwitchFacts) -> String {
    let mut obj = serde_json::json!({
        "from_email": facts.from_email,
        "to_email": facts.to_email,
        "reason": facts.reason,
        "how": facts.how,
        "prompt_id": facts.prompt_id,
        "prompt_text": facts.prompt_text,
        "conversation_id": facts.conversation_id,
        "prompt_reinjected": facts.prompt_reinjected,
        "moved_at": Utc::now().timestamp(),
        "switch_ok": facts.switch_ok,
        "instance_id": facts.instance_id,
        "ide_type": facts.ide_type,
        "idc_machine_alias": facts.idc_machine_alias,
        "ide_path": facts.ide_path,
        "switch_reason": facts.switch_reason,
    });
    if let Some(ref steps) = facts.steps {
        if let Ok(steps_val) = serde_json::to_value(steps) {
            obj["steps"] = steps_val;
        }
    }
    obj.to_string()
}

pub fn get_instance_switch_history(
    instance_id: &str,
    limit: usize,
) -> Result<InstanceSwitchHistoryResponse, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    let inst_id_trimmed = instance_id.trim();

    let mut total: usize = 0;
    let mut records: Vec<InstanceSwitchRecord> = Vec::new();

    let instance_name = if inst_id_trimmed == "default" || inst_id_trimmed.is_empty() {
        "Default Instance".to_string()
    } else {
        crate::modules::instance::load_registry()
            .ok()
            .and_then(|reg| {
                reg.instances
                    .into_iter()
                    .find(|i| i.id == inst_id_trimmed)
                    .map(|i| i.name)
            })
            .unwrap_or_else(|| inst_id_trimmed.to_string())
    };

    let switch_cond = "
        WHERE (action_code = 3 OR action = 'SwitchAccount' OR action = 'switch_account')
          AND (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '' OR instance_id IS NULL)))";

    for split in &splits {
        let conn = open_split(Path::new(&split.file_path))?;
        let count_sql = format!("SELECT COUNT(*) FROM tasks {}", switch_cond);
        let split_count: i64 = conn
            .query_row(&count_sql, params![inst_id_trimmed], |row| row.get(0))
            .unwrap_or(0);
        total += split_count as usize;

        if records.len() < limit {
            let need = limit - records.len();
            let query_sql = format!(
                "SELECT id, action, action_code, status, subject, detail, instance_id, created_at, finished_at, payload_json,
                        COALESCE(from_email, ''), COALESCE(to_email, '')
                 FROM tasks {}
                 ORDER BY created_at DESC LIMIT ?2",
                switch_cond
            );
            let mut stmt = conn.prepare(&query_sql).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![inst_id_trimmed, need as i64], |row| {
                    let id: String = row.get(0)?;
                    let legacy: String = row.get(1)?;
                    let code: Option<i32> = row.get(2)?;
                    let (action_code, action, action_label) = resolve_action(code, &legacy);
                    let status: String = row.get(3)?;
                    let subject: String = row.get(4)?;
                    let detail: String = row.get(5)?;
                    let inst_id: String = row.get(6)?;
                    let created_at: i64 = row.get(7)?;
                    let finished_at: Option<i64> = row.get(8)?;
                    let payload_json: String = row.get(9)?;
                    let from_email: String = row.get(10)?;
                    let to_email: String = row.get(11)?;

                    Ok((
                        id,
                        action_code,
                        action,
                        action_label,
                        status,
                        subject,
                        detail,
                        inst_id,
                        created_at,
                        finished_at,
                        payload_json,
                        from_email,
                        to_email,
                    ))
                })
                .map_err(|e| e.to_string())?;

            for r in rows {
                let (
                    id,
                    action_code,
                    action,
                    action_label,
                    status,
                    subject,
                    detail,
                    inst_id,
                    created_at,
                    finished_at,
                    payload_json,
                    mut from_email,
                    mut to_email,
                ) = r.map_err(|e| e.to_string())?;

                let parsed_val: Option<serde_json::Value> =
                    serde_json::from_str(&payload_json).ok();
                let steps = parsed_val
                    .as_ref()
                    .and_then(|v| v.get("steps"))
                    .and_then(|s| serde_json::from_value::<SwitchAuditSteps>(s.clone()).ok());

                if from_email.is_empty() {
                    if let Some(ref v) = parsed_val {
                        from_email = v
                            .get("from_email")
                            .and_then(|e| e.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                }
                if to_email.is_empty() {
                    if let Some(ref v) = parsed_val {
                        to_email = v
                            .get("to_email")
                            .and_then(|e| e.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                }

                let switch_reason = parsed_val
                    .as_ref()
                    .and_then(|v| v.get("switch_reason").or_else(|| v.get("reason")))
                    .and_then(|r| r.as_str())
                    .unwrap_or("")
                    .to_string();

                let duration_ms = match (created_at, finished_at) {
                    (c, Some(f)) if f >= c => (f - c) * 1000,
                    _ => 0,
                };

                records.push(InstanceSwitchRecord {
                    id,
                    action_code,
                    action,
                    action_label,
                    status,
                    subject,
                    detail,
                    instance_id: inst_id,
                    instance_name: instance_name.clone(),
                    from_account_email: from_email.clone(),
                    to_account_email: to_email.clone(),
                    from_email,
                    to_email,
                    switch_reason,
                    created_at,
                    finished_at,
                    duration_ms,
                    steps,
                    payload_json,
                });
            }
        }
    }

    // Fallback to hot_tasks_cache if splits had no matching records
    if total == 0 {
        let count_sql = format!("SELECT COUNT(*) FROM hot_tasks_cache {}", switch_cond);
        let hot_count: i64 = index
            .query_row(&count_sql, params![inst_id_trimmed], |row| row.get(0))
            .unwrap_or(0);
        total = hot_count as usize;

        if total > 0 && records.is_empty() {
            let query_sql = format!(
                "SELECT id, action, action_code, status, subject, detail, instance_id, created_at, finished_at, split_path,
                        COALESCE(from_email, ''), COALESCE(to_email, '')
                 FROM hot_tasks_cache {}
                 ORDER BY created_at DESC LIMIT ?2",
                switch_cond
            );
            let mut stmt = index.prepare(&query_sql).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![inst_id_trimmed, limit as i64], |row| {
                    let id: String = row.get(0)?;
                    let legacy: String = row.get(1)?;
                    let code: Option<i32> = row.get(2)?;
                    let (action_code, action, action_label) = resolve_action(code, &legacy);
                    let status: String = row.get(3)?;
                    let subject: String = row.get(4)?;
                    let detail: String = row.get(5)?;
                    let inst_id: String = row.get(6)?;
                    let created_at: i64 = row.get(7)?;
                    let finished_at: Option<i64> = row.get(8)?;
                    let split_path: String = row.get(9)?;
                    let from_email: String = row.get(10)?;
                    let to_email: String = row.get(11)?;
                    Ok((
                        id,
                        action_code,
                        action,
                        action_label,
                        status,
                        subject,
                        detail,
                        inst_id,
                        created_at,
                        finished_at,
                        split_path,
                        from_email,
                        to_email,
                    ))
                })
                .map_err(|e| e.to_string())?;

            for r in rows {
                let (
                    id,
                    action_code,
                    action,
                    action_label,
                    status,
                    subject,
                    detail,
                    inst_id,
                    created_at,
                    finished_at,
                    split_path,
                    mut from_email,
                    mut to_email,
                ) = r.map_err(|e| e.to_string())?;

                let payload_json = if !split_path.is_empty() && Path::new(&split_path).exists() {
                    open_split(Path::new(&split_path))
                        .ok()
                        .and_then(|c| {
                            c.query_row(
                                "SELECT payload_json FROM tasks WHERE id = ?1",
                                params![id],
                                |row| row.get::<_, String>(0),
                            )
                            .ok()
                        })
                        .unwrap_or_default()
                } else {
                    String::new()
                };

                let parsed_val: Option<serde_json::Value> =
                    serde_json::from_str(&payload_json).ok();
                let steps = parsed_val
                    .as_ref()
                    .and_then(|v| v.get("steps"))
                    .and_then(|s| serde_json::from_value::<SwitchAuditSteps>(s.clone()).ok());

                if from_email.is_empty() {
                    if let Some(ref v) = parsed_val {
                        from_email = v
                            .get("from_email")
                            .and_then(|e| e.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                }
                if to_email.is_empty() {
                    if let Some(ref v) = parsed_val {
                        to_email = v
                            .get("to_email")
                            .and_then(|e| e.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                }

                let duration_ms = match (created_at, finished_at) {
                    (c, Some(f)) if f >= c => (f - c) * 1000,
                    _ => 0,
                };

                records.push(InstanceSwitchRecord {
                    id,
                    action_code,
                    action,
                    action_label,
                    status,
                    subject,
                    detail,
                    instance_id: inst_id,
                    instance_name: instance_name.clone(),
                    from_account_email: from_email.clone(),
                    to_account_email: to_email.clone(),
                    from_email,
                    to_email,
                    switch_reason: "Account Switch".to_string(),
                    created_at,
                    finished_at,
                    duration_ms,
                    steps,
                    payload_json,
                });
            }
        }
    }

    Ok(InstanceSwitchHistoryResponse {
        total,
        instance_id: inst_id_trimmed.to_string(),
        records: records.clone(),
        items: records,
    })
}

pub fn get_instance_audit_trail(
    instance_id: &str,
    limit: usize,
) -> Result<Vec<TaskRecord>, String> {
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    let inst_id_trimmed = instance_id.trim();

    let mut records: Vec<TaskRecord> = Vec::new();
    let cond = "
        WHERE (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '' OR instance_id IS NULL)))";

    for split in &splits {
        if records.len() >= limit {
            break;
        }
        let need = limit - records.len();
        let conn = open_split(Path::new(&split.file_path))?;
        let query_sql = format!(
            "SELECT id, action, action_code, status, subject, detail, instance_id, created_at, finished_at,
                    COALESCE(from_email, ''), COALESCE(to_email, '')
             FROM tasks {}
             ORDER BY created_at DESC LIMIT ?2",
            cond
        );
        let mut stmt = conn.prepare(&query_sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![inst_id_trimmed, need as i64], |row| {
                let legacy: String = row.get(1)?;
                let code: Option<i32> = row.get(2)?;
                let (action_code, action, action_label) = resolve_action(code, &legacy);
                Ok(TaskRecord {
                    id: row.get(0)?,
                    action_code,
                    action,
                    action_label,
                    status: row.get(3)?,
                    subject: row.get(4)?,
                    detail: row.get(5)?,
                    instance_id: row.get(6)?,
                    split_path: split.file_path.clone(),
                    created_at: row.get(7)?,
                    finished_at: row.get(8)?,
                    from_email: row.get(9)?,
                    to_email: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for r in rows {
            records.push(r.map_err(|e| e.to_string())?);
        }
    }

    if records.is_empty() {
        let query_sql = format!(
            "SELECT id, action, action_code, status, subject, detail, instance_id, split_path, created_at, finished_at,
                    COALESCE(from_email, ''), COALESCE(to_email, '')
             FROM hot_tasks_cache {}
             ORDER BY created_at DESC LIMIT ?2",
            cond
        );
        let mut stmt = index.prepare(&query_sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![inst_id_trimmed, limit as i64], |row| {
                let legacy: String = row.get(1)?;
                let code: Option<i32> = row.get(2)?;
                let (action_code, action, action_label) = resolve_action(code, &legacy);
                Ok(TaskRecord {
                    id: row.get(0)?,
                    action_code,
                    action,
                    action_label,
                    status: row.get(3)?,
                    subject: row.get(4)?,
                    detail: row.get(5)?,
                    instance_id: row.get(6)?,
                    split_path: row.get(7)?,
                    created_at: row.get(8)?,
                    finished_at: row.get(9)?,
                    from_email: row.get(10)?,
                    to_email: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for r in rows {
            records.push(r.map_err(|e| e.to_string())?);
        }
    }

    Ok(records)
}

fn payload_emails(payload_json: Option<&str>) -> (String, String) {
    let Some(raw) = payload_json else {
        return (String::new(), String::new());
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return (String::new(), String::new());
    };
    let text = |key: &str| {
        value
            .get(key)
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string()
    };
    (text("from_email"), text("to_email"))
}

fn complete(
    id: &str,
    status: &str,
    detail: &str,
    payload_json: Option<&str>,
) -> Result<(), String> {
    if id.is_empty() {
        return Ok(());
    }
    let dir = history_dir()?;
    let index = open_index(&dir)?;
    let splits = load_splits(&index, &dir)?;
    let now = Utc::now().timestamp();
    let (from_email, to_email) = payload_emails(payload_json);
    let mut updated_in_split = false;
    for split in splits {
        let conn = open_split(Path::new(&split.file_path))?;
        let changed = if let Some(payload) = payload_json {
            conn.execute(
                "UPDATE tasks SET status = ?1, detail = ?2, finished_at = ?3, payload_json = ?4, from_email = ?5, to_email = ?6 WHERE id = ?7",
                params![status, detail, now, payload, from_email, to_email, id],
            )
            .map_err(|err| err.to_string())?
        } else {
            conn.execute(
                "UPDATE tasks SET status = ?1, detail = ?2, finished_at = ?3 WHERE id = ?4",
                params![status, detail, now, id],
            )
            .map_err(|err| err.to_string())?
        };
        if changed > 0 {
            updated_in_split = true;
            break;
        }
    }

    // Synchronously update hot_tasks_cache and cap at 200 entries
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "UPDATE hot_tasks_cache SET status = ?1, detail = ?2, finished_at = ?3, from_email = ?4, to_email = ?5 WHERE id = ?6",
        params![status, detail, now, from_email, to_email, id],
    ), "db execute");
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(index.execute(
        "DELETE FROM hot_tasks_cache WHERE id NOT IN (SELECT id FROM hot_tasks_cache ORDER BY created_at DESC LIMIT 200)",
        [],
    ), "db execute");

    if updated_in_split {
        Ok(())
    } else {
        Err(format!("history task '{}' was not found", id))
    }
}

fn history_index_path(dir: &Path) -> PathBuf {
    dir.join("task_index.db")
}

fn open_index(dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let conn = Connection::open(history_index_path(dir)).map_err(|err| err.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|err| err.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS split_catalog (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            file_path TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            closed_at INTEGER,
            row_count INTEGER NOT NULL DEFAULT 0,
            is_current INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|err| err.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS hot_tasks_cache (
            id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            action_code INTEGER,
            status TEXT NOT NULL,
            subject TEXT NOT NULL,
            detail TEXT NOT NULL DEFAULT '',
            instance_id TEXT NOT NULL DEFAULT '',
            split_path TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            finished_at INTEGER,
            from_email TEXT NOT NULL DEFAULT '',
            to_email TEXT NOT NULL DEFAULT ''
        )",
        [],
    )
    .map_err(|err| err.to_string())?;
    Ok(conn)
}

fn open_split(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let conn = Connection::open(path).map_err(|err| err.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|err| err.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            status TEXT NOT NULL,
            subject TEXT NOT NULL,
            detail TEXT NOT NULL,
            instance_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            finished_at INTEGER
        )",
        [],
    )
    .map_err(|err| err.to_string())?;
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute("ALTER TABLE tasks ADD COLUMN action_code INTEGER", []),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN payload_json TEXT NOT NULL DEFAULT ''",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN from_email TEXT NOT NULL DEFAULT ''",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN to_email TEXT NOT NULL DEFAULT ''",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "UPDATE tasks
         SET from_email = COALESCE(json_extract(payload_json, '$.from_email'), from_email),
             to_email = COALESCE(json_extract(payload_json, '$.to_email'), to_email)
         WHERE payload_json <> '' AND (from_email = '' OR to_email = '')",
            [],
        ),
        "db execute",
    );
    Ok(conn)
}

fn current_split(index: &Connection, dir: &Path) -> Result<(String, PathBuf), String> {
    let existing: Option<(String, String)> = index
        .query_row(
            "SELECT id, file_path FROM split_catalog WHERE is_current = 1 ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    if let Some((id, path)) = existing {
        return Ok((id, PathBuf::from(path)));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let path = dir.join(format!("history-{}.db", &id[..8]));
    let now = Utc::now().timestamp();
    let _ = open_split(&path)?;
    index
        .execute(
            "INSERT INTO split_catalog (id, kind, file_path, created_at, closed_at, row_count, is_current)
             VALUES (?1, 'history', ?2, ?3, NULL, 0, 1)",
            params![id, path.to_string_lossy().to_string(), now],
        )
        .map_err(|err| err.to_string())?;
    Ok((id, path))
}

fn load_splits(index: &Connection, _dir: &Path) -> Result<Vec<SplitInfo>, String> {
    let mut stmt = index
        .prepare(
            "SELECT id, file_path, created_at, closed_at, row_count, is_current
             FROM split_catalog ORDER BY created_at DESC",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SplitInfo {
                id: row.get(0)?,
                file_path: row.get(1)?,
                created_at: row.get(2)?,
                closed_at: row.get(3)?,
                row_count: row.get(4)?,
                is_current: row.get::<_, i64>(5)? == 1,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_payload_keeps_from_to_reason_and_reinject() {
        let raw = switch_payload(&SwitchFacts {
            from_email: "alpha@gmail.com".to_string(),
            to_email: "beta@gmail.com".to_string(),
            reason: "Manual account switch".to_string(),
            how: "Closed the IDE, wrote the account, opened the IDE.".to_string(),
            prompt_id: "p1".to_string(),
            prompt_text: "keep going".to_string(),
            conversation_id: "conv-same".to_string(),
            prompt_reinjected: true,
            switch_ok: true,
            instance_id: "default".to_string(),
            ide_type: "antigravity".to_string(),
            idc_machine_alias: "node-1".to_string(),
            ide_path: "/usr/bin/antigravity".to_string(),
            switch_reason: "Manual account switch".to_string(),
            steps: None,
        });
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["from_email"], "alpha@gmail.com");
        assert_eq!(value["to_email"], "beta@gmail.com");
        assert_eq!(value["reason"], "Manual account switch");
        assert_eq!(value["prompt_text"], "keep going");
        assert_eq!(value["conversation_id"], "conv-same");
        assert_eq!(value["prompt_reinjected"], true);
        assert_eq!(value["instance_id"], "default");
        assert_eq!(value["ide_type"], "antigravity");
        assert_eq!(value["idc_machine_alias"], "node-1");
        assert_eq!(value["ide_path"], "/usr/bin/antigravity");
        assert_eq!(value["switch_reason"], "Manual account switch");
        assert_eq!(value.get("steps"), None);
        let (from_email, to_email) = payload_emails(Some(&raw));
        assert_eq!(from_email, "alpha@gmail.com");
        assert_eq!(to_email, "beta@gmail.com");
    }

    #[test]
    fn switch_payload_serializes_structured_audit_steps() {
        let raw = switch_payload(&SwitchFacts {
            from_email: "a@gmail.com".to_string(),
            to_email: "b@gmail.com".to_string(),
            reason: "Switch".to_string(),
            how: "Clean restart".to_string(),
            prompt_id: "".to_string(),
            prompt_text: "".to_string(),
            conversation_id: "".to_string(),
            prompt_reinjected: true,
            switch_ok: true,
            instance_id: "inst-1".to_string(),
            ide_type: "antigravity".to_string(),
            idc_machine_alias: "node-1".to_string(),
            ide_path: "/bin/antigravity".to_string(),
            switch_reason: "Switch".to_string(),
            steps: Some(SwitchAuditSteps {
                backup: Some(SwitchBackupStep {
                    prompt_count: 3,
                    project_names: vec!["proj-1".to_string()],
                    project_paths: vec!["/path/proj-1".to_string()],
                    backup_batch_id: "batch-123".to_string(),
                    success: true,
                }),
                reset: Some(SwitchResetStep {
                    terminated_pids: vec![1234, 5678],
                    auth_swapped: true,
                    credentials_injected: true,
                    success: true,
                }),
                restore: Some(SwitchRestoreStep {
                    method: "resume_task_json + prompt_channel_restore".to_string(),
                    restored_count: 3,
                    dispatched_count: 3,
                    prompt_channel_waited: true,
                    success: true,
                }),
                verification: Some(SwitchVerificationStep {
                    verified: true,
                    message: "Verified prompts restored and session active".to_string(),
                }),
            }),
        });
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert!(value.get("steps").is_some());
        assert_eq!(value["steps"]["backup"]["prompt_count"], 3);
        assert_eq!(value["steps"]["reset"]["terminated_pids"][0], 1234);
        assert_eq!(value["steps"]["restore"]["restored_count"], 3);
        assert_eq!(value["steps"]["verification"]["verified"], true);
    }
}
