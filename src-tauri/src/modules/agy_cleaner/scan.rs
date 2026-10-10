use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::*;

/// Scan all conversations across all candidate base directories and sort by recency (newest first)
pub fn scan_conversations(keep_count: usize) -> Vec<ConversationItem> {
    let candidate_dirs = get_gemini_candidate_dirs();
    let mut conversations = Vec::new();
    let mut seen_cids = std::collections::HashSet::new();
    let mut active_db_cids = std::collections::HashSet::new();

    for base_dir in &candidate_dirs {
        let conv_dir = base_dir.join("conversations");
        if !conv_dir.is_dir() {
            continue;
        }

        // Load indexed metadata from conversation_summaries.db if present in this base_dir
        let summaries_db_path = base_dir.join("conversation_summaries.db");
        let mut summaries_map: std::collections::HashMap<
            String,
            (String, String, i64, String, String),
        > = std::collections::HashMap::new();

        if summaries_db_path.is_file() {
            if let Ok(conn) = Connection::open_with_flags(
                &summaries_db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ) {
                let query_extended = "SELECT conversation_id, title, preview, step_count, workspace_uris, last_modified_time, COALESCE(status, ''), COALESCE(not_fully_idle, 0) FROM conversation_summaries";
                if let Ok(mut stmt) = conn.prepare(query_extended) {
                    let rows = stmt.query_map([], |row| {
                        let cid: String = row.get(0)?;
                        let title: String = row.get(1).unwrap_or_default();
                        let preview: String = row.get(2).unwrap_or_default();
                        let step_count: i64 = row.get(3).unwrap_or(0);
                        let uris: String = row.get(4).unwrap_or_default();
                        let last_mod: String = row.get(5).unwrap_or_default();
                        let status: String = row.get(6).unwrap_or_default();
                        let not_fully_idle: i32 = row.get(7).unwrap_or(0);
                        Ok((
                            cid,
                            title,
                            preview,
                            step_count,
                            uris,
                            last_mod,
                            status,
                            not_fully_idle,
                        ))
                    });

                    if let Ok(items) = rows {
                        for item in items.flatten() {
                            if item.6.to_uppercase().contains("RUNNING") || item.7 != 0 {
                                active_db_cids.insert(item.0.clone());
                            }
                            summaries_map.insert(item.0, (item.1, item.2, item.3, item.4, item.5));
                        }
                    }
                } else {
                    let query = "SELECT conversation_id, title, preview, step_count, workspace_uris, last_modified_time FROM conversation_summaries";
                    if let Ok(mut stmt) = conn.prepare(query) {
                        let rows = stmt.query_map([], |row| {
                            let cid: String = row.get(0)?;
                            let title: String = row.get(1).unwrap_or_default();
                            let preview: String = row.get(2).unwrap_or_default();
                            let step_count: i64 = row.get(3).unwrap_or(0);
                            let uris: String = row.get(4).unwrap_or_default();
                            let last_mod: String = row.get(5).unwrap_or_default();
                            Ok((cid, title, preview, step_count, uris, last_mod))
                        });

                        if let Ok(items) = rows {
                            for item in items.flatten() {
                                summaries_map
                                    .insert(item.0, (item.1, item.2, item.3, item.4, item.5));
                            }
                        }
                    }
                }
            }
        }

        if let Ok(entries) = fs::read_dir(&conv_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_db = path.extension().and_then(|s| s.to_str()) == Some("db");
                if !is_db {
                    continue;
                }

                let cid = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(stem) => stem.to_string(),
                    None => continue,
                };

                if !seen_cids.insert(cid.clone()) {
                    continue;
                }

                let metadata = fs::metadata(&path).ok();
                let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let mtime_secs = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let (title, preview, step_count, workspace_uris, last_mod) =
                    if let Some(entry_meta) = summaries_map.get(&cid) {
                        (
                            entry_meta.0.clone(),
                            entry_meta.1.clone(),
                            entry_meta.2,
                            entry_meta.3.clone(),
                            entry_meta.4.clone(),
                        )
                    } else {
                        (
                            String::new(),
                            String::new(),
                            0,
                            String::new(),
                            mtime_secs.to_string(),
                        )
                    };

                conversations.push(ConversationItem {
                    conversation_id: cid,
                    title,
                    preview,
                    step_count,
                    workspace_uris,
                    last_modified_time: last_mod,
                    db_path: path.to_string_lossy().to_string(),
                    file_size,
                    is_preserved: false,
                });
            }
        }
    }

    // Sort descending by recency
    conversations.sort_by(|a, b| {
        b.last_modified_time
            .cmp(&a.last_modified_time)
            .then_with(|| b.file_size.cmp(&a.file_size))
    });

    // Safety Invariant: Protect conversations tied to active/queued prompts and retain latest 5 sessions of active projects
    let mut protected_cids = std::collections::HashSet::new();

    // 0. Include active/running conversations detected directly from conversation_summaries.db
    protected_cids.extend(active_db_cids);

    // 1. Collect conversation/session IDs from active or queued prompts in repo_db
    if let Ok(all_prompts) = crate::modules::repo_db::list_all_prompts() {
        for p in all_prompts {
            let is_active = p.status == "running"
                || p.status == "queued"
                || p.status == "pending"
                || p.status == "backed_up"
                || p.status == "dispatched"
                || p.status == "executing";
            if is_active {
                if let Some(ref sid) = p.session_id {
                    let trimmed = sid.trim();
                    if !trimmed.is_empty() && trimmed != "-" {
                        protected_cids.insert(trimmed.to_string());
                    }
                }
                if !p.id.trim().is_empty() {
                    protected_cids.insert(p.id.trim().to_string());
                }
            }
        }
    }

    // 2. Identify running projects and retain their latest 5 conversation sessions
    if let Ok(running_projs) = crate::modules::repo_db::list_running_projects() {
        for proj in running_projs {
            let p_norm = proj.repo_path.trim().to_lowercase().replace('\\', "/");
            let p_id_norm = proj.id.trim().to_lowercase();
            let p_name_norm = proj.repo_name.trim().to_lowercase();

            // Check if .antigravity_resume_task.json exists and protect its session/conversation ID
            if !proj.repo_path.trim().is_empty() {
                let resume_file =
                    PathBuf::from(&proj.repo_path).join(".antigravity_resume_task.json");
                if resume_file.exists() {
                    if let Ok(content) = fs::read_to_string(&resume_file) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(cid) = val.get("conversation_id").and_then(|v| v.as_str()) {
                                if !cid.trim().is_empty() {
                                    protected_cids.insert(cid.trim().to_string());
                                }
                            }
                            if let Some(sid) = val.get("session_id").and_then(|v| v.as_str()) {
                                if !sid.trim().is_empty() {
                                    protected_cids.insert(sid.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }

            // Find matching conversations for this project (conversations is already sorted newest first)
            let mut match_indices = Vec::new();
            for (idx, c) in conversations.iter().enumerate() {
                let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
                let db_norm = c.db_path.to_lowercase().replace('\\', "/");
                let is_match = (!p_norm.is_empty()
                    && (uris_norm.contains(&p_norm) || db_norm.contains(&p_norm)))
                    || (!p_id_norm.is_empty() && uris_norm.contains(&p_id_norm))
                    || (!p_name_norm.is_empty() && uris_norm.contains(&p_name_norm));
                if is_match {
                    match_indices.push(idx);
                }
            }

            // Retain up to 5 latest conversation sessions for this active project
            for idx in match_indices.into_iter().take(5) {
                conversations[idx].is_preserved = true;
            }
        }
    }

    // 3. Mark all active prompt conversation IDs as preserved
    for conv in conversations.iter_mut() {
        if protected_cids.contains(&conv.conversation_id) {
            conv.is_preserved = true;
        }
    }

    // 4. Mark top global keep_count as preserved
    for (idx, conv) in conversations.iter_mut().enumerate() {
        let is_within_keep = idx < keep_count;
        if is_within_keep {
            conv.is_preserved = true;
        }
    }

    conversations
}

/// Perform a dry-run pre-flight check
pub fn preflight_check(keep_count: usize) -> PreflightReport {
    let convs = scan_conversations(keep_count);
    let mut preserved = Vec::new();
    let mut pruned = Vec::new();

    let mut total_bytes = 0u64;
    let mut projected_reclaimed = 0u64;

    for conv in convs {
        total_bytes += conv.file_size;
        if conv.is_preserved {
            preserved.push(conv);
        } else {
            projected_reclaimed += conv.file_size;
            pruned.push(conv);
        }
    }

    let existing_cache_paths = crate::modules::cache::get_existing_cache_paths();
    let cache_paths_count = existing_cache_paths.len();
    let mut cache_bytes = 0u64;
    let mut cache_targets = Vec::new();

    for p in &existing_cache_paths {
        let sz = get_dir_size_bytes(p);
        cache_bytes += sz;
        cache_targets.push(format!(
            "{} ({:.2} MB)",
            p.display(),
            sz as f64 / 1024.0 / 1024.0
        ));
    }

    let staging_dir = get_temp_staging_dir().to_string_lossy().to_string();

    PreflightReport {
        total_conversations: preserved.len() + pruned.len(),
        keep_count,
        preserved_count: preserved.len(),
        pruned_count: pruned.len(),
        total_conversation_bytes: total_bytes,
        projected_reclaimed_bytes: projected_reclaimed,
        cache_paths_count,
        cache_bytes,
        conversations_to_preserve: preserved,
        conversations_to_prune: pruned,
        cache_targets,
        staging_dir,
    }
}
