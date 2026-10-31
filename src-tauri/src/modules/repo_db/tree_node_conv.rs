//! Repo DB: tree conversation node builders

use super::models::{
    ActivePrompt, AgmConversationNode, AgmPromptTurnNode, CascadeRunStatusType, RunningProject,
};
use super::project_queries::normalize_path_for_compare;
use super::sequences::{
    ensure_conversation_sequence_in_conn, extract_prompt_tail_snippet, extract_prompt_words_preview,
};
use super::state::get_active_agy_workers;
use super::text_utils::{extract_smart_prompt_summary, StrExt};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

fn is_placeholder_title(title: &str) -> bool {
    let trimmed = title.trim();
    let lower = trimmed.to_ascii_lowercase();
    trimmed.is_empty() || lower.starts_with("untitled") || lower.starts_with("new conversation")
}

fn conversation_display_title(title: &str, prompt: &str) -> String {
    if !is_placeholder_title(title) {
        return title.trim().to_string();
    }

    let summary = extract_smart_prompt_summary(prompt, 80);
    if summary.trim().is_empty() {
        return "Untitled Conversation".to_string();
    }

    summary
}

fn workspace_relative_media(path: &str, repo: &str) -> String {
    let normalized = path.replace('\\', "/");
    let repo_norm = repo.replace('\\', "/");
    let repo_trim = repo_norm.trim_end_matches('/');
    if repo_trim.is_empty() {
        return normalized;
    }

    let lower_path = normalized.to_lowercase();
    let lower_repo = repo_trim.to_lowercase();
    let matches_repo = lower_path.starts_with(&lower_repo)
        && (lower_path.len() == lower_repo.len()
            || lower_path.as_bytes().get(lower_repo.len()) == Some(&b'/'));
    if matches_repo {
        return normalized[repo_trim.len()..]
            .trim_start_matches('/')
            .to_string();
    }

    if normalized.starts_with("file:") {
        return normalized;
    }

    let is_drive = normalized.len() >= 2 && normalized.as_bytes().get(1) == Some(&b':');
    if is_drive {
        return format!("file:///{normalized}");
    }

    normalized
}

/// Returns true if the instance ID represents the default instance or its aliases.
pub(crate) fn is_default_instance_match(inst_id: &str) -> bool {
    const DEFAULT_IDS: &[&str] = &[
        "default",
        "__default__",
        "",
        "#1 default",
        "1 default",
        "#1 (default)",
        "default (active)",
        "#1 default (active)",
    ];
    let clean = inst_id.trim();
    clean.equals_any_ignore_case(DEFAULT_IDS)
        || (clean.starts_with_any_ignore_case(&["#1"])
            && clean.has_any_word_ignore_case(&["default"]))
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
    if is_proj_default || is_inst_default {
        return is_proj_default && is_inst_default;
    }
    let clean_inst = inst_id.trim().trim_start_matches('#');
    let clean_proj = norm_proj_inst.trim().trim_start_matches('#');
    let inst_prefix = clean_inst.split(&['-', '_', ' '][..]).next().unwrap_or("");
    let proj_prefix = clean_proj.split(&['-', '_', ' '][..]).next().unwrap_or("");
    if !inst_prefix.is_empty() && inst_prefix == proj_prefix {
        return true;
    }
    false
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
    const UNTITLED_PREFIXES: &[&str] = &["untitled"];
    const NEW_CONV_NAMES: &[&str] = &["new conversation"];
    let is_untitled = title_clean.starts_with_any_ignore_case(UNTITLED_PREFIXES);
    let is_new_conv = title_clean.equals_any_ignore_case(NEW_CONV_NAMES);
    is_untitled || is_new_conv
}

/// Affirmative retention check when filtering conversations by only_running.
pub(crate) fn should_retain_unrunning_conversation(only_running: bool, is_running: bool) -> bool {
    if !only_running {
        return true;
    }
    is_running
}

pub(crate) fn is_queued_or_pending_status(status: &str) -> bool {
    const QUEUED_STATUSES: &[&str] = &[
        "queued",
        "queue",
        "pending",
        "enqueued_ide",
        "dispatched",
        "in_progress",
    ];
    status.equals_any_ignore_case(QUEUED_STATUSES)
}

