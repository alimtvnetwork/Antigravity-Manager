use crate::modules::agy_cleaner;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use super::*;
use crate::modules::repo_db;

/// Backward compatible wrapper with instance support
pub fn backup_active_running_prompts(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    backup_active_running_prompts_for_instance(instance_id, custom_file)
}

/// Backup all currently active and queued running prompts into the split SQLite database scoped to an instance
pub fn backup_active_running_prompts_for_instance(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    let target_inst = instance_id.unwrap_or("default");
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        auto_cleanup_expired(custom_file, 86400),
        "auto_cleanup_expired",
    );

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = repo_db::discover_running_prompts_from_antigravity(target_inst);
    let conversations = agy_cleaner::scan_conversations(100);
    let now = Utc::now().timestamp();
    let freshness_cutoff = now - 7200;

    // Consolidate candidate prompts to back up: prioritize live discovered prompts
    let mut candidate_prompts = Vec::new();
    for p in running_prompts.clone() {
        candidate_prompts.push(p);
    }
    for p in all_prompts {
        let is_inst_match = p.instance_id.is_empty()
            || p.instance_id == target_inst
            || (target_inst == "default"
                && (p.instance_id == "default" || p.instance_id.is_empty()));
        // RC-5: queued rows are backup candidates at any age — the 2h
        // freshness window silently dropped long-queued prompts before a
        // switch. Running / backed_up rows keep the freshness filter to
        // bound stale-row backup load.
        let is_fresh_enough = p.status == "queued" || p.updated_at >= freshness_cutoff;
        if is_inst_match
            && (p.status == "running" || p.status == "queued" || p.status == "backed_up")
            && is_fresh_enough
            && !candidate_prompts.iter().any(|c| {
                c.id == p.id
                    || (c.repo_path.to_lowercase().replace('\\', "/")
                        == p.repo_path.to_lowercase().replace('\\', "/")
                        && c.prompt_content.trim() == p.prompt_content.trim())
            })
        {
            candidate_prompts.push(p);
        }
    }

    let conn = connect_backup_db(custom_file)?;
    let batch_id = format!("batch_{}", Uuid::new_v4().simple());

    let db_path_str = get_backup_prompts_db_path(custom_file)?
        .to_string_lossy()
        .to_string();

    let batch_info = BackupBatchInfo {
        id: batch_id.clone(),
        created_at: now,
        prompts_count: candidate_prompts.len(),
        file_path: db_path_str.clone(),
        retention_days: 1,
        is_fully_restored: false,
    };

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            batch_info.id,
            batch_info.created_at,
            batch_info.prompts_count,
            batch_info.file_path,
            batch_info.retention_days,
            batch_info.is_fully_restored,
        ],
    )
    .map_err(|e| format!("Failed to record backup batch: {}", e))?;

    let mut records = Vec::new();
    for (idx, p) in candidate_prompts.iter().enumerate() {
        let repo_norm = p.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || uris_norm.contains(&p.project_id.to_lowercase())
        });

        // Preserve the prompt's ORIGINAL session_id verbatim so restore and
        // re-send re-attach to the same conversation. The live matched
        // conversation is only a fallback when the prompt carries no session.
        let conv_id = p
            .session_id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| matched_conv.map(|c| c.conversation_id.clone()))
            .unwrap_or_else(|| "-".to_string());

        let conv_name = matched_conv.and_then(|c| {
            if c.title.trim().is_empty() {
                None
            } else {
                Some(c.title.clone())
            }
        });

        let proj_name = resolve_friendly_project_name(&p.repo_path, &p.project_id);

        // Extract ALL data URIs from content when the live row carries no
        // payload yet, so no image is silently dropped (AC2).
        let (backup_uris, _) =
            crate::modules::repo_db::detection::extract_image_uris_or_paths(&p.prompt_content);
        let mut images_payload = p
            .image_payload
            .clone()
            .or_else(|| crate::modules::repo_db::detection::serialize_image_uris(&backup_uris));
        // Preserve upstream screenshot-id token tagging (single-payload form
        // only; JSON-array multi-URI payloads keep their own structure so the
        // token append cannot corrupt the JSON).
        if let Some(ss_token) = repo_db::extract_screenshot_id_token(&p.prompt_content) {
            let needs_token = images_payload
                .as_ref()
                .map(|cur| !cur.contains(&ss_token) && !cur.starts_with('['))
                .unwrap_or(true);
            if needs_token {
                images_payload = Some(match images_payload {
                    Some(cur) => format!("{};screenshot_id={}", cur, ss_token),
                    None => format!("screenshot_id={}", ss_token),
                });
            }
        }
        let has_images = images_payload.is_some();

        let record = PromptBackupRecord {
            id: format!("rec_{}", Uuid::new_v4().simple()),
            backup_batch_id: batch_id.clone(),
            prompt_id: p.id.clone(),
            project_name: proj_name,
            project_path: p.repo_path.clone(),
            project_id: p.project_id.clone(),
            conversation_id: conv_id,
            conversation_name: conv_name,
            sequence_id: (idx + 1) as i64,
            prompt_text: p.prompt_content.clone(),
            has_images,
            images_payload,
            status: p.status.clone(),
            created_at: now,
            is_restored: false,
            restored_at: None,
            instance_id: Some(target_inst.to_string()),
        };

        // Deduplicate records to prevent duplicate reinjections into prompt_backups
        let is_live_prompt = running_prompts.iter().any(|rp| rp.id == record.prompt_id);
        let existing_unrestored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND is_restored = 0 LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_unrestored {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, is_restored = 0, has_images = ?4, images_payload = ?5, prompt_text = ?6 WHERE id = ?7",
                params![record.backup_batch_id, now, target_inst, record.has_images, record.images_payload, record.prompt_text, existing_rec_id],
            ), "db execute");
            let payload_bytes = record.images_payload.as_ref().map(|s| s.len()).unwrap_or(0);
            crate::modules::logger::log_info(&format!(
                "[PROMPT_BACKUP] Backed up prompt {} for instance '{}' (has_images: {}, payload_bytes: {})",
                record.prompt_id,
                record.instance_id.as_deref().unwrap_or(target_inst),
                record.has_images,
                payload_bytes
            ));
            records.push(record);
            continue;
        }

        // If previously restored for this instance, reactivate for this new switch batch
        let existing_restored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND (instance_id = ?4 OR instance_id IS NULL OR instance_id = '') LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text, target_inst],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_restored {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, is_restored = 0, restored_at = NULL, has_images = ?4, images_payload = ?5, prompt_text = ?6 WHERE id = ?7",
                params![record.backup_batch_id, now, target_inst, record.has_images, record.images_payload, record.prompt_text, existing_rec_id],
            ), "db execute");
            let payload_bytes = record.images_payload.as_ref().map(|s| s.len()).unwrap_or(0);
            crate::modules::logger::log_info(&format!(
                "[PROMPT_BACKUP] Backed up prompt {} for instance '{}' (has_images: {}, payload_bytes: {})",
                record.prompt_id,
                record.instance_id.as_deref().unwrap_or(target_inst),
                record.has_images,
                payload_bytes
            ));
            records.push(record);
            continue;
        }

        conn.execute(
            "INSERT INTO prompt_backups (
                id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at, instance_id
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                record.id,
                record.backup_batch_id,
                record.prompt_id,
                record.project_name,
                record.project_path,
                record.project_id,
                record.conversation_id,
                record.conversation_name,
                record.sequence_id,
                record.prompt_text,
                record.has_images,
                record.images_payload,
                record.status,
                record.created_at,
                record.is_restored,
                record.restored_at,
                record.instance_id.as_deref().unwrap_or(target_inst),
            ],
        )
        .map_err(|e| format!("Failed to insert prompt backup record: {}", e))?;

        let payload_bytes = record.images_payload.as_ref().map(|s| s.len()).unwrap_or(0);
        crate::modules::logger::log_info(&format!(
            "[PROMPT_BACKUP] Backed up prompt {} for instance '{}' (has_images: {}, payload_bytes: {})",
            record.prompt_id,
            record.instance_id.as_deref().unwrap_or(target_inst),
            record.has_images,
            payload_bytes
        ));

        records.push(record);
    }

    let app_config = crate::modules::config::load_app_config().unwrap_or_default();
    let cap = if app_config.max_backup_prompts > 0 {
        app_config.max_backup_prompts
    } else if app_config.auto_profile_switcher.max_backup_prompts > 0 {
        app_config.auto_profile_switcher.max_backup_prompts as usize
    } else {
        100
    };
    // Justification: retention cleanup failure is non-fatal; logged and retried next backup
    crate::error::record_ignored(
        enforce_max_backup_prompts_cap(&conn, cap),
        "enforce_max_backup_prompts_cap",
    );

    Ok((batch_info, records))
}

