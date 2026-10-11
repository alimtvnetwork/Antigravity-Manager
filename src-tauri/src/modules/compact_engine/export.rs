use super::db;
use super::types::*;
use super::workdir::{clean_path_str, to_relative_work_path, WorkDirResolver};
use crate::error::AppError;
use crate::modules::json_envelope::{JsonAttributes, JsonEnvelope, WorkDirectoryConfig};
use chrono::Utc;
use rusqlite::Connection;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Gathers active state from SQLite databases and builds a normalized `CompactExportData` snapshot.
pub fn gather_compact_data(
    work_dir: &Path,
    is_pinned_included: bool,
    is_relative: bool,
) -> Result<CompactExportData, AppError> {
    let mut pinned_projects = Vec::new();
    let mut conversations = Vec::new();
    let mut running_prompts = Vec::new();
    let mut prompt_history = Vec::new();
    let mut project_key_map = HashMap::<String, String>::new(); // path/id -> project_key

    // 1. Gather Pinned Projects (if included)
    if is_pinned_included {
        gather_pinned_projects(work_dir, is_relative, &mut pinned_projects, &mut project_key_map)?;
    }

    // 2. Gather Running Prompts (FIFO order)
    gather_running_prompts(work_dir, is_relative, &project_key_map, &mut running_prompts)?;

    // 3. Gather Conversations
    gather_conversations(work_dir, is_relative, &project_key_map, &mut conversations)?;

    // 4. Gather Prompt History (capped at 500 records)
    gather_prompt_history(&project_key_map, &mut prompt_history)?;

    // 5. Host machine metadata
    let source_machine = hostname_or_default();
    let meta = CompactMetadata {
        schema_version: "2.0".to_string(),
        created_at: Utc::now().to_rfc3339(),
        source_machine,
        source_work_dir: clean_path_str(work_dir),
        is_portable: is_relative,
        notes: Some(format!(
            "Exported {} pinned project(s), {} conversation(s), {} running prompt(s), {} history record(s)",
            pinned_projects.len(),
            conversations.len(),
            running_prompts.len(),
            prompt_history.len()
        )),
    };

    Ok(CompactExportData {
        meta,
        pinned_projects,
        conversations,
        running_prompts,
        prompt_history,
    })
}

