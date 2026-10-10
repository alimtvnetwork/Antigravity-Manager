use crate::modules::audit_action::{resolve_action, AuditAction};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

use super::*;

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