/// Backup ONLY currently queued prompts into the split SQLite database,
/// preserving strict FIFO enqueue order.
///
/// Queue rows are read `ORDER BY created_at ASC, id ASC` (the house FIFO
/// invariant) and recorded with `sequence_id` matching their FIFO position,
/// so a later re-send replays them in the exact order they were enqueued.
/// Each record captures: prompt text, destination (project / instance /
/// conversation), and image/attachment references (`has_images`,
/// `images_payload`).
pub fn backup_queued_prompts_for_instance(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    let target_inst = instance_id.unwrap_or("default");
    let unscoped = instance_id.is_none();
    // Justification: best-effort retention cleanup; failure logged, backup proceeds
    crate::error::record_ignored(
        auto_cleanup_expired(custom_file, 86400),
        "auto_cleanup_expired",
    );

    // FIFO read of the live queue: earliest-enqueued first.
    let repo_conn = repo_db::connect_db().map_err(|e| format!("Failed to open repo DB: {}", e))?;
    let is_default_scope = target_inst == "default" || target_inst == "__default__";
    let queued_sql = if unscoped {
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
         FROM active_prompts WHERE status = 'queued' ORDER BY created_at ASC, id ASC"
    } else if is_default_scope {
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
         FROM active_prompts WHERE status = 'queued'
           AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL)
         ORDER BY created_at ASC, id ASC"
    } else {
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
         FROM active_prompts WHERE status = 'queued' AND instance_id = ?1
         ORDER BY created_at ASC, id ASC"
    };
    let mut qstmt = repo_conn
        .prepare(queued_sql)
        .map_err(|e| format!("Failed to prepare queued prompts query: {}", e))?;
    let map_row = |row: &rusqlite::Row| {
        Ok(repo_db::ActivePrompt {
            id: row.get(0)?,
            project_id: row.get(1)?,
            instance_id: row.get(2)?,
            repo_path: row.get(3)?,
            prompt_content: row.get(4)?,
            model: row.get(5)?,
            session_id: row.get(6)?,
            status: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            image_payload: row.get(10).ok(),
        })
    };
    let queued_prompts: Vec<repo_db::ActivePrompt> = if unscoped || is_default_scope {
        qstmt
            .query_map([], map_row)
            .map_err(|e| format!("Failed to query queued prompts: {}", e))?
            .flatten()
            .collect()
    } else {
        qstmt
            .query_map(params![target_inst], map_row)
            .map_err(|e| format!("Failed to query queued prompts: {}", e))?
            .flatten()
            .collect()
    };

    let conversations = agy_cleaner::scan_conversations(100);
    let now = Utc::now().timestamp();

    let conn = connect_backup_db(custom_file)?;
    let batch_id = format!("batch_{}", Uuid::new_v4().simple());

    let db_path_str = get_backup_prompts_db_path(custom_file)?
        .to_string_lossy()
        .to_string();

    let batch_info = BackupBatchInfo {
        id: batch_id.clone(),
        created_at: now,
        prompts_count: queued_prompts.len(),
        file_path: db_path_str.clone(),
        retention_days: 1,
        is_fully_restored: false,
    };

    conn.execute(
        "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            batch_info.id,
            batch_info.created_at,
            batch_info.prompts_count,
            batch_info.file_path,
            batch_info.retention_days,
            batch_info.is_fully_restored,
        ],
    )
    .map_err(|e| format!("Failed to record backup batch: {}", e))?;

    let mut records = Vec::new();
    for (idx, p) in queued_prompts.iter().enumerate() {
        let repo_norm = p.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || uris_norm.contains(&p.project_id.to_lowercase())
        });

        // Preserve the prompt's ORIGINAL session_id verbatim so restore and
        // re-send re-attach to the same conversation. The live matched
        // conversation is only a fallback when the prompt carries no session.
        let conv_id = p
            .session_id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| matched_conv.map(|c| c.conversation_id.clone()))
            .unwrap_or_else(|| "-".to_string());

        let conv_name = matched_conv.and_then(|c| {
            if c.title.trim().is_empty() {
                None
            } else {
                Some(c.title.clone())
            }
        });

        let proj_name = resolve_friendly_project_name(&p.repo_path, &p.project_id);
        // Preserve the prompt's own instance on unscoped backups so a later
        // per-instance re-send dispatches under the right instance.
        let eff_inst = if p.instance_id.trim().is_empty() {
            target_inst.to_string()
        } else {
            p.instance_id.clone()
        };

        // Fresh enqueues store image_payload: None on the live row; extract
        // ALL data URIs from content so no image is silently dropped (AC2).
        let (backup_uris, _) =
            crate::modules::repo_db::detection::extract_image_uris_or_paths(&p.prompt_content);
        let mut images_payload = p
            .image_payload
            .clone()
            .or_else(|| crate::modules::repo_db::detection::serialize_image_uris(&backup_uris));
        // Preserve upstream screenshot-id token tagging (single-payload form
        // only; JSON-array multi-URI payloads keep their own structure so the
        // token append cannot corrupt the JSON).
        if let Some(ss_token) = repo_db::extract_screenshot_id_token(&p.prompt_content) {
            let needs_token = images_payload
                .as_ref()
                .map(|cur| !cur.contains(&ss_token) && !cur.starts_with('['))
                .unwrap_or(true);
            if needs_token {
                images_payload = Some(match images_payload {
                    Some(cur) => format!("{};screenshot_id={}", cur, ss_token),
                    None => format!("screenshot_id={}", ss_token),
                });
            }
        }
        let has_images = images_payload.is_some();

        let record = PromptBackupRecord {
            id: format!("rec_{}", Uuid::new_v4().simple()),
            backup_batch_id: batch_id.clone(),
            prompt_id: p.id.clone(),
            project_name: proj_name,
            project_path: p.repo_path.clone(),
            project_id: p.project_id.clone(),
            conversation_id: conv_id,
            conversation_name: conv_name,
            // FIFO position in the live queue at backup time.
            sequence_id: (idx + 1) as i64,
            prompt_text: p.prompt_content.clone(),
            has_images,
            images_payload,
            status: p.status.clone(),
            created_at: now,
            is_restored: false,
            restored_at: None,
            instance_id: Some(eff_inst.clone()),
        };

        // Deduplicate records to prevent duplicate reinjections into prompt_backups
        let existing_unrestored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND is_restored = 0 LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_unrestored {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, sequence_id = ?4, is_restored = 0, has_images = ?5, images_payload = ?6, prompt_text = ?7 WHERE id = ?8",
                params![record.backup_batch_id, now, eff_inst, record.sequence_id, record.has_images, record.images_payload, record.prompt_text, existing_rec_id],
            ), "db execute");
            let payload_bytes = record.images_payload.as_ref().map(|s| s.len()).unwrap_or(0);
            crate::modules::logger::log_info(&format!(
                "[PROMPT_BACKUP] Backed up prompt {} for instance '{}' (has_images: {}, payload_bytes: {})",
                record.prompt_id,
                record.instance_id.as_deref().unwrap_or(target_inst),
                record.has_images,
                payload_bytes
            ));
            records.push(record);
            continue;
        }

        // If previously restored for this instance, reactivate for this new queue snapshot
        let existing_restored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND (instance_id = ?4 OR instance_id IS NULL OR instance_id = '') LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text, eff_inst],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_restored {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, sequence_id = ?4, is_restored = 0, restored_at = NULL, has_images = ?5, images_payload = ?6, prompt_text = ?7 WHERE id = ?8",
                params![record.backup_batch_id, now, eff_inst, record.sequence_id, record.has_images, record.images_payload, record.prompt_text, existing_rec_id],
            ), "db execute");
            let payload_bytes = record.images_payload.as_ref().map(|s| s.len()).unwrap_or(0);
            crate::modules::logger::log_info(&format!(
                "[PROMPT_BACKUP] Backed up prompt {} for instance '{}' (has_images: {}, payload_bytes: {})",
                record.prompt_id,
                record.instance_id.as_deref().unwrap_or(target_inst),
                record.has_images,
                payload_bytes
            ));
            records.push(record);
            continue;
        }

        conn.execute(
            "INSERT INTO prompt_backups (
                id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at, instance_id
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                record.id,
                record.backup_batch_id,
                record.prompt_id,
                record.project_name,
                record.project_path,
                record.project_id,
                record.conversation_id,
                record.conversation_name,
                record.sequence_id,
                record.prompt_text,
                record.has_images,
                record.images_payload,
                record.status,
                record.created_at,
                record.is_restored,
                record.restored_at,
                record.instance_id.as_deref().unwrap_or(target_inst),
            ],
        )
        .map_err(|e| format!("Failed to insert queued prompt backup record: {}", e))?;

        let payload_bytes = record.images_payload.as_ref().map(|s| s.len()).unwrap_or(0);
        crate::modules::logger::log_info(&format!(
            "[PROMPT_BACKUP] Backed up prompt {} for instance '{}' (has_images: {}, payload_bytes: {})",
            record.prompt_id,
            record.instance_id.as_deref().unwrap_or(target_inst),
            record.has_images,
            payload_bytes
        ));

        records.push(record);
    }

    let app_config = crate::modules::config::load_app_config().unwrap_or_default();
    let cap = if app_config.max_backup_prompts > 0 {
        app_config.max_backup_prompts
    } else if app_config.auto_profile_switcher.max_backup_prompts > 0 {
        app_config.auto_profile_switcher.max_backup_prompts as usize
    } else {
        100
    };
    // Justification: retention cleanup failure is non-fatal; logged and retried next backup
    crate::error::record_ignored(
        enforce_max_backup_prompts_cap(&conn, cap),
        "enforce_max_backup_prompts_cap",
    );

    Ok((batch_info, records))
}

