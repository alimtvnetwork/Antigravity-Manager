//! Repo DB: tree conversation node builders

use super::dispatch::parse_flexible_timestamp;
use super::models::{ActivePrompt, AgmConversationNode, RunningProject};
use super::project_queries::normalize_path_for_compare;
use super::sequences::{
    ensure_conversation_sequence_in_conn, extract_prompt_tail_snippet, extract_prompt_words_preview,
};
use super::state::get_active_agy_workers;
use super::text_utils::extract_smart_prompt_summary;
use rusqlite::Connection;

pub(crate) fn build_transcript_conv_nodes(
    ctx: &super::tree_nodes::TreeNodeCtx,
    proj: &RunningProject,
    norm_path: &str,
    norm_proj_inst: &str,
    project_key: &str,
    is_inst_alive: bool,
    conv_nodes: &mut Vec<AgmConversationNode>,
) {
    let convs_by_inst_and_path = ctx.convs_by_inst_and_path;
    let word_cap = ctx.word_cap;
    let only_running = ctx.only_running;
    let conn_opt = ctx.conn_opt;
    let registry = ctx.registry;
    // Instance identity for the current project (used when conv_inst_id matches proj.instance_id)
    let instance_seq_num: Option<i64> = None;
    let instance_name: String = proj.instance_id.clone();
    let instance_exe_name: String =
        crate::modules::instance::resolve_instance_exe_name(&proj.instance_id, None);
    if let Some(raw_convs) =
        convs_by_inst_and_path.get(&(norm_proj_inst.clone(), norm_path.clone()))
    {
        for item in raw_convs {
            let (preview_200w, word_count) = extract_prompt_words_preview(&item.prompt, word_cap);

            // Filter out empty "Untitled Conversation" (0 Words) nodes
            let is_untitled =
                item.title.trim().is_empty() || item.title.to_lowercase().starts_with("untitled");
            let is_empty_prompt = item.prompt.trim().is_empty() || word_count == 0;
            if is_untitled && is_empty_prompt {
                continue;
            }

            let effective_is_run = is_inst_alive && item.is_running;
            if only_running && !effective_is_run {
                continue;
            }
            let c_seq = if let Some(ref conn) = conn_opt {
                ensure_conversation_sequence_in_conn(
                    conn,
                    &item.cid,
                    &project_key,
                    &item.title,
                    &item.conv_inst_id,
                )
            } else {
                (conv_nodes.len() as i64) + 1
            };
            let short_id = if item.cid.len() >= 8 {
                item.cid[..8].to_string()
            } else {
                item.cid.clone()
            };
            let tail_snippet = extract_prompt_tail_snippet(&item.prompt, 12);

            let (c_inst_seq, c_inst_name, c_inst_exe) = if item.conv_inst_id == proj.instance_id {
                (
                    instance_seq_num,
                    instance_name.clone(),
                    instance_exe_name.clone(),
                )
            } else if item.conv_inst_id == "default" || item.conv_inst_id == "__default__" {
                (
                    Some(1),
                    "default".to_string(),
                    crate::modules::instance::resolve_instance_exe_name("default", None),
                )
            } else if let Some(inst) = registry
                .instances
                .iter()
                .find(|i| i.id == item.conv_inst_id || i.name == item.conv_inst_id)
            {
                (
                    inst.seq_num,
                    inst.name.clone(),
                    crate::modules::instance::resolve_instance_exe_name(
                        &inst.id,
                        inst.executable_path.as_deref(),
                    ),
                )
            } else {
                (
                    None,
                    item.conv_inst_id.clone(),
                    crate::modules::instance::resolve_instance_exe_name(&item.conv_inst_id, None),
                )
            };

            conv_nodes.push(AgmConversationNode {
                seq_id: c_seq,
                seq_code: format!("C{:03}", c_seq),
                gitmap_seq_code: format!("GM:{}", short_id),
                conversation_id: item.cid.clone(),
                short_id,
                title: if item.title.trim().is_empty() {
                    "Untitled Conversation".to_string()
                } else {
                    item.title.clone()
                },
                status: if effective_is_run {
                    "RUNNING".to_string()
                } else if item.is_queued {
                    "QUEUED".to_string()
                } else if item.status.trim().is_empty() || item.status.contains("RUNNING") {
                    "IDLE".to_string()
                } else {
                    item.status.clone()
                },
                is_running: effective_is_run,
                step_count: item.steps,
                instance_id: item.conv_inst_id.clone(),
                instance_seq_num: c_inst_seq,
                instance_name: c_inst_name,
                instance_exe_name: c_inst_exe,
                prompt_preview_200w: preview_200w,
                prompt_tail_snippet: tail_snippet,
                prompt_word_count: word_count,
                last_modified: item.last_mod.clone(),
                byte_size: Some(item.prompt.len()),
                repeat_count: None,
                repeat_badge: None,
                sub_runs: Vec::new(),
                prompt_category: item.prompt_category.clone(),
                is_queued: item.is_queued,
                latest_step_summary: item.latest_step_summary.clone(),
                latest_response: item.latest_response.clone(),
                execution_results: item.execution_results.clone(),
                tool_calls_summary: item.tool_calls_summary.clone(),
                full_prompt_text: Some(item.prompt.clone()),
            });
        }
    }
}

