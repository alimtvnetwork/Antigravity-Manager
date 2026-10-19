//! Repo DB: tree conversation node builders

use super::models::{AgmConversationNode, AgmPromptTurnNode, CascadeRunStatusType, RunningProject};
use super::project_queries::normalize_path_for_compare;
use super::sequences::{
    ensure_conversation_sequence_in_conn, extract_prompt_tail_snippet, extract_prompt_words_preview,
};
use super::state::get_active_agy_workers;
use super::text_utils::extract_smart_prompt_summary;
use rusqlite::Connection;

/// Returns true if the instance ID represents the default instance or its copies/aliases.
pub(crate) fn is_default_instance_match(inst_id: &str) -> bool {
    let is_exact_default = inst_id == "default" || inst_id == "__default__";
    let is_alias_default = inst_id.is_empty() || inst_id.starts_with("default-copy");
    is_exact_default || is_alias_default
}

/// Returns true if inst_id matches the project's instance directly or via default alias compatibility.
pub(crate) fn is_instance_compatible(
    inst_id: &str,
    norm_proj_inst: &str,
    is_proj_default: bool,
) -> bool {
    if inst_id == norm_proj_inst {
        return true;
    }
    let is_inst_default = is_default_instance_match(inst_id);
    is_proj_default && is_inst_default
}

/// Affirmative detection of empty, placeholder, or untitled conversations.
pub(crate) fn is_vacant_ghost_conversation(
    title: &str,
    prompt: &str,
    word_count: usize,
    has_zero_turns: bool,
) -> bool {
    if !has_zero_turns {
        return false;
    }
    let is_prompt_empty = prompt.trim().is_empty();
    let has_zero_words = word_count == 0;
    let is_prompt_vacant = is_prompt_empty || has_zero_words;
    if !is_prompt_vacant {
        return false;
    }
    let title_clean = title.trim();
    if title_clean.is_empty() {
        return true;
    }
    let title_lower = title_clean.to_lowercase();
    let is_untitled = title_lower.starts_with("untitled");
    let is_new_conv = title_lower == "new conversation";
    is_untitled || is_new_conv
}

/// Affirmative retention check when filtering conversations by only_running.
pub(crate) fn should_retain_unrunning_conversation(only_running: bool, is_running: bool) -> bool {
    if !only_running {
        return true;
    }
    is_running
}

/// Evaluates whether an unrunning conversation candidate has sufficient title or prompt content.
pub(crate) fn should_retain_prompt_candidate(
    is_non_prompt: bool,
    is_conv_running: bool,
    has_meaningful_title: bool,
    has_prompt_content: bool,
) -> bool {
    if !is_non_prompt {
        return true;
    }
    if is_conv_running {
        return true;
    }
    has_meaningful_title || has_prompt_content
}

/// Affirmative evaluation of owning instance OS liveness (shared between tree_collect and tree_phases).
pub(crate) fn is_owning_instance_alive(
    owning_inst_id: &str,
    registry: &crate::models::instance::InstanceRegistry,
) -> bool {
    if is_default_instance_match(owning_inst_id) {
        let is_process_running = crate::modules::process::is_antigravity_running(None);
        if is_process_running {
            return true;
        }
        let def_dir = crate::modules::instance::get_default_antigravity_data_dir();
        let pids =
            crate::modules::instance::find_pids_for_data_dir(&def_dir.to_string_lossy(), true);
        let has_pids = !pids.is_empty();
        return has_pids;
    }
    if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.id == owning_inst_id || i.name == owning_inst_id)
    {
        let is_inst_running =
            crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        if is_inst_running {
            return true;
        }
        let is_global_running = crate::modules::process::is_antigravity_running(None);
        return inst.is_default && is_global_running;
    }
    crate::modules::process::is_antigravity_running(None)
}

pub(crate) fn resolve_instance_meta(
    instance_id: &str,
    registry: &crate::models::instance::InstanceRegistry,
) -> (Option<u32>, String, String) {
    if is_default_instance_match(instance_id) {
        (
            Some(1),
            "default".to_string(),
            crate::modules::instance::resolve_instance_exe_name("default", None),
        )
    } else if let Some((idx, inst)) = registry
        .instances
        .iter()
        .enumerate()
        .find(|(_, i)| i.id == instance_id || i.name == instance_id)
    {
        let seq = inst.seq_num.or_else(|| {
            let is_inst_default = inst.is_default || is_default_instance_match(&inst.id);
            if is_inst_default {
                Some(1)
            } else {
                Some((idx as u32) + 1)
            }
        });
        (
            seq,
            inst.name.clone(),
            crate::modules::instance::resolve_instance_exe_name(
                &inst.id,
                inst.executable_path.as_deref(),
            ),
        )
    } else {
        (
            None,
            instance_id.to_string(),
            crate::modules::instance::resolve_instance_exe_name(instance_id, None),
        )
    }
}