/// Gathers pinned projects from `repo_prompts.db` sequences and fallback sources.
fn gather_pinned_projects(
    work_dir: &Path,
    is_relative: bool,
    out: &mut Vec<CompactPinnedProject>,
    key_map: &mut HashMap<String, String>,
) -> Result<(), AppError> {
    let mut seen_keys = HashSet::new();

    // Source A: `agm_project_sequences` in repo_prompts.db
    if let Ok(repo_db_path) = crate::modules::repo_db::get_repo_db_path() {
        if repo_db_path.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &repo_db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let sql = "SELECT project_key, seq_id, project_id, repo_name, repo_path, updated_at 
                           FROM agm_project_sequences ORDER BY seq_id ASC";
                if let Ok(mut stmt) = conn.prepare(sql) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, i64>(5)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (pkey, seq_id, pid, rname, rpath, updated_epoch) = item;
                            if seen_keys.contains(&pkey) {
                                continue;
                            }
                            seen_keys.insert(pkey.clone());

                            let rel_path = if is_relative {
                                to_relative_work_path(&PathBuf::from(&rpath), work_dir).unwrap_or(rpath.clone())
                            } else {
                                clean_path_str(&PathBuf::from(&rpath))
                            };

                            let pinned_at = chrono::DateTime::from_timestamp(updated_epoch, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| Utc::now().to_rfc3339());

                            key_map.insert(pid.clone(), pkey.clone());
                            key_map.insert(rpath.clone(), pkey.clone());
                            key_map.insert(clean_path_str(&PathBuf::from(&rpath)), pkey.clone());
                            key_map.insert(rname.clone(), pkey.clone());

                            out.push(CompactPinnedProject {
                                project_key: pkey,
                                repo_name: rname,
                                relative_path: rel_path,
                                pinned_at,
                                is_active: true,
                                display_order: seq_id as i32,
                                notes: None,
                            });
                        }
                    }
                }

                // Source B: running_projects in repo_prompts.db (if not already mapped)
                let sql_rp = "SELECT id, repo_name, repo_path, updated_at FROM running_projects WHERE is_running = 1";
                if let Ok(mut stmt) = conn.prepare(sql_rp) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (pid, rname, rpath, updated_epoch) = item;
                            let clean_rp = clean_path_str(&PathBuf::from(&rpath));
                            if key_map.contains_key(&pid) || key_map.contains_key(&clean_rp) {
                                continue;
                            }
                            let next_num = out.len() + 1;
                            let pkey = format!("P{:03}", next_num);
                            seen_keys.insert(pkey.clone());

                            let rel_path = if is_relative {
                                to_relative_work_path(&PathBuf::from(&rpath), work_dir).unwrap_or(rpath.clone())
                            } else {
                                clean_rp.clone()
                            };

                            let pinned_at = chrono::DateTime::from_timestamp(updated_epoch, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| Utc::now().to_rfc3339());

                            key_map.insert(pid, pkey.clone());
                            key_map.insert(clean_rp, pkey.clone());
                            key_map.insert(rname.clone(), pkey.clone());

                            out.push(CompactPinnedProject {
                                project_key: pkey,
                                repo_name: rname,
                                relative_path: rel_path,
                                pinned_at,
                                is_active: true,
                                display_order: next_num as i32,
                                notes: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Source C: GitMap AgyProjectSequence table in gitmap.db if empty
    if out.is_empty() {
        if let Some(gitmap_dir) = WorkDirResolver::query_gitmap_default_workdir() {
            // Find gitmap.db path
            if let Some(local_app_data) = dirs::data_local_dir() {
                let db_path = local_app_data.join("gitmap-cli").join("data").join("gitmap.db");
                if db_path.exists() {
                    if let Ok(conn) = Connection::open_with_flags(
                        &db_path,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                    ) {
                        let sql = "SELECT project_id, project_name, workspace_path, sequence_num, created_at 
                                   FROM AgyProjectSequence ORDER BY sequence_num ASC";
                        if let Ok(mut stmt) = conn.prepare(sql) {
                            if let Ok(rows) = stmt.query_map([], |row| {
                                Ok((
                                    row.get::<_, String>(0)?,
                                    row.get::<_, String>(1)?,
                                    row.get::<_, String>(2)?,
                                    row.get::<_, i64>(3)?,
                                    row.get::<_, String>(4)?,
                                ))
                            }) {
                                for item in rows.flatten() {
                                    let (pid, rname, rpath, seq_num, cat) = item;
                                    let pkey = format!("P{:03}", seq_num);
                                    if seen_keys.contains(&pkey) {
                                        continue;
                                    }
                                    seen_keys.insert(pkey.clone());

                                    let rel_path = if is_relative {
                                        to_relative_work_path(&PathBuf::from(&rpath), work_dir).unwrap_or(rpath.clone())
                                    } else {
                                        clean_path_str(&PathBuf::from(&rpath))
                                    };

                                    key_map.insert(pid.clone(), pkey.clone());
                                    key_map.insert(clean_path_str(&PathBuf::from(&rpath)), pkey.clone());
                                    key_map.insert(rname.clone(), pkey.clone());

                                    out.push(CompactPinnedProject {
                                        project_key: pkey,
                                        repo_name: rname,
                                        relative_path: rel_path,
                                        pinned_at: cat,
                                        is_active: true,
                                        display_order: seq_num as i32,
                                        notes: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            let _ = gitmap_dir;
        }
    }

    Ok(())
}

/// Gathers active and queued prompts adhering strictly to FIFO ordering (`ORDER BY created_at ASC, id ASC`).
fn gather_running_prompts(
    _work_dir: &Path,
    _is_relative: bool,
    key_map: &HashMap<String, String>,
    out: &mut Vec<CompactRunningPrompt>,
) -> Result<(), AppError> {
    let mut seen_ids = HashSet::new();

    // Source A: `active_prompts` in repo_prompts.db
    if let Ok(repo_db_path) = crate::modules::repo_db::get_repo_db_path() {
        if repo_db_path.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &repo_db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let sql = "SELECT id, project_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
                           FROM active_prompts ORDER BY created_at ASC, id ASC";
                if let Ok(mut stmt) = conn.prepare(sql) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, Option<String>>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, String>(6)?,
                            row.get::<_, i64>(7)?,
                            row.get::<_, i64>(8)?,
                            row.get::<_, Option<String>>(9)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (id, pid, rpath, content, model, sess_id, status, c_epoch, u_epoch, img) = item;
                            if seen_ids.contains(&id) {
                                continue;
                            }
                            seen_ids.insert(id.clone());

                            let pkey = key_map
                                .get(&pid)
                                .or_else(|| key_map.get(&clean_path_str(&PathBuf::from(&rpath))))
                                .cloned()
                                .unwrap_or_else(|| "P001".to_string());

                            let conv_id = sess_id.unwrap_or_else(|| "conv-default".to_string());
                            let created_at = chrono::DateTime::from_timestamp(c_epoch, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| Utc::now().to_rfc3339());
                            let updated_at = chrono::DateTime::from_timestamp(u_epoch, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| Utc::now().to_rfc3339());

                            out.push(CompactRunningPrompt {
                                prompt_id: id,
                                project_key: pkey,
                                conversation_id: conv_id,
                                prompt_content: content,
                                model,
                                status,
                                created_at,
                                updated_at,
                                image_payload: img,
                                is_restored: false,
                            });
                        }
                    }
                }
            }
        }
    }

    // Source B: `prompt_backups` in backup-prompts.db where is_restored = 0
    if let Ok(backup_db_path) = crate::modules::backup_prompts_db::get_backup_prompts_db_path(None) {
        if backup_db_path.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &backup_db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let sql = "SELECT id, prompt_id, project_name, project_path, conversation_id, prompt_text, created_at, images_payload, status 
                           FROM prompt_backups WHERE is_restored = 0 ORDER BY created_at ASC, id ASC";
                if let Ok(mut stmt) = conn.prepare(sql) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                            row.get::<_, i64>(6)?,
                            row.get::<_, Option<String>>(7)?,
                            row.get::<_, String>(8)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (id, pid, pname, ppath, conv_id, text, c_epoch, img, status) = item;
                            let prompt_id = if pid.is_empty() { id } else { pid };
                            if seen_ids.contains(&prompt_id) {
                                continue;
                            }
                            seen_ids.insert(prompt_id.clone());

                            let pkey = key_map
                                .get(&clean_path_str(&PathBuf::from(&ppath)))
                                .or_else(|| key_map.get(&pname))
                                .cloned()
                                .unwrap_or_else(|| "P001".to_string());

                            let created_at = chrono::DateTime::from_timestamp(c_epoch, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| Utc::now().to_rfc3339());

                            out.push(CompactRunningPrompt {
                                prompt_id,
                                project_key: pkey,
                                conversation_id: conv_id,
                                prompt_content: text,
                                model: None,
                                status,
                                created_at: created_at.clone(),
                                updated_at: created_at,
                                image_payload: img,
                                is_restored: false,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Gathers conversation threads across instances and sequences.
fn gather_conversations(
    work_dir: &Path,
    is_relative: bool,
    key_map: &HashMap<String, String>,
    out: &mut Vec<CompactConversation>,
) -> Result<(), AppError> {
    let mut seen_convs = HashSet::new();

    // Source A: `agm_conversation_sequences` in repo_prompts.db
    if let Ok(repo_db_path) = crate::modules::repo_db::get_repo_db_path() {
        if repo_db_path.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &repo_db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let sql = "SELECT conversation_id, seq_id, project_key, title, updated_at FROM agm_conversation_sequences ORDER BY seq_id ASC";
                if let Ok(mut stmt) = conn.prepare(sql) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, i64>(4)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (cid, seq_id, pkey, title, updated_epoch) = item;
                            if seen_convs.contains(&cid) {
                                continue;
                            }
                            seen_convs.insert(cid.clone());

                            let last_active_at = chrono::DateTime::from_timestamp(updated_epoch, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| Utc::now().to_rfc3339());

                            out.push(CompactConversation {
                                conversation_id: cid,
                                project_key: pkey,
                                seq_id,
                                title,
                                relative_path: ".".to_string(),
                                last_active_at,
                                message_count: 1,
                                is_archived: false,
                                has_active_prompt: false,
                            });
                        }
                    }
                }
            }
        }
    }

    // Source B: Candidate conversation_summaries.db files
    let tagged_dirs = crate::modules::repo_db::gemini_dirs_tagged(None);
    for (_inst_id, base_dir) in tagged_dirs {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }
        if let Ok(conn) = Connection::open_with_flags(
            &summaries_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        ) {
            let sql = "SELECT conversation_id, title, last_modified_time, status, not_fully_idle, workspace_uris 
                       FROM conversation_summaries ORDER BY last_modified_time DESC LIMIT 100";
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<i32>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                }) {
                    for item in rows.flatten() {
                        let (cid, title_opt, time_opt, status_opt, idle_opt, ws_opt) = item;
                        if seen_convs.contains(&cid) {
                            continue;
                        }
                        seen_convs.insert(cid.clone());

                        let title = title_opt.unwrap_or_else(|| format!("Conversation {}", cid));
                        let last_active_at = time_opt.unwrap_or_else(|| Utc::now().to_rfc3339());
                        let is_running = status_opt.map(|s| s.contains("RUNNING")).unwrap_or(false)
                            || idle_opt.map(|i| i > 0).unwrap_or(false);

                        // Resolve relative path from workspace_uris
                        let mut rel_path = ".".to_string();
                        let mut pkey = "P001".to_string();
                        if let Some(ws_raw) = ws_opt {
                            if let Ok(uris) = serde_json::from_str::<Vec<String>>(&ws_raw) {
                                if let Some(first_uri) = uris.first() {
                                    let raw_path = crate::modules::repo_db::failed_commands::decode_uri_to_path_pub(first_uri);
                                    let clean_p = clean_path_str(&PathBuf::from(&raw_path));
                                    if let Some(k) = key_map.get(&clean_p) {
                                        pkey = k.clone();
                                    }
                                    rel_path = if is_relative {
                                        to_relative_work_path(&PathBuf::from(&raw_path), work_dir).unwrap_or(raw_path)
                                    } else {
                                        clean_p
                                    };
                                }
                            }
                        }

                        let next_seq = out.len() as i64 + 1;
                        out.push(CompactConversation {
                            conversation_id: cid,
                            project_key: pkey,
                            seq_id: next_seq,
                            title,
                            relative_path: rel_path,
                            last_active_at,
                            message_count: 10,
                            is_archived: false,
                            has_active_prompt: is_running,
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

/// Gathers historical prompt records from `backup-prompts.db`.
fn gather_prompt_history(
    key_map: &HashMap<String, String>,
    out: &mut Vec<CompactPromptHistory>,
) -> Result<(), AppError> {
    if let Ok(backup_db_path) = crate::modules::backup_prompts_db::get_backup_prompts_db_path(None) {
        if backup_db_path.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &backup_db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let sql = "SELECT id, prompt_id, project_name, project_path, conversation_id, prompt_text, created_at, is_restored 
                           FROM prompt_backups ORDER BY created_at DESC LIMIT 500";
                if let Ok(mut stmt) = conn.prepare(sql) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5)?,
                            row.get::<_, i64>(6)?,
                            row.get::<_, i32>(7)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (id, _pid, pname, ppath, conv_id, text, c_epoch, is_restored) = item;
                            let pkey = key_map
                                .get(&clean_path_str(&PathBuf::from(&ppath)))
                                .or_else(|| key_map.get(&pname))
                                .cloned()
                                .unwrap_or_else(|| "P001".to_string());

                            let executed_at = chrono::DateTime::from_timestamp(c_epoch, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| Utc::now().to_rfc3339());

                            out.push(CompactPromptHistory {
                                history_id: id,
                                project_key: pkey,
                                conversation_id: conv_id,
                                prompt_content: text,
                                model: None,
                                role: "user".to_string(),
                                executed_at,
                                exit_code: Some(0),
                                has_completed: is_restored != 0,
                                duration_ms: 0,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Executes the full compaction export process driven by `ExportOptions`.
pub fn export_compact(options: &ExportOptions) -> Result<ExportResult, AppError> {
    let start_instant = std::time::Instant::now();
    let work_dir = WorkDirResolver::resolve_work_dir(options.work_dir.as_deref())?;

    let data = gather_compact_data(&work_dir, options.is_pinned_included, options.is_relative)?;

    let target_path_buf = if let Some(ref path_str) = options.file {
        PathBuf::from(path_str.trim())
    } else {
        let ts = Utc::now().format("%Y%m%d-%H%M%S");
        PathBuf::from(format!("agm-workspace-compact-{ts}.json"))
    };

    let is_sqlite = target_path_buf
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("db") || ext.eq_ignore_ascii_case("sqlite"))
        .unwrap_or(false);

    if is_sqlite {
        db::write_compact_db(&target_path_buf, &data)?;
    } else {
        export_to_json_envelope(&target_path_buf, &data, &work_dir)?;
    }

    let duration_ms = start_instant.elapsed().as_millis() as u64;

    Ok(ExportResult {
        output_path: clean_path_str(&target_path_buf),
        is_sqlite,
        pinned_count: data.pinned_projects.len(),
        conversations_count: data.conversations.len(),
        running_count: data.running_prompts.len(),
        history_count: data.prompt_history.len(),
        duration_ms,
    })
}

/// Serializes and writes a `CompactExportData` payload into a Two-Tier JSON Envelope v2.0 file.
pub fn export_to_json_envelope(
    path: &Path,
    data: &CompactExportData,
    work_dir: &Path,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Config(format!("Failed to create export folder {parent:?}: {e}")))?;
        }
    }

    let work_dir_clean = clean_path_str(work_dir);

    // Build variables
    let mut vars = HashMap::new();
    vars.insert("workDir".to_string(), Value::String(work_dir_clean.clone()));
    let first_repo = data
        .pinned_projects
        .first()
        .map(|p| p.repo_name.clone())
        .unwrap_or_else(|| "Antigravity-Manager".to_string());
    vars.insert(
        "repoDir".to_string(),
        Value::String(format!("${{workDir}}/{}", first_repo)),
    );

    // Build Two-Tier JSON Envelope v2.0
    let mut envelope = JsonEnvelope::new("agm/compact-export", data.clone())
        .with_variables(vars)
        .with_work_directory(WorkDirectoryConfig {
            path: "${workDir}".to_string(),
            default_path: Some(work_dir_clean.clone()),
            is_applied: true,
            is_enforced: false,
        })
        .with_description(
            "Portable workspace compaction snapshot including pinned projects, conversations, running prompts, and execution history."
        )
        .with_source("agm-cli");

    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("export.json");
    envelope.attributes.import_command = Some(format!("agm import-compact \"{}\" -y", filename));
    envelope.attributes.export_command = Some(format!("agm export-compact --file \"{}\"", filename));
    envelope.attributes.help_command = Some(format!("agm which-format \"{}\"", filename));

    let json_bytes = serde_json::to_string_pretty(&envelope)
        .map_err(|e| AppError::Config(format!("JSON serialization failed: {e}")))?;

    fs::write(path, json_bytes)
        .map_err(|e| AppError::Config(format!("Failed to write export file {path:?}: {e}")))?;

    Ok(())
}

/// Serializes and writes a `CompactExportData` payload into a standalone portable SQLite database file.
pub fn export_to_sqlite_file(path: &Path, data: &CompactExportData) -> Result<(), AppError> {
    db::write_compact_db(path, data)
}

fn hostname_or_default() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}
