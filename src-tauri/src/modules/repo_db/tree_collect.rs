//! Repo DB: tree conversation collection

#[derive(Debug, Clone)]
pub(crate) struct RawConvItem {
    pub(crate) cid: String,
    pub(crate) title: String,
    pub(crate) prompt: String,
    pub(crate) status: String,
    pub(crate) is_running: bool,
    pub(crate) steps: usize,
    pub(crate) last_mod: String,
    pub(crate) conv_inst_id: String,
    pub(crate) prompt_category: String,
    pub(crate) is_queued: bool,
    pub(crate) latest_step_summary: Option<String>,
    pub(crate) latest_response: Option<String>,
    pub(crate) execution_results: Option<String>,
    pub(crate) tool_calls_summary: Option<String>,
}

use super::gemini_dirs::gemini_dirs_tagged;
use super::models::ActivePrompt;
use super::project_queries::list_all_prompts;
use super::schema::connect_db;
use super::transcript::{inspect_conversation_transcript, resolve_transcript_path};
use chrono::Utc;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) fn collect_tree_conversation_items(
    target: Option<&str>,
) -> (
    Option<rusqlite::Connection>,
    std::collections::HashMap<(String, String), Vec<RawConvItem>>,
    std::collections::HashMap<(String, String), Vec<ActivePrompt>>,
    Vec<(String, std::path::PathBuf)>,
) {
    let conn_opt = connect_db().ok();

    let mut convs_by_inst_and_path: std::collections::HashMap<(String, String), Vec<RawConvItem>> =
        std::collections::HashMap::new();

    let mut active_prompts_by_inst_and_path: std::collections::HashMap<
        (String, String),
        Vec<ActivePrompt>,
    > = std::collections::HashMap::new();
    if let Ok(active_list) = list_all_prompts() {
        for ap in active_list {
            if ap.instance_id.trim().is_empty() {
                continue;
            }
            let key = normalize_path_for_compare(&ap.repo_path);
            let inst = if ap.instance_id == "default" || ap.instance_id == "__default__" {
                "default".to_string()
            } else {
                crate::modules::instance::resolve_instance_id(&ap.instance_id)
                    .unwrap_or_else(|_| ap.instance_id.clone())
            };
            active_prompts_by_inst_and_path
                .entry((inst, key))
                .or_default()
                .push(ap);
        }
    }

    let candidate_dirs = gemini_dirs_tagged(target);

    let mut seen_tree_cids: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();

    for (owning_inst_id, base) in &candidate_dirs {
        let summaries_db = base.join("conversation_summaries.db");
        if summaries_db.exists() {
            let s_conn = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
            .or_else(|_| {
                let uri = format!(
                    "file:{}?immutable=1",
                    summaries_db.to_string_lossy().replace('\\', "/")
                );
                Connection::open_with_flags(
                    &uri,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                )
            });

            if let Ok(s_conn) = s_conn {
                let is_owning_inst_alive = if owning_inst_id == "default"
                    || owning_inst_id == "__default__"
                {
                    crate::modules::process::is_antigravity_running(None) || {
                        let def_dir = crate::modules::instance::get_default_antigravity_data_dir();
                        !crate::modules::instance::find_pids_for_data_dir(
                            &def_dir.to_string_lossy(),
                            true,
                        )
                        .is_empty()
                    }
                } else if let Some(inst) = registry
                    .instances
                    .iter()
                    .find(|i| i.id == *owning_inst_id || i.name == *owning_inst_id)
                {
                    crate::modules::instance::is_instance_running(
                        &inst.id,
                        &inst.data_dir,
                        inst.pid,
                    ) || (inst.is_default && crate::modules::process::is_antigravity_running(None))
                } else {
                    crate::modules::process::is_antigravity_running(None)
                };

                // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
                crate::error::record_ignored(
                    s_conn.pragma_update(None, "busy_timeout", 3000),
                    "set busy_timeout on Antigravity summaries database",
                );
                if let Ok(mut stmt) = s_conn.prepare(
                    "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time 
                     FROM conversation_summaries 
                     ORDER BY last_modified_time DESC 
                     LIMIT 200",
                ) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, i32>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, String>(6)?,
                        ))
                    }) {
                        for item in rows.flatten() {
                            let (cid, title, preview, status, not_fully_idle, ws_uris_opt, last_time_str) = item;

                            let norm_owning_inst = if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
                                "default".to_string()
                            } else {
                                crate::modules::instance::resolve_instance_id(owning_inst_id).unwrap_or_else(|_| owning_inst_id.to_string())
                            };
                            if !seen_tree_cids.insert((norm_owning_inst.clone(), cid.clone())) {
                                continue;
                            }

                            let conv_ts = parse_flexible_timestamp(&last_time_str);
                            let is_recent = conv_ts > 0 && (now - conv_ts <= 60);

                            let is_idle_count = not_fully_idle == 0;
                            let has_idle_status = status.contains("IDLE")
                                || status.contains("COMPLETED")
                                || status.contains("FAILED")
                                || status.contains("CANCELLED");
                            let is_explicit_idle = is_idle_count || has_idle_status;

                            let inspection = inspect_conversation_transcript(base, &cid);

                            let is_conv_running = if is_explicit_idle || inspection.is_terminal_done {
                                false
                            } else if is_owning_inst_alive && not_fully_idle > 0 && status.contains("RUNNING") && is_recent {
                                true
                            } else if is_owning_inst_alive && inspection.is_recent_active && !has_idle_status {
                                true
                            } else {
                                false
                            };
                            let steps = inspection.step_count;
                            let effective_prompt = inspection
                                .latest_prompt
                                .filter(|s| !s.trim().is_empty())
                                .unwrap_or_else(|| preview.clone());

                            // Non-prompt filter: skip internal background hooks or non-prompts if not running
                            if inspection.is_non_prompt && !is_conv_running {
                                continue;
                            }

                            let prompt_category = if inspection.is_subagent {
                                "subagent".to_string()
                            } else {
                                let p_lower = effective_prompt.to_lowercase();
                                let t_lower = title.to_lowercase();
                                if t_lower.contains("subagent")
                                    || t_lower.contains("research")
                                    || t_lower.contains("worker")
                                    || t_lower.contains("author")
                                    || t_lower.contains("analysis")
                                    || t_lower.contains("memory analysis")
                                    || t_lower.contains("read & understand")
                                    || t_lower.contains("safe removal")
                                    || t_lower.contains("debugger")
                                    || t_lower.contains("architect")
                                    || t_lower.contains("tester")
                                    || p_lower.contains("you are research")
                                    || p_lower.contains("you are worker")
                                    || p_lower.contains("you are a")
                                    || p_lower.contains("invoked by a caller agent")
                                    || p_lower.contains("execute enhanced read memory")
                                    || p_lower.contains("<subagent_reminder>")
                                    || p_lower.contains("<system_message>")
                                    || p_lower.contains("internal subagent")
                                {
                                    "subagent".to_string()
                                } else {
                                    "user".to_string()
                                }
                            };

                            let is_queued = status.to_lowercase().contains("queue")
                                || status.to_lowercase().contains("pending");
                            let latest_step_summary = inspection.latest_step_summary;

                            // Ghost conversation filter: skip untitled / empty title with empty prompt (regardless of running status)
                            let is_untitled_candidate = title.trim().is_empty()
                                || title.to_lowercase().starts_with("untitled")
                                || title.to_lowercase() == "new conversation";
                            let (_, eff_wc) = extract_prompt_words_preview(&effective_prompt, 5);
                            if (is_untitled_candidate && (effective_prompt.trim().is_empty() || eff_wc == 0))
                                || (effective_prompt.trim().is_empty() && eff_wc == 0)
                            {
                                continue;
                            }

                            let mut assigned_paths: Vec<String> = Vec::new();
                            if let Some(ws_raw) = ws_uris_opt {
                                let uris: Vec<String> =
                                    serde_json::from_str(&ws_raw).unwrap_or_default();
                                for u in uris {
                                    let p = normalize_path_for_compare(&decode_uri_to_path(&u));
                                    if !p.is_empty() && !assigned_paths.contains(&p) {
                                        assigned_paths.push(p);
                                    }
                                }
                            }

                            // Prefix-length / valid path guard: if marked running with assigned paths but no decodable path >= 6 chars, force idle
                            let is_conv_running = if is_conv_running
                                && !assigned_paths.is_empty()
                                && !assigned_paths.iter().any(|p| p.len() >= 6)
                            {
                                false
                            } else {
                                is_conv_running
                            };
                            if assigned_paths.is_empty() {
                                assigned_paths.push("__unassigned__".to_string());
                            }
                            for p_key in assigned_paths {
                                convs_by_inst_and_path
                                    .entry((norm_owning_inst.clone(), p_key))
                                    .or_default()
                                    .push(RawConvItem {
                                        cid: cid.clone(),
                                        title: title.clone(),
                                        prompt: effective_prompt.clone(),
                                        status: if is_conv_running {
                                            "RUNNING".to_string()
                                        } else if is_queued {
                                            "QUEUED".to_string()
                                        } else if inspection.is_terminal_done || is_explicit_idle || status.contains("RUNNING") {
                                            "IDLE".to_string()
                                        } else {
                                            status.clone()
                                        },
                                        is_running: is_conv_running,
                                        steps,
                                        last_mod: last_time_str.clone(),
                                        conv_inst_id: owning_inst_id.clone(),
                                        prompt_category: prompt_category.clone(),
                                        is_queued,
                                        latest_step_summary: latest_step_summary.clone(),
                                        latest_response: inspection.latest_response.clone(),
                                        execution_results: inspection.execution_results.clone(),
                                        tool_calls_summary: inspection.tool_calls_summary.clone(),
                                    });
                            }
                        }
                    }
                }
            }
        }
    }

    (
        conn_opt,
        convs_by_inst_and_path,
        active_prompts_by_inst_and_path,
        candidate_dirs,
    )
}
