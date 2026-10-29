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
    pub(crate) prompts: Vec<AgmPromptTurnNode>,
}

use super::gemini_dirs::gemini_dirs_tagged;
use super::models::{ActivePrompt, AgmPromptTurnNode, CascadeRunStatusType};
use super::project_queries::list_all_prompts;
use super::schema::connect_db;
use super::text_utils::{extract_smart_prompt_summary, StrExt};
use super::transcript::{extract_conversation_prompt_turns, inspect_conversation_transcript};
use super::tree_node_conv::*;
use crate::modules::repo_db::dispatch::parse_flexible_timestamp;
use crate::modules::repo_db::failed_commands::decode_uri_to_path;
use crate::modules::repo_db::project_queries::normalize_path_for_compare;
use crate::modules::repo_db::sequences::extract_prompt_words_preview;
use rusqlite::Connection;

#[allow(clippy::type_complexity)]
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
            let key = normalize_path_for_compare(&ap.repo_path);
            let inst = if is_default_instance_match(ap.instance_id.trim()) {
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
        if !summaries_db.exists() {
            continue;
        }

        let Ok(s_conn) = Connection::open_with_flags(
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
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        }) else {
            continue;
        };

        let registry = crate::modules::instance::registry::load_registry().unwrap_or_default();
        let is_owning_inst_alive = is_owning_instance_alive(owning_inst_id, &registry);

        // Justification: busy_timeout on a third-party read-only database is a nicety; reads continue with the default
        crate::error::record_ignored(
            s_conn.pragma_update(None, "busy_timeout", 3000),
            "set busy_timeout on Antigravity summaries database",
        );

        let Ok(mut stmt) = s_conn.prepare(
            "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time \
             FROM conversation_summaries \
             ORDER BY last_modified_time DESC \
             LIMIT 500",
        ) else {
            continue;
        };

        let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
            ))
        }) else {
            continue;
        };

        for item in rows.flatten() {
            let (cid, title, preview, status, not_fully_idle, ws_uris_opt, last_time_str) = item;

            let norm_owning_inst = if is_default_instance_match(owning_inst_id) {
                "default".to_string()
            } else {
                crate::modules::instance::resolve_instance_id(owning_inst_id)
                    .unwrap_or_else(|_| owning_inst_id.to_string())
            };
            if !seen_tree_cids.insert((norm_owning_inst.clone(), cid.clone())) {
                continue;
            }

            let now = chrono::Utc::now().timestamp();
            let conv_ts = parse_flexible_timestamp(&last_time_str);
            let is_recent = conv_ts > 0 && (now - conv_ts <= 600);

            let cascade_status = CascadeRunStatusType::from_status_str(&status);
            let is_explicit_idle = cascade_status.is_idle_or_terminal();

            let inspection = inspect_conversation_transcript(base, &cid);

            let is_terminal = is_explicit_idle || inspection.is_terminal_done;
            let is_live_active = inspection.is_recent_active && !inspection.is_terminal_done;
            let has_live_supremacy = is_owning_inst_alive && is_live_active;

            let has_activity_marker = not_fully_idle > 0 && is_recent;
            let is_active_fallback = is_owning_inst_alive && has_activity_marker;

            let is_conv_running = if has_live_supremacy {
                // Live transcript activity supremacy: if the instance process is alive
                // and the transcript was written within the last 15 seconds without a terminal done record,
                // the conversation is actively running in real life regardless of stale database status.
                true
            } else if is_terminal {
                // Explicit Idle: unreset not_fully_idle artifacts must not override terminal status
                false
            } else {
                is_active_fallback
            };
            let prompt_turns = extract_conversation_prompt_turns(base, &cid, is_conv_running);
            let steps = inspection.step_count;
            let mut effective_prompt = inspection
                .latest_prompt
                .filter(|s| !s.trim().is_empty())
                .or_else(|| prompt_turns.last().map(|t| t.prompt_text.clone()))
                .or_else(|| prompt_turns.first().map(|t| t.prompt_text.clone()))
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| preview.clone());

            let title_clean = title.trim();
            let title_lower = title_clean.to_lowercase();
            let is_placeholder_title =
                title_lower.starts_with("untitled") || title_lower == "new conversation";
            let has_meaningful_title = !title_clean.is_empty() && !is_placeholder_title;

            // Root human prompt prominence: derive conversation title from root initial turn
            // rather than collapsing into trailing subagents or short responses ("ok", "thanks")
            let root_human_prompt = prompt_turns
                .iter()
                .find(|t| !t.prompt_text.trim().is_empty())
                .map(|t| t.prompt_text.as_str())
                .or_else(|| prompt_turns.first().map(|t| t.prompt_text.as_str()));

            let resolved_title = if has_meaningful_title {
                title_clean.to_string()
            } else if let Some(root_p) = root_human_prompt {
                let summary = extract_smart_prompt_summary(root_p, 80);
                if !summary.trim().is_empty() {
                    summary
                } else {
                    title.clone()
                }
            } else if !preview.trim().is_empty() {
                let summary = extract_smart_prompt_summary(&preview, 80);
                if !summary.trim().is_empty() {
                    summary
                } else {
                    title.clone()
                }
            } else {
                title.clone()
            };

            let is_effective_prompt_empty = effective_prompt.trim().is_empty();
            if is_effective_prompt_empty && has_meaningful_title {
                effective_prompt = resolved_title.clone();
            }

            let has_prompt_content =
                !effective_prompt.trim().is_empty() || !prompt_turns.is_empty();

            // Non-prompt filter: skip internal background hooks or non-prompts if not running AND no meaningful title or prompts
            let should_retain = should_retain_prompt_candidate(
                inspection.is_non_prompt,
                is_conv_running,
                has_meaningful_title,
                has_prompt_content,
            );
            if !should_retain {
                continue;
            }

            let prompt_category = if inspection.is_subagent {
                "subagent".to_string()
            } else {
                const TITLE_KEYWORDS: &[&str] = &["subagent", "research", "worker", "author"];
                const PROMPT_KEYWORDS: &[&str] = &[
                    "you are research",
                    "you are worker",
                    "invoked by a caller agent",
                    "execute enhanced read memory",
                    "<subagent_reminder>",
                    "<system_message>",
                    "internal subagent",
                ];
                let is_sub = resolved_title.has_any_word_ignore_case(TITLE_KEYWORDS)
                    || effective_prompt.has_any_word_ignore_case(PROMPT_KEYWORDS);
                if is_sub {
                    "subagent".to_string()
                } else {
                    "user".to_string()
                }
            };

            const QUEUE_STATUSES: &[&str] = &["queued", "queue", "pending", "in_progress"];
            let is_queued = status.has_any_word_ignore_case(QUEUE_STATUSES);
            let latest_step_summary = inspection.latest_step_summary;

            // Ghost conversation filter: skip untitled / empty title with empty prompt AND zero turns
            let (_, eff_wc) = extract_prompt_words_preview(&effective_prompt, 5);
            let has_zero_turns = prompt_turns.is_empty();
            let is_ghost = is_vacant_ghost_conversation(
                &resolved_title,
                &effective_prompt,
                eff_wc,
                has_zero_turns,
            );
            if is_ghost {
                continue;
            }

            let mut assigned_paths: Vec<String> = Vec::new();
            if let Some(ws_raw) = ws_uris_opt {
                let uris: Vec<String> = serde_json::from_str(&ws_raw).unwrap_or_default();
                for u in uris {
                    let p = normalize_path_for_compare(&decode_uri_to_path(&u));
                    if !p.is_empty() && !assigned_paths.contains(&p) {
                        assigned_paths.push(p);
                    }
                }
            }

            // Prefix-length / valid path guard: if marked running with assigned paths but no decodable path >= 6 chars, force idle
            let has_assigned_paths = !assigned_paths.is_empty();
            let has_valid_prefix = assigned_paths.iter().any(|p| p.len() >= 6);
            let has_invalid_paths = has_assigned_paths && !has_valid_prefix;

            let is_conv_running = if is_conv_running && has_invalid_paths {
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
                        title: resolved_title.clone(),
                        prompt: effective_prompt.clone(),
                        status: if is_conv_running {
                            "RUNNING".to_string()
                        } else if is_queued {
                            "QUEUED".to_string()
                        } else if is_terminal || cascade_status.is_running() {
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
                        prompts: prompt_turns.clone(),
                    });
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