pub(crate) fn build_transcript_conv_nodes(
    ctx: &super::tree_nodes::TreeNodeCtx,
    proj: &RunningProject,
    norm_path: &str,
    norm_proj_inst: &str,
    project_key: &str,
    _is_inst_alive: bool,
    conv_nodes: &mut Vec<AgmConversationNode>,
) {
    let convs_by_inst_and_path = ctx.convs_by_inst_and_path;
    let word_cap = ctx.word_cap;
    let only_running = ctx.only_running;
    let conn_opt = ctx.conn_opt;
    let registry = ctx.registry;
    // Instance identity for the current project (used when conv_inst_id matches proj.instance_id)
    let (instance_seq_num, instance_name, instance_exe_name) =
        resolve_instance_meta(&proj.instance_id, registry);
    let is_proj_default = is_default_instance_match(norm_proj_inst);

    let mut candidate_raw_convs: Vec<&super::tree_collect::RawConvItem> = Vec::new();
    let mut seen_cids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for ((inst_id, p_path), items) in convs_by_inst_and_path {
        if p_path == norm_path {
            let matches_inst = is_instance_compatible(inst_id, norm_proj_inst, is_proj_default);
            if matches_inst {
                for it in items {
                    if seen_cids.insert(it.cid.clone()) {
                        candidate_raw_convs.push(it);
                    }
                }
            }
        }
    }

    if !candidate_raw_convs.is_empty() {
        for item in candidate_raw_convs {
            let (preview_200w, word_count) = extract_prompt_words_preview(&item.prompt, word_cap);

            // Filter out empty "Untitled Conversation" (0 Words) nodes
            let is_ghost =
                is_vacant_ghost_conversation(&item.title, &item.prompt, word_count, true);
            if is_ghost {
                continue;
            }

            let effective_is_run = item.is_running;
            let should_retain =
                should_retain_unrunning_conversation(only_running, effective_is_run);
            if !should_retain {
                continue;
            }
            let c_seq = if let Some(ref conn) = conn_opt {
                ensure_conversation_sequence_in_conn(
                    conn,
                    &item.cid,
                    project_key,
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
            } else {
                resolve_instance_meta(&item.conv_inst_id, registry)
            };

            let is_item_running = CascadeRunStatusType::from_status_str(&item.status).is_running();
            let has_empty_status = item.status.trim().is_empty();
            let is_idle_status = has_empty_status || is_item_running;
            let conv_status = if effective_is_run {
                "RUNNING".to_string()
            } else if item.is_queued {
                "QUEUED".to_string()
            } else if is_idle_status {
                "IDLE".to_string()
            } else {
                item.status.clone()
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
                status: conv_status,
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
                prompts: item.prompts.clone(),
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
    let (instance_seq_num, instance_name, instance_exe_name) =
        resolve_instance_meta(&proj.instance_id, registry);
    if let Some(aps) =
        active_prompts_by_inst_and_path.get(&(norm_proj_inst.to_string(), norm_path.to_string()))
    {
        for ap in aps {
            let cid = ap.session_id.clone().unwrap_or_else(|| ap.id.clone());
            let target_idx = conv_nodes.iter().position(|c| {
                let is_id_match =
                    c.conversation_id == cid || c.conversation_id.eq_ignore_ascii_case(&cid);
                if is_id_match {
                    return true;
                }
                let content_trimmed = ap.prompt_content.trim();
                let has_matching_prompt = c.prompts.iter().any(|p| {
                    let is_text_match = p.prompt_text.trim() == content_trimmed;
                    let is_preview_match = p.preview_200w.trim() == content_trimmed;
                    is_text_match || is_preview_match
                });
                if has_matching_prompt {
                    return true;
                }
                c.full_prompt_text.as_deref().unwrap_or("").trim() == content_trimmed
            });

            if let Some(idx) = target_idx {
                let target_conv = &mut conv_nodes[idx];
                let ap_status_lower = ap.status.to_lowercase();
                let is_pending_stage =
                    ap_status_lower.contains("queue") || ap_status_lower.contains("pending");
                let is_active_stage = ap_status_lower.contains("running")
                    || ap_status_lower.contains("dispatched")
                    || ap_status_lower.contains("in_flight");
                let is_relevant_status = is_pending_stage || is_active_stage;

                if is_relevant_status {
                    let is_queued = is_pending_stage;
                    let is_fresh = (now - ap.updated_at) <= 600;
                    let is_alive_and_fresh = is_inst_alive && is_fresh;
                    let is_ap_run = is_alive_and_fresh && is_active_stage;

                    // Check if an existing turn already matches this prompt content
                    let existing_turn = target_conv
                        .prompts
                        .iter_mut()
                        .find(|p| p.prompt_text.trim() == ap.prompt_content.trim());

                    if let Some(turn) = existing_turn {
                        if is_ap_run {
                            turn.is_running = true;
                            turn.status = "RUNNING".to_string();
                            target_conv.is_running = true;
                            target_conv.status = "RUNNING".to_string();
                        } else if is_queued {
                            turn.is_queued = true;
                            turn.status = "QUEUED".to_string();
                            let can_enqueue = !target_conv.is_running;
                            if can_enqueue {
                                target_conv.is_queued = true;
                                target_conv.status = "QUEUED".to_string();
                            }
                        }
                    } else {
                        let turn_num = target_conv.prompts.len() + 1;
                        let (ap_preview, _) =
                            extract_prompt_words_preview(&ap.prompt_content, word_cap);
                        let ap_turn = AgmPromptTurnNode {
                            turn_number: turn_num,
                            step_index: target_conv.prompts.len(),
                            prompt_text: ap.prompt_content.clone(),
                            preview_200w: ap_preview,
                            status: if is_ap_run {
                                "RUNNING".to_string()
                            } else if is_queued {
                                "QUEUED".to_string()
                            } else {
                                "COMPLETED".to_string()
                            },
                            is_running: is_ap_run,
                            is_queued,
                            timestamp: chrono::DateTime::from_timestamp(ap.updated_at, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                            created_at: if ap.created_at > 0 && ap.created_at < 100_000_000_000 {
                                ap.created_at * 1000
                            } else {
                                ap.created_at
                            },
                            media_paths: Vec::new(),
                            media_thumbnails: Vec::new(),
                        };
                        target_conv.prompts.push(ap_turn);
                        if is_queued {
                            target_conv.is_queued = true;
                        }
                        let can_enqueue_conv = is_queued && !target_conv.is_running;
                        if is_ap_run {
                            target_conv.is_running = true;
                            target_conv.status = "RUNNING".to_string();
                        } else if can_enqueue_conv {
                            target_conv.status = "QUEUED".to_string();
                        }
                    }
                }
                continue;
            }
            let (preview_200w, word_count) =
                extract_prompt_words_preview(&ap.prompt_content, word_cap);

            let title = extract_smart_prompt_summary(&ap.prompt_content, 60);
            let is_ghost =
                is_vacant_ghost_conversation(&title, &ap.prompt_content, word_count, true);
            if is_ghost {
                continue;
            }

            let has_active_worker = if let Ok(workers) = get_active_agy_workers().lock() {
                let clean_target = normalize_path_for_compare(&proj.repo_path);
                workers.iter().any(|(key, &wpid)| {
                    let (w_inst, w_path) = match key.split_once(':') {
                        Some((inst, path)) => (inst, path),
                        None => ("", key.as_str()),
                    };
                    let is_inst_match = if is_default_instance_match(&proj.instance_id) {
                        is_default_instance_match(w_inst)
                    } else {
                        w_inst.eq_ignore_ascii_case(&proj.instance_id)
                    };
                    let has_clean_target = !clean_target.is_empty();
                    let is_path_equal = normalize_path_for_compare(w_path) == clean_target;
                    let is_path_match = has_clean_target && is_path_equal;

                    let is_target_match = is_inst_match && is_path_match;
                    let has_valid_pid = wpid > 0;
                    if is_target_match && has_valid_pid {
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
                    let norm_owning = if is_default_instance_match(owning_inst_id) {
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
                                    let cascade_status =
                                        CascadeRunStatusType::from_status_str(&sum_status);
                                    let has_idle_marker = cascade_status.is_idle_or_terminal();
                                    let is_cascade_running = cascade_status.is_running();
                                    let has_busy_flag = not_idle > 0;
                                    let is_turn_busy = is_cascade_running && has_busy_flag;
                                    let is_active_turn = is_turn_busy && !has_idle_marker;
                                    if is_active_turn {
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
            let is_running_status =
                ap.status == "running" || ap.status == "in_flight" || ap.status == "dispatched";
            let is_within_ttl = (now - ap.updated_at) <= max_allowed_ttl;
            let is_status_fresh = is_running_status && is_within_ttl;
            let is_executing = is_inst_alive && has_confirmed_running;
            let is_run = is_status_fresh && is_executing;
            let should_retain_ap = should_retain_unrunning_conversation(only_running, is_run);
            if !should_retain_ap {
                continue;
            }
            let c_seq = if let Some(ref conn) = conn_opt {
                ensure_conversation_sequence_in_conn(
                    conn,
                    &cid,
                    project_key,
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
            } else {
                resolve_instance_meta(&ap.instance_id, registry)
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

            let ap_turn = AgmPromptTurnNode {
                turn_number: 1,
                step_index: 0,
                prompt_text: ap.prompt_content.clone(),
                preview_200w: preview_200w.clone(),
                status: if is_run {
                    "RUNNING".to_string()
                } else if is_ap_queued {
                    "QUEUED".to_string()
                } else {
                    "COMPLETED".to_string()
                },
                is_running: is_run,
                is_queued: is_ap_queued,
                timestamp: chrono::DateTime::from_timestamp(ap.updated_at, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                created_at: if ap.created_at > 0 && ap.created_at < 100_000_000_000 {
                    ap.created_at * 1000
                } else {
                    ap.created_at
                },
                media_paths: Vec::new(),
                media_thumbnails: Vec::new(),
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
                } else if is_running_status {
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
                prompts: vec![ap_turn],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_default_instance_match() {
        assert!(is_default_instance_match("default"));
        assert!(is_default_instance_match("__default__"));
        assert!(is_default_instance_match(""));
        assert!(is_default_instance_match("default-copy"));
        assert!(is_default_instance_match("default-copy-5597"));
        assert!(!is_default_instance_match("instance-1"));
        assert!(!is_default_instance_match("custom_inst"));
    }

    #[test]
    fn test_is_instance_compatible() {
        assert!(is_instance_compatible("default", "default", true));
        assert!(is_instance_compatible("__default__", "default", true));
        assert!(is_instance_compatible("default-copy-1", "default", true));
        assert!(is_instance_compatible("worker_1", "worker_1", false));
        assert!(!is_instance_compatible("worker_2", "worker_1", false));
        assert!(!is_instance_compatible("worker_2", "default", true));
    }

    #[test]
    fn test_is_vacant_ghost_conversation() {
        // Zero turns with empty prompt and untitled
        assert!(is_vacant_ghost_conversation("Untitled", "", 0, true));
        assert!(is_vacant_ghost_conversation(
            "New Conversation",
            "   ",
            0,
            true
        ));
        assert!(is_vacant_ghost_conversation("", "", 0, true));
        // Has turns -> not ghost
        assert!(!is_vacant_ghost_conversation("Untitled", "", 0, false));
        // Has words in prompt -> not ghost
        assert!(!is_vacant_ghost_conversation(
            "Untitled",
            "hello world",
            2,
            true
        ));
        // Meaningful title -> not ghost
        assert!(!is_vacant_ghost_conversation("Fix login bug", "", 0, true));
    }

    #[test]
    fn test_should_retain_unrunning_conversation() {
        assert!(should_retain_unrunning_conversation(false, false));
        assert!(should_retain_unrunning_conversation(false, true));
        assert!(should_retain_unrunning_conversation(true, true));
        assert!(!should_retain_unrunning_conversation(true, false));
    }

    #[test]
    fn test_should_retain_prompt_candidate() {
        // Not a non-prompt -> retain
        assert!(should_retain_prompt_candidate(false, false, false, false));
        // Running -> retain
        assert!(should_retain_prompt_candidate(true, true, false, false));
        // Has meaningful title -> retain
        assert!(should_retain_prompt_candidate(true, false, true, false));
        // Has prompt content -> retain
        assert!(should_retain_prompt_candidate(true, false, false, true));
        // Non-prompt, not running, no title, no content -> discard
        assert!(!should_retain_prompt_candidate(true, false, false, false));
    }

    #[test]
    fn test_is_owning_instance_alive_fallback() {
        let registry = crate::models::instance::InstanceRegistry::default();
        let _ = is_owning_instance_alive("non_existent_instance", &registry);
    }
}
