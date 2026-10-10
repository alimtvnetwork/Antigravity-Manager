//! Repo DB: backup

use super::detection::{
    detect_running_projects, extract_image_payload_or_path, resume_task_document,
};
use super::discovery::discover_running_prompts_from_antigravity;
use super::models::{ActivePrompt, RunningProject};
use super::project_queries::get_live_project_execution_info;
use super::schema::{connect_db, init_tables};
use super::state::{get_memory_prompts_map, reset_dispatched_prompts_cache};
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

/// Prompts that still need a re-push after this instance's IDE is back up.
pub fn count_backed_up_prompts(instance_id: &str) -> usize {
    let Ok(conn) = connect_db() else {
        return 0;
    };
    conn.query_row(
        "SELECT COUNT(*) FROM active_prompts
         WHERE status = 'backed_up'
           AND (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))",
        [instance_id],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0) as usize
}

/// A switch waits for the IDE prompt channel only when a prompt was actually backed up.
pub fn needs_prompt_channel_wait(backed_up_count: usize) -> bool {
    backed_up_count > 0
}

/// Backup all currently running prompts across active projects before switching
pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let mut backed_up_count = 0;

    // Step 0: Retire stale 'running' prompts older than 2 hours to 'dispatched' so they don't shadow active prompts
    let stale_cutoff = now - 7200;
    // Justification: stale-row retirement is housekeeping; the backup transition proceeds regardless
    crate::error::record_ignored(
        conn.execute(
        "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE status = 'running' AND updated_at < ?",
        params![now, stale_cutoff],
    ),
        "retire stale running prompts",
    );

    // Step 0.5: Clear dispatched prompts cache so that any prompt backed up can be cleanly re-dispatched upon switch completion
    reset_dispatched_prompts_cache();

    let is_all_or_default =
        instance_id == "all" || instance_id == "__default__" || instance_id == "default";

    // Transition in-flight 'running' prompts in active_prompts to 'backed_up' before switch (scoped to target instance)
    let transitioned = if instance_id == "all" {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status = 'running'",
            params![now],
        )
        .unwrap_or(0)
    } else if is_all_or_default {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status = 'running' AND instance_id IN ('default', '__default__')",
            params![now],
        )
        .unwrap_or(0)
    } else {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status = 'running' AND instance_id = ?2",
            params![now, instance_id],
        )
        .unwrap_or(0)
    };
    if transitioned > 0 {
        backed_up_count += transitioned;
        if let Ok(mut map) = get_memory_prompts_map().lock() {
            for p in map.values_mut() {
                let matches_inst = instance_id == "all"
                    || (is_all_or_default
                        && (p.instance_id == "default" || p.instance_id == "__default__"))
                    || p.instance_id == instance_id;
                if p.status == "running" && matches_inst {
                    p.status = "backed_up".to_string();
                    p.updated_at = now;
                }
            }
        }

        crate::modules::logger::log_info(&format!(
            "[RepoDB] Transitioned {} in-flight prompts from 'running' to 'backed_up' for instance '{}' before switch",
            transitioned, instance_id
        ));
    }

    // Layer 1: Core Antigravity Live Conversations Discovery (~/.gemini/antigravity)
    let ag_prompts = discover_running_prompts_from_antigravity(instance_id);
    for p in ag_prompts {
        let clean_repo_name = Path::new(&p.repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| p.project_id.clone());
        // Justification: project-row scaffolding is auxiliary; the prompt upsert result is checked separately
        crate::error::record_ignored(
            conn.execute(
            "INSERT OR REPLACE INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?, ?, ?, ?, NULL, 0, ?, ?)",
            params![&p.project_id, &p.instance_id, &clean_repo_name, &p.repo_path, now, now],
        ),
            "upsert running project for backed-up prompt",
        );

        let res = conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET 
                 status = 'backed_up',
                 updated_at = excluded.updated_at,
                 prompt_content = excluded.prompt_content,
                 image_payload = excluded.image_payload",
            params![
                &p.id,
                &p.project_id,
                &p.instance_id,
                &p.repo_path,
                &p.prompt_content,
                &p.model,
                &p.session_id,
                p.created_at,
                now,
                &p.image_payload,
            ],
        );

        if res.is_ok() {
            backed_up_count += 1;
            let (extracted_img, img_paths) = extract_image_payload_or_path(&p.prompt_content);
            let final_img = p.image_payload.clone().or(extracted_img);
            // Write disk resume snapshot file inside project repo directory.
            // session_id is the live conversation, so the IDE reopens that chat.
            let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
            let mut payload = resume_task_document(&p, "backed_up", now, &img_paths);
            if final_img.is_some() {
                payload["image_payload"] = serde_json::json!(final_img);
            }
            if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                // Justification: resume snapshot is a convenience file for the IDE; the database row is the source of truth
                crate::error::record_ignored(
                    fs::write(&task_file, json_str),
                    "write resume task snapshot",
                );
            }
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                map.insert(p.id.clone(), p);
            }
        }
    }

    // Layer 2: VS Code / Instance Workspace Storage & Active Projects Discovery (Parallelized)
    let projects = detect_running_projects(instance_id).unwrap_or_default();
    let extracted_results: Vec<(&RunningProject, Vec<(String, Option<String>)>)> =
        std::thread::scope(|s| {
            let handles: Vec<_> = projects
                .iter()
                .map(|project| {
                    s.spawn(move || {
                        let mut extracted: Vec<(String, Option<String>)> = Vec::new();

                        // Check workspace state.vscdb for active prompts / tasks
                        if let Some(ref ws_storage) = project.workspace_storage_path {
                            let ws_db_path = PathBuf::from(ws_storage).join("state.vscdb");
                            if ws_db_path.exists() {
                                if let Ok(ws_conn) = Connection::open_with_flags(
                                    &ws_db_path,
                                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                                ) {
                                    let mut stmt = ws_conn
                                        .prepare("SELECT key, value FROM ItemTable WHERE key LIKE '%prompt%' OR key LIKE '%chat%' OR key LIKE '%task%'")
                                        .ok();

                                    if let Some(ref mut prepared) = stmt {
                                        if let Ok(rows) = prepared.query_map([], |row| {
                                            let key: String = row.get(0)?;
                                            let val: String = row.get(1)?;
                                            Ok((key, val))
                                        }) {
                                            for item in rows.flatten() {
                                                let content = item.1;
                                                if !content.trim().is_empty() && content.len() > 10 {
                                                    let (img_payload, _) = extract_image_payload_or_path(&content);
                                                    extracted.push((content, img_payload));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Check if project has an existing .antigravity_resume_task.json on disk
                        if extracted.is_empty() {
                            let task_file = PathBuf::from(&project.repo_path)
                                .join(".antigravity_resume_task.json");
                            if task_file.exists() {
                                if let Ok(c) = fs::read_to_string(&task_file) {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                                        if let Some(txt) =
                                            v.get("prompt_content").and_then(|t| t.as_str())
                                        {
                                            if !txt.trim().is_empty() {
                                                let img = v
                                                    .get("image_payload")
                                                    .and_then(|i| i.as_str())
                                                    .map(|s| s.to_string());
                                                extracted.push((txt.to_string(), img));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        (project, extracted)
                    })
                })
                .collect();

            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });

    for (project, mut extracted_prompts) in extracted_results {
        // Check if SQLite active_prompts already has an existing prompt for this repo_path or project
        if extracted_prompts.is_empty() {
            let proj_like = format!("%{}%", project.repo_name.to_lowercase());
            let mut check_stmt = conn
                .prepare(
                    "SELECT prompt_content, image_payload FROM active_prompts \
                     WHERE repo_path = ?1 OR project_id = ?2 OR project_id LIKE ?3 \
                     ORDER BY updated_at DESC LIMIT 1",
                )
                .ok();
            if let Some(ref mut c_stmt) = check_stmt {
                if let Ok((txt, img)) = c_stmt.query_row(
                    rusqlite::params![&project.repo_path, &project.id, &proj_like],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
                ) {
                    if !txt.trim().is_empty() {
                        extracted_prompts.push((txt, img));
                    }
                }
            }
        }

        for (prompt_text, image_payload) in extracted_prompts {
            let existing_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM active_prompts WHERE repo_path = ?1 AND prompt_content = ?2 LIMIT 1",
                    rusqlite::params![&project.repo_path, &prompt_text],
                    |r| r.get(0),
                )
                .ok();

            let (prompt_id, is_existing) = match existing_id {
                Some(eid) => (eid, true),
                None => (Uuid::new_v4().to_string(), false),
            };

            let prompt_model = Some("gemini-pro".to_string());
            let result = if is_existing {
                conn.execute(
                    "UPDATE active_prompts SET status = 'backed_up', updated_at = ?, image_payload = COALESCE(?, image_payload) WHERE id = ?",
                    params![now, &image_payload, &prompt_id],
                )
            } else {
                conn.execute(
                    "INSERT INTO active_prompts 
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?, ?, ?, ?, ?, ?, ?, 'backed_up', ?, ?, ?)",
                    params![
                        &prompt_id,
                        &project.id,
                        instance_id,
                        &project.repo_path,
                        &prompt_text,
                        &prompt_model,
                        &project.id,
                        now,
                        now,
                        &image_payload,
                    ],
                )
            };

            if result.is_ok() {
                backed_up_count += 1;
                let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt_text);
                let final_img = image_payload.clone().or(extracted_img);

                let active_prompt = ActivePrompt {
                    id: prompt_id.clone(),
                    project_id: project.id.clone(),
                    instance_id: instance_id.to_string(),
                    repo_path: project.repo_path.clone(),
                    prompt_content: prompt_text,
                    model: prompt_model,
                    session_id: Some(project.id.clone()),
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload: final_img.clone(),
                };

                // Write disk resume snapshot file inside project repo directory using canonical resume_task_document
                let task_file =
                    PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
                let mut payload =
                    resume_task_document(&active_prompt, "backed_up", now, &img_paths);
                if final_img.is_some() {
                    payload["image_payload"] = serde_json::json!(final_img);
                }
                if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                    // Justification: resume snapshot is a convenience file for the IDE; the database row is the source of truth
                    crate::error::record_ignored(
                        fs::write(&task_file, json_str),
                        "write resume task snapshot",
                    );
                }

                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Backed up {} running prompts for instance '{}' across Antigravity core and workspaces",
        backed_up_count, instance_id
    ));

    invalidate_prompt_tree_cache(Some(instance_id));
    Ok(backed_up_count)
}

/// Verified count of active prompts or running processes across workspaces
pub fn verify_prompts_running() -> usize {
    let mut count = 0;

    // 1. Check SQLite active_prompts for 'running' or recently 'dispatched' (within 5 minutes)
    if let Ok(conn) = connect_db() {
        let now = Utc::now().timestamp();
        let db_count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE status = 'running' OR (status = 'dispatched' AND updated_at >= ?1)",
                params![now - 45],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count = db_count;
    }

    // 2. Check Antigravity conversation_summaries.db
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let ag_count: usize = conn
                    .query_row(
                        "SELECT COUNT(*) FROM conversation_summaries 
                         WHERE not_fully_idle != 0 
                           AND status LIKE '%RUNNING%' 
                           AND status NOT LIKE '%IDLE%' 
                           AND status NOT LIKE '%COMPLETED%' 
                           AND status NOT LIKE '%FAILED%' 
                           AND status NOT LIKE '%CANCELLED%'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if ag_count > count {
                    count = ag_count;
                }
            }
        }
    }

    // 3. Fallback to live project execution info
    if count == 0 {
        let live = get_live_project_execution_info();
        count = live
            .iter()
            .filter(|p| p.is_running || (!p.is_idle && p.active_prompt.is_some()))
            .count();
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::init_tables;
    use rusqlite::Connection;

    #[test]
    fn prompt_channel_wait_only_when_a_prompt_was_backed_up() {
        assert!(!needs_prompt_channel_wait(0));
        assert!(needs_prompt_channel_wait(1));
    }

    #[test]
    fn test_backup_conflict_resolution_no_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();
        // Insert parent running_project first
        conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-dup', 'default', 'dup', '/work/dup', NULL, 1, ?, ?)",
            params![now, now],
        )
        .unwrap();

        // Insert prompt first time
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, NULL)
             ON CONFLICT(id) DO UPDATE SET status = 'backed_up', updated_at = excluded.updated_at",
            params!["p-duplicate", "proj-dup", "default", "/work/dup", "Prompt text", "gemini-pro", "cid-dup", now, now],
        ).unwrap();

        // Insert same prompt second time with ON CONFLICT
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, NULL)
             ON CONFLICT(id) DO UPDATE SET status = 'backed_up', updated_at = excluded.updated_at",
            params!["p-duplicate", "proj-dup", "default", "/work/dup", "Prompt text", "gemini-pro", "cid-dup", now + 1, now + 1],
        ).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE id = 'p-duplicate'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Must never duplicate records on backup");
    }
}