pub(crate) fn build_active_prompt_conv_nodes(
    ctx: &super::tree_nodes::TreeNodeCtx,
    proj: &RunningProject,
    norm_path: &str,
    norm_proj_inst: &str,
    project_key: &str,
    is_inst_alive: bool,
    conv_nodes: &mut Vec<AgmConversationNode>,
) {
    let active_prompts_by_inst_and_path = ctx.active_prompts_by_inst_and_path;
    let candidate_dirs = ctx.candidate_dirs;
    let word_cap = ctx.word_cap;
    let only_running = ctx.only_running;
    let conn_opt = ctx.conn_opt;
    let registry = ctx.registry;
    let now = ctx.now;
    if let Some(aps) =
        active_prompts_by_inst_and_path.get(&(norm_proj_inst.clone(), norm_path.clone()))
    {
        for ap in aps {
            let cid = ap.session_id.clone().unwrap_or_else(|| ap.id.clone());
            if conv_nodes.iter().any(|c| c.conversation_id == cid) {
                continue;
            }
            let (preview_200w, word_count) =
                extract_prompt_words_preview(&ap.prompt_content, word_cap);

            let title = extract_smart_prompt_summary(&ap.prompt_content, 60);
            let is_untitled =
                title.trim().is_empty() || title.to_lowercase().starts_with("untitled");
            let is_empty_prompt = ap.prompt_content.trim().is_empty() || word_count == 0;
            if is_untitled && is_empty_prompt {
                continue;
            }

            let has_active_worker = if let Ok(workers) = get_active_agy_workers().lock() {
                let clean_target = normalize_path_for_compare(&proj.repo_path);
                workers.iter().any(|(key, &wpid)| {
                    let (w_inst, w_path) = match key.split_once(':') {
                        Some((inst, path)) => (inst, path),
                        None => ("", key.as_str()),
                    };
                    let is_inst_match =
                        if proj.instance_id == "default" || proj.instance_id == "__default__" {
                            w_inst == "default" || w_inst == "__default__"
                        } else {
                            w_inst.eq_ignore_ascii_case(&proj.instance_id)
                        };
                    let is_path_match = !clean_target.is_empty()
                        && normalize_path_for_compare(w_path) == clean_target;
                    if is_inst_match && is_path_match && wpid > 0 {
                        crate::modules::instance::is_pid_alive_os(wpid)
                    } else {
                        false
                    }
                })
            } else {
                false
            };

            let has_non_idle_summary = if let Some(ref session_id) = ap.session_id {
                let mut is_non_idle = false;
                for (owning_inst_id, base) in candidate_dirs {
                    let norm_owning =
                        if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
                            "default"
                        } else {
                            owning_inst_id.as_str()
                        };
                    if norm_owning == norm_proj_inst || norm_owning == proj.instance_id {
                        let s_db = base.join("conversation_summaries.db");
                        if s_db.exists() {
                            if let Ok(s_conn) = Connection::open_with_flags(
                                &s_db,
                                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                                    | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                            ) {
                                // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
                                crate::error::record_ignored(
                                    s_conn.pragma_update(None, "busy_timeout", 1000),
                                    "set busy_timeout on Antigravity summaries database",
                                );
                                let query = "SELECT status, not_fully_idle FROM conversation_summaries WHERE conversation_id = ?1 LIMIT 1";
                                if let Ok((sum_status, not_idle)) =
                                    s_conn.query_row(query, [session_id], |row| {
                                        Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
                                    })
                                {
                                    let has_idle_marker = sum_status.contains("IDLE")
                                        || sum_status.contains("COMPLETED")
                                        || sum_status.contains("FAILED")
                                        || sum_status.contains("CANCELLED");
                                    if sum_status.contains("RUNNING")
                                        && not_idle > 0
                                        && !has_idle_marker
                                    {
                                        is_non_idle = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                is_non_idle
            } else {
                false
            };

            let has_confirmed_running = has_active_worker || has_non_idle_summary;

            let is_ap_queued = ap.status == "queued" || ap.status == "pending";
            let max_allowed_ttl = if has_active_worker { 300 } else { 45 };
            let is_run = is_inst_alive
                && (ap.status == "running"
                    || ap.status == "in_flight"
                    || ap.status == "dispatched")
                && (now - ap.updated_at <= max_allowed_ttl)
                && has_confirmed_running;
            if only_running && !is_run {
                continue;
            }
            let c_seq = if let Some(ref conn) = conn_opt {
                ensure_conversation_sequence_in_conn(
                    conn,
                    &cid,
                    &project_key,
                    &title,
                    &ap.instance_id,
                )
            } else {
                (conv_nodes.len() as i64) + 1
            };
            let short_id = if cid.len() >= 8 {
                cid[..8].to_string()
            } else {
                cid.clone()
            };
            let tail_snippet = extract_prompt_tail_snippet(&ap.prompt_content, 12);

            let (c_inst_seq, c_inst_name, c_inst_exe) = if ap.instance_id == proj.instance_id {
                (
                    instance_seq_num,
                    instance_name.clone(),
                    instance_exe_name.clone(),
                )
            } else if ap.instance_id == "default" || ap.instance_id == "__default__" {
                (
                    Some(1),
                    "default".to_string(),
                    crate::modules::instance::resolve_instance_exe_name("default", None),
                )
            } else if let Some(inst) = registry
                .instances
                .iter()
                .find(|i| i.id == ap.instance_id || i.name == ap.instance_id)
            {
                (
                    inst.seq_num,
                    inst.name.clone(),
                    crate::modules::instance::resolve_instance_exe_name(
                        &inst.id,
                        inst.executable_path.as_deref(),
                    ),
                )
            } else {
                (
                    None,
                    ap.instance_id.clone(),
                    crate::modules::instance::resolve_instance_exe_name(&ap.instance_id, None),
                )
            };

            let ap_category = {
                let p_lower = ap.prompt_content.to_lowercase();
                if p_lower.contains("you are research")
                    || p_lower.contains("you are worker")
                    || p_lower.contains("<system_message>")
                {
                    "subagent".to_string()
                } else {
                    "user".to_string()
                }
            };
            let ap_step_summary = if is_run {
                Some("Prompt in flight / processing".to_string())
            } else if is_ap_queued {
                Some("In queue, waiting for slot".to_string())
            } else {
                None
            };

            conv_nodes.push(AgmConversationNode {
                seq_id: c_seq,
                seq_code: format!("C{:03}", c_seq),
                gitmap_seq_code: format!("GM:{}", short_id),
                conversation_id: cid,
                short_id,
                title,
                status: if is_run {
                    "RUNNING".to_string()
                } else if is_ap_queued {
                    "QUEUED".to_string()
                } else if ap.status == "running"
                    || ap.status == "dispatched"
                    || ap.status == "in_flight"
                {
                    "IDLE".to_string()
                } else {
                    ap.status.to_uppercase()
                },
                is_running: is_run,
                step_count: 0,
                instance_id: ap.instance_id.clone(),
                instance_seq_num: c_inst_seq,
                instance_name: c_inst_name,
                instance_exe_name: c_inst_exe,
                prompt_preview_200w: preview_200w,
                prompt_tail_snippet: tail_snippet,
                prompt_word_count: word_count,
                last_modified: ap.updated_at.to_string(),
                byte_size: Some(ap.prompt_content.len()),
                repeat_count: None,
                repeat_badge: None,
                sub_runs: Vec::new(),
                prompt_category: ap_category,
                is_queued: is_ap_queued,
                latest_step_summary: ap_step_summary,
                latest_response: None,
                execution_results: None,
                tool_calls_summary: None,
                full_prompt_text: Some(ap.prompt_content.clone()),
            });
        }
    }
}