/// Collect active prompts that belong on this project node: same instance +
/// path, empty path, or a session_id that matches an existing conversation.
pub(crate) fn collect_active_prompts_for_project(
    map: &HashMap<(String, String), Vec<ActivePrompt>>,
    inst: &str,
    path: &str,
    conv_nodes: &[AgmConversationNode],
) -> Vec<ActivePrompt> {
    let conv_ids: HashSet<&str> = conv_nodes
        .iter()
        .map(|c| c.conversation_id.as_str())
        .collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<ActivePrompt> = Vec::new();
    let inst_is_default = is_default_instance_match(inst);
    for ((ap_inst, ap_path), aps) in map {
        if !is_instance_compatible(ap_inst, inst, inst_is_default) {
            continue;
        }
        let path_ok = ap_path == path;
        for ap in aps {
            if !seen.insert(ap.id.clone()) {
                continue;
            }
            let sid = ap.session_id.as_deref().unwrap_or("");
            let session_ok = !sid.is_empty() && conv_ids.contains(sid);
            if path_ok || session_ok {
                out.push(ap.clone());
            }
        }
    }
    out
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
        .find(|i| owning_inst_id.equals_any(&[&i.id, &i.name]))
    {
        let is_inst_running =
            crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        if is_inst_running {
            return true;
        }
        let pids =
            crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
        if !pids.is_empty() {
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

            let mut conv_media_paths = Vec::new();
            let mut conv_media_thumbnails = Vec::new();
            for p in &item.prompts {
                for m in &p.media_paths {
                    if !conv_media_paths.contains(m) {
                        conv_media_paths.push(m.clone());
                    }
                }
                for t in &p.media_thumbnails {
                    if !conv_media_thumbnails.contains(t) {
                        conv_media_thumbnails.push(t.clone());
                    }
                }
            }

            let root_prompt = item
                .prompts
                .first()
                .map(|p| p.prompt_text.as_str())
                .unwrap_or(&item.prompt);

            conv_nodes.push(AgmConversationNode {
                seq_id: c_seq,
                seq_code: format!("C{:03}", c_seq),
                gitmap_seq_code: format!("GM:{}", short_id),
                conversation_id: item.cid.clone(),
                short_id,
                title: conversation_display_title(&item.title, root_prompt),
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
                media_paths: conv_media_paths
                    .iter()
                    .map(|path| workspace_relative_media(path, norm_path))
                    .collect(),
                media_thumbnails: conv_media_thumbnails
                    .iter()
                    .map(|path| workspace_relative_media(path, norm_path))
                    .collect(),
                prompts: {
                    let mut prompts = item.prompts.clone();
                    for turn in &mut prompts {
                        for path in &mut turn.media_paths {
                            *path = workspace_relative_media(path, norm_path);
                        }
                        for path in &mut turn.media_thumbnails {
                            *path = workspace_relative_media(path, norm_path);
                        }
                    }
                    prompts
                },
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
    let aps = collect_active_prompts_for_project(
        active_prompts_by_inst_and_path,
        norm_proj_inst,
        norm_path,
        conv_nodes,
    );
    for ap in &aps {
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
            const PENDING_STAGES: &[&str] = &["queued", "queue", "pending", "enqueued_ide"];
            const ACTIVE_STAGES: &[&str] = &["running", "dispatched", "in_flight", "in_progress"];
            let is_pending_stage = ap.status.has_any_word_ignore_case(PENDING_STAGES);
            let is_active_stage = ap.status.has_any_word_ignore_case(ACTIVE_STAGES);
            let is_relevant_status = is_pending_stage || is_active_stage;

            if is_relevant_status {
                let is_fresh = if is_pending_stage {
                    (now - ap.updated_at) <= 3600
                } else {
                    (now - ap.updated_at) <= 600
                };
                let is_alive_and_fresh = is_inst_alive && is_fresh;
                let is_queued = is_pending_stage;
                let is_ap_run = is_alive_and_fresh && is_active_stage;

                let (ap_media_paths, ap_media_thumbnails) =
                    if let Some(ref payload) = ap.image_payload {
                        let paths: Vec<String> = payload
                            .split(';')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|path| workspace_relative_media(path, norm_path))
                            .collect();
                        (paths.clone(), paths)
                    } else {
                        (Vec::new(), Vec::new())
                    };

                // For active running prompts, attach to existing transcript turn if found.
                // For queued prompts, NEVER merge into past completed turns — each queued prompt is an independent queued action!
                let existing_turn = if is_ap_run && !is_queued {
                    target_conv
                        .prompts
                        .iter_mut()
                        .find(|p| p.prompt_text.trim() == ap.prompt_content.trim())
                } else {
                    None
                };

                if let Some(turn) = existing_turn {
                    if !ap_media_paths.is_empty() {
                        for m in &ap_media_paths {
                            if !turn.media_paths.contains(m) {
                                turn.media_paths.push(m.clone());
                            }
                            if !target_conv.media_paths.contains(m) {
                                target_conv.media_paths.push(m.clone());
                            }
                        }
                        for t in &ap_media_thumbnails {
                            if !turn.media_thumbnails.contains(t) {
                                turn.media_thumbnails.push(t.clone());
                            }
                            if !target_conv.media_thumbnails.contains(t) {
                                target_conv.media_thumbnails.push(t.clone());
                            }
                        }
                    }
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
                        media_paths: ap_media_paths.clone(),
                        media_thumbnails: ap_media_thumbnails.clone(),
                    };
                    for m in &ap_media_paths {
                        if !target_conv.media_paths.contains(m) {
                            target_conv.media_paths.push(m.clone());
                        }
                    }
                    for t in &ap_media_thumbnails {
                        if !target_conv.media_thumbnails.contains(t) {
                            target_conv.media_thumbnails.push(t.clone());
                        }
                    }
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
        let (preview_200w, word_count) = extract_prompt_words_preview(&ap.prompt_content, word_cap);

        let title = extract_smart_prompt_summary(&ap.prompt_content, 60);
        let is_ghost = is_vacant_ghost_conversation(&title, &ap.prompt_content, word_count, true);
        if is_ghost && !is_queued_or_pending_status(&ap.status) {
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

        const RUNNING_STATUSES: &[&str] = &["running", "in_flight", "dispatched"];
        let is_ap_queued = is_queued_or_pending_status(&ap.status);
        let max_allowed_ttl = if has_active_worker { 300 } else { 45 };
        let is_running_status = ap.status.equals_any(RUNNING_STATUSES);
        let is_within_ttl = (now - ap.updated_at) <= max_allowed_ttl;
        let is_status_fresh = is_running_status && is_within_ttl;
        let is_executing = is_inst_alive && has_confirmed_running;
        let is_run = is_status_fresh && is_executing;
        let should_retain_ap =
            is_ap_queued || should_retain_unrunning_conversation(only_running, is_run);
        if !should_retain_ap {
            continue;
        }
        let c_seq = if let Some(ref conn) = conn_opt {
            ensure_conversation_sequence_in_conn(conn, &cid, project_key, &title, &ap.instance_id)
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

        let (ap_media_paths, ap_media_thumbnails) = if let Some(ref payload) = ap.image_payload {
            let paths: Vec<String> = payload
                .split(';')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|path| workspace_relative_media(path, norm_path))
                .collect();
            (paths.clone(), paths)
        } else {
            (Vec::new(), Vec::new())
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
            media_paths: ap_media_paths.clone(),
            media_thumbnails: ap_media_thumbnails.clone(),
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
            media_paths: ap_media_paths,
            media_thumbnails: ap_media_thumbnails,
            prompts: vec![ap_turn],
        });
    }

    // Live DevTools CDP In-Memory Queued Composer Message Harvesting:
    // If the instance process is active, query the IDE composer DOM via CDP to discover
    // any messages queued behind an active task ("Queued Messages N") that have not yet
    // been flushed to transcript.jsonl.
    let is_any_alive = is_inst_alive || crate::modules::process::is_antigravity_running(None);
    if is_any_alive {
        let cdp_queued = crate::modules::instance::extract_queued_composer_messages_for_instance(
            &proj.instance_id,
        );
        if !cdp_queued.is_empty() {
            // WHY index-first: chaining iter_mut().find() with
            // or_else(|| conv_nodes.first_mut()) borrows conv_nodes
            // twice (E0500), because the find result holds a mutable
            // borrow while the closure takes another. Resolving the
            // index under a shared borrow first, then indexing mutably,
            // keeps the exact selection (first running, else first,
            // else none on an empty list) with a single mutable borrow.
            let active_idx = conv_nodes
                .iter()
                .position(|c| c.is_running)
                .or_else(|| conv_nodes.first().map(|_| 0));
            if let Some(idx) = active_idx {
                let target_conv = &mut conv_nodes[idx];
                for q_text in cdp_queued {
                    let clean_text = q_text.trim();
                    if clean_text.is_empty() {
                        continue;
                    }
                    let already_exists = target_conv.prompts.iter().any(|p| {
                        let p_trimmed = p.prompt_text.trim();
                        p_trimmed == clean_text
                            || (clean_text.len() >= 20 && p_trimmed.starts_with(&clean_text[..20]))
                            || (p_trimmed.len() >= 20 && clean_text.starts_with(&p_trimmed[..20]))
                    });
                    if !already_exists {
                        let turn_num = target_conv.prompts.len() + 1;
                        let (preview_200w, _) = extract_prompt_words_preview(clean_text, word_cap);
                        target_conv.prompts.push(AgmPromptTurnNode {
                            turn_number: turn_num,
                            step_index: target_conv.prompts.len(),
                            prompt_text: clean_text.to_string(),
                            preview_200w,
                            status: "QUEUED".to_string(),
                            is_running: false,
                            is_queued: true,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            created_at: chrono::Utc::now().timestamp_millis(),
                            media_paths: Vec::new(),
                            media_thumbnails: Vec::new(),
                        });
                        target_conv.is_queued = true;
                    }
                }
            }
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
        assert!(is_default_instance_match("#1 Default"));
        assert!(is_default_instance_match("1 Default"));
        assert!(is_default_instance_match("#1 (Default)"));
        assert!(is_default_instance_match("default (active)"));
        assert!(!is_default_instance_match("default-copy"));
        assert!(!is_default_instance_match("default-copy-5597"));
        assert!(!is_default_instance_match("instance-1"));
        assert!(!is_default_instance_match("custom_inst"));
    }

    #[test]
    fn test_is_instance_compatible() {
        assert!(is_instance_compatible("default", "default", true));
        assert!(is_instance_compatible("__default__", "default", true));
        assert!(!is_instance_compatible("default-copy-1", "default", true));
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
    fn test_collect_active_prompts_matches_session_across_paths() {
        let conv = AgmConversationNode {
            seq_id: 1,
            seq_code: "C001".into(),
            gitmap_seq_code: "GM:abcd".into(),
            conversation_id: "conv-session-1".into(),
            short_id: "abcd".into(),
            title: "Work".into(),
            status: "IDLE".into(),
            is_running: false,
            step_count: 1,
            instance_id: "default".into(),
            instance_seq_num: None,
            instance_name: String::new(),
            instance_exe_name: String::new(),
            prompt_preview_200w: "hello".into(),
            prompt_tail_snippet: String::new(),
            prompt_word_count: 1,
            last_modified: "1".into(),
            byte_size: None,
            repeat_count: None,
            repeat_badge: None,
            sub_runs: Vec::new(),
            prompt_category: String::new(),
            is_queued: false,
            latest_step_summary: None,
            latest_response: None,
            execution_results: None,
            tool_calls_summary: None,
            full_prompt_text: None,
            media_paths: Vec::new(),
            media_thumbnails: Vec::new(),
            prompts: Vec::new(),
        };
        let mut map: HashMap<(String, String), Vec<ActivePrompt>> = HashMap::new();
        map.insert(
            ("default".into(), "/other/path".into()),
            vec![ActivePrompt {
                id: "queued-1".into(),
                project_id: "work".into(),
                instance_id: "default".into(),
                repo_path: "/other/path".into(),
                prompt_content: "enqueue me".into(),
                model: None,
                session_id: Some("conv-session-1".into()),
                status: "queued".into(),
                created_at: 1,
                updated_at: 1,
                image_payload: None,
            }],
        );
        let collected = collect_active_prompts_for_project(
            &map,
            "default",
            "/workspace/work",
            std::slice::from_ref(&conv),
        );
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].id, "queued-1");
        assert!(is_queued_or_pending_status("QUEUED"));
        assert!(is_queued_or_pending_status("queued"));
        assert!(is_queued_or_pending_status("pending"));
        assert!(is_queued_or_pending_status("enqueued_ide"));
        assert!(is_queued_or_pending_status("dispatched"));
        assert!(!is_queued_or_pending_status("running"));
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

    #[test]
    fn test_conversation_display_title() {
        assert_eq!(
            conversation_display_title("Untitled", "Fix login timeout"),
            "Fix login timeout"
        );
        assert_eq!(
            conversation_display_title("New Conversation", "Ship the release notes"),
            "Ship the release notes"
        );
        assert_eq!(
            conversation_display_title("Fix login bug", "some other prompt text"),
            "Fix login bug"
        );
    }

    #[test]
    fn test_workspace_relative_media() {
        assert_eq!(
            workspace_relative_media("C:/repo/assets/shot.png", "C:/repo"),
            "assets/shot.png"
        );
        assert_eq!(
            workspace_relative_media("D:/other/img.png", "C:/repo"),
            "file:///D:/other/img.png"
        );
        assert_eq!(
            workspace_relative_media("C:/repo-other/img.png", "C:/repo"),
            "file:///C:/repo-other/img.png"
        );
    }
}