/// Enforce FIFO retention cap on restored prompt_backups rows.
/// RC-5: unrestored rows (`is_restored = 0`) are NEVER pruned — they are
/// the only durable copy of prompts still owed a restore. The cap applies
/// to restored rows only, keeping the newest `max_cap` of them.
pub fn enforce_max_backup_prompts_cap(conn: &Connection, max_cap: usize) -> Result<usize, String> {
    if max_cap == 0 {
        return Ok(0);
    }

    let restored_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if restored_count <= max_cap {
        return Ok(0);
    }

    let deleted = conn
        .execute(
            "DELETE FROM prompt_backups 
             WHERE is_restored = 1
               AND id NOT IN (
                 SELECT id FROM prompt_backups 
                 WHERE is_restored = 1
                 ORDER BY created_at DESC, id DESC 
                 LIMIT ?1
             )",
            params![max_cap as i64],
        )
        .map_err(|e| format!("Failed to prune prompt backups to cap {}: {}", max_cap, e))?;

    // Prune orphaned backup_batches
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "DELETE FROM backup_batches 
             WHERE id NOT IN (SELECT DISTINCT backup_batch_id FROM prompt_backups)",
            [],
        ),
        "prune orphaned backup_batches",
    );

    crate::modules::logger::log_info(&format!(
        "[BackupPrompts] Enforced max cap {}: pruned {} older restored backup records (restored total was {})",
        max_cap, deleted, restored_count
    ));

    Ok(deleted)
}
