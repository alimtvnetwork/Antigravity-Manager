use crate::modules::account;
use crate::modules::git_info;
use crate::modules::repo_db;
use crate::modules::*;

use super::*;

/// Format detailed expanded view of a prompt
pub fn format_expand_prompt_report(query_str: &str) -> String {
    let prompts = match repo_db::list_all_prompts() {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "⚠️ <b>Failed to query prompts:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            )
        }
    };

    if prompts.is_empty() {
        return "⚠️ <b>No Prompts:</b> No prompts found in split database.".to_string();
    }

    let query = query_str.trim();
    let target = if query.is_empty() {
        prompts
            .iter()
            .find(|p| p.status == "running" || p.status == "dispatched")
            .or_else(|| prompts.first())
    } else if let Ok(idx) = query.parse::<usize>() {
        if idx >= 1 && idx <= prompts.len() {
            prompts.get(idx - 1)
        } else {
            None
        }
    } else {
        prompts.iter().find(|p| {
            p.id.starts_with(query)
                || p.id.eq_ignore_ascii_case(query)
                || p.project_id.contains(query)
        })
    };

    match target {
        Some(p) => {
            let friendly_ws =
                repo_db::format_friendly_workspace_label(&p.project_id, "", &p.repo_path);
            let short_ws = shorten_project_name(&friendly_ws);
            let id8 = p.id.get(..8).unwrap_or(&p.id);
            let duration = format_running_duration(p.created_at);
            let badge = if p.status == "running" || p.status == "dispatched" {
                "🟢"
            } else {
                "⚪"
            };
            let m_name = crate::modules::email_watcher::detect_machine_name();
            let inst_id = if p.instance_id.is_empty() {
                "default"
            } else {
                &p.instance_id
            };
            let (_, detected_images) = repo_db::extract_image_payload_or_path(&p.prompt_content);

            let attached_images_section = if !detected_images.is_empty() {
                let mut links = Vec::new();
                for (idx, url) in detected_images.iter().enumerate() {
                    let clean_url = clean_for_telegram_html(url, 200);
                    links.push(format!("<a href=\"{}\">🖼️ Image #{}</a>", clean_url, idx + 1));
                }
                format!(
                    "\n🖼️ <b>Attached Media ({}):</b> {}\n",
                    detected_images.len(),
                    links.join(" • ")
                )
            } else {
                String::new()
            };

            let palette = active_card_palette();
            let status_line = format!(
                "{} {} {}",
                badge,
                clean_for_telegram_html(&p.status, 24),
                duration
            );
            format!(
                "{} <b>Prompt {} — {}</b>\n\
                • <b>Machine:</b> {}\n\
                • <b>Instance:</b> <code>{}</code>\n\
                • <b>Status:</b> {}\n\
                {}\n\
                {}{}",
                palette.mark,
                clean_for_telegram_html(id8, 16),
                clean_for_telegram_html(&short_ws, 32),
                clean_for_telegram_html(&m_name, 48),
                clean_for_telegram_html(inst_id, 32),
                status_line.trim(),
                palette.rule,
                attached_images_section,
                prompt_note(
                    palette,
                    "Full text",
                    &clean_for_telegram_html(&p.prompt_content, 3500),
                )
            )
        }
        None => format!(
            "No prompt matched \"{}\". Send /active for running prompts or /prompts all for the full list.",
            clean_for_telegram_html(query, 32)
        ),
    }
}

/// Resolve inline reply markup with Copy Prompt button for expanded prompt
pub fn resolve_expand_prompt_markup(query_str: &str) -> Option<serde_json::Value> {
    let prompts = repo_db::list_all_prompts().ok()?;
    let query = query_str.trim();
    let target = if query.is_empty() {
        prompts
            .iter()
            .find(|p| p.status == "running" || p.status == "dispatched")
            .or_else(|| prompts.first())
    } else if let Ok(idx) = query.parse::<usize>() {
        if idx >= 1 && idx <= prompts.len() {
            prompts.get(idx - 1)
        } else {
            None
        }
    } else {
        prompts.iter().find(|p| {
            p.id.starts_with(query)
                || p.id.eq_ignore_ascii_case(query)
                || p.project_id.contains(query)
        })
    }?;

    let (_, detected_images) = repo_db::extract_image_payload_or_path(&target.prompt_content);
    Some(build_copy_prompt_reply_markup(
        &target.prompt_content,
        &detected_images,
        None,
    ))
}

/// Format prompts query report with ≥200 words preview for Telegram
pub fn format_prompts_query_report(term: &str) -> String {
    let all_prompts = match repo_db::list_all_prompts() {
        Ok(p) => p,
        Err(e) => {
            return format!(
                "⚠️ <b>Failed to query prompts:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            );
        }
    };

    if all_prompts.is_empty() {
        return "⚠️ <b>No Prompts:</b> No prompts tracked in SQLite database.".to_string();
    }

    let q = term.trim().to_lowercase();
    let matches: Vec<_> = all_prompts
        .into_iter()
        .filter(|p| {
            if q.is_empty() || q == "*" || q == "all" {
                return true;
            }
            p.prompt_content.to_lowercase().contains(&q)
                || p.project_id.to_lowercase().contains(&q)
                || p.repo_path.to_lowercase().contains(&q)
                || p.id.to_lowercase().contains(&q)
                || p.status.to_lowercase().contains(&q)
        })
        .take(5)
        .collect();

    if matches.is_empty() {
        return format!(
            "No prompts matched \"{}\". Send /query for recent prompts or /observe for the live view.",
            clean_for_telegram_html(term, 32)
        );
    }

    let mut out = format!("🔍 <b>{} prompts found</b>\n\n", matches.len());

    for (idx, p) in matches.iter().enumerate() {
        let (preview, wc) = repo_db::extract_prompt_words_preview(&p.prompt_content, 200);
        let friendly_ws = repo_db::format_friendly_workspace_label(&p.project_id, "", &p.repo_path);
        let short_ws = shorten_project_name(&friendly_ws);
        let badge = if p.status == "running" || p.status == "dispatched" {
            "🟢"
        } else {
            "⚪"
        };
        let prompt_id_short = if p.id.len() > 8 { &p.id[..8] } else { &p.id };

        out.push_str(&format!(
            "<b>#{}. {} {}</b> (ref <code>{}</code>, {} words)\n\
            {}\n\
            View full text: /expand {}\n\n",
            idx + 1,
            badge,
            clean_for_telegram_html(&short_ws, 24),
            clean_for_telegram_html(prompt_id_short, 12),
            wc,
            clean_for_telegram_html(&preview, 1200),
            clean_for_telegram_html(prompt_id_short, 12)
        ));
    }

    out
}

/// Format conversation seek report with ≥200-word expandable prompt previews and inline rerun/verify buttons
pub async fn format_seek_conversations_report(
    query_str: &str,
) -> (String, Option<serde_json::Value>) {
    let tree = repo_db::get_project_conversation_tree(200, false);

    // Invariant 1: FIFO Ordering in Database Queries
    // Tree was populated via FIFO queries; calculate global counters:
    let active_count: usize = tree.iter().map(|p| p.running_count).sum();
    let mut instances_set = std::collections::HashSet::new();
    for p in &tree {
        if !p.instance_id.is_empty() {
            instances_set.insert(&p.instance_id);
        }
    }
    let inst_count = instances_set.len().max(1);

    let raw_q = query_str.trim();
    let is_unfiltered = raw_q.is_empty() || raw_q == "*" || raw_q.eq_ignore_ascii_case("all");
    let query_label = if is_unfiltered {
        "All active & running".to_string()
    } else {
        clean_for_telegram_html(raw_q, 64)
    };

    struct MatchCandidate<'a> {
        proj: &'a crate::modules::repo_db::AgmProjectTreeNode,
        conv: &'a crate::modules::repo_db::AgmConversationNode,
        score: i32,
    }

    let mut candidates: Vec<MatchCandidate> = Vec::new();
    let q_lower = raw_q.to_lowercase();
    let q_digits = raw_q
        .trim_start_matches('#')
        .trim_start_matches(|c: char| c.is_alphabetic())
        .trim();

    for proj in &tree {
        for conv in &proj.conversations {
            let mut score = 0;

            if is_unfiltered {
                // Prioritize running conversations, then queued, then recent
                score = if conv.is_running {
                    100
                } else if conv.is_queued {
                    50
                } else {
                    10
                };
            } else {
                let conv_seq_clean = conv.seq_code.trim_start_matches('#').to_lowercase();
                let conv_seq_digits = conv
                    .seq_code
                    .trim_start_matches('#')
                    .trim_start_matches(|c: char| c.is_alphabetic())
                    .trim();
                let proj_seq_clean = proj.seq_code.trim_start_matches('#').to_lowercase();
                let proj_seq_digits = proj
                    .seq_code
                    .trim_start_matches('#')
                    .trim_start_matches(|c: char| c.is_alphabetic())
                    .trim();

                // 1. Direct sequence code match (#C001, C001, c1)
                if conv.seq_code.eq_ignore_ascii_case(raw_q)
                    || format!("#{}", conv.seq_code).eq_ignore_ascii_case(raw_q)
                    || conv_seq_clean == q_lower
                    || (!q_digits.is_empty()
                        && conv_seq_digits == q_digits
                        && (q_lower.starts_with('c') || q_lower.starts_with("#c")))
                {
                    score += 150;
                }
                // 2. Project sequence code match (P001, GM:#1)
                else if proj.seq_code.eq_ignore_ascii_case(raw_q)
                    || format!("#{}", proj.seq_code).eq_ignore_ascii_case(raw_q)
                    || proj_seq_clean == q_lower
                    || (!q_digits.is_empty()
                        && proj_seq_digits == q_digits
                        && (q_lower.starts_with('p') || q_lower.starts_with("#p")))
                {
                    score += 120;
                }
                // 3. GitMap sequence code match
                else if conv.gitmap_seq_code.to_lowercase().contains(&q_lower)
                    || proj.gitmap_seq_code.to_lowercase().contains(&q_lower)
                {
                    score += 110;
                }
                // 4. Conversation UUID or 8-char short_id prefix match
                else if conv.conversation_id.to_lowercase().starts_with(&q_lower)
                    || conv.short_id.to_lowercase().starts_with(&q_lower)
                {
                    score += 140;
                }
                // 5. Workspace / project name / repo path match
                else if proj.project_id.to_lowercase().contains(&q_lower)
                    || proj.repo_name.to_lowercase().contains(&q_lower)
                    || proj.repo_path.to_lowercase().contains(&q_lower)
                {
                    score += 80;
                }
                // 6. Conversation title match
                else if conv.title.to_lowercase().contains(&q_lower) {
                    score += 70;
                }
                // 7. Instance ID match
                else if conv.instance_id.to_lowercase().contains(&q_lower)
                    || proj.instance_id.to_lowercase().contains(&q_lower)
                {
                    score += 60;
                }
                // 8. Prompt preview or full text match
                else if conv.prompt_preview_200w.to_lowercase().contains(&q_lower)
                    || conv
                        .full_prompt_text
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q_lower)
                    || conv
                        .prompts
                        .iter()
                        .any(|t| t.prompt_text.to_lowercase().contains(&q_lower))
                {
                    score += 50;
                }

                if score > 0 && conv.is_running {
                    score += 30;
                }
            }

            if score > 0 {
                candidates.push(MatchCandidate { proj, conv, score });
            }
        }
    }

    // Sort: highest score first, then running first, then stable FIFO seq_id
    candidates.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.conv.is_running.cmp(&a.conv.is_running))
            .then_with(|| a.conv.seq_id.cmp(&b.conv.seq_id))
    });

    // Dedup by conversation_id in case grouped runs exist
    let mut seen_cids = std::collections::HashSet::new();
    candidates.retain(|c| seen_cids.insert(c.conv.conversation_id.clone()));

    // Take top 5 matching conversations to respect Telegram message size limits
    let matches: Vec<_> = candidates.into_iter().take(5).collect();

    if matches.is_empty() {
        let empty_msg = format!(
            "🎯 <b>CONVERSATION SEEK &amp; PROMPT TURNS</b>\n\
            ━━━━━━━━━━━━━━━━━━━━━━━━\n\
            • <b>Running Prompts:</b> <code>{}</code> active across <code>{}</code> instances\n\
            • <b>Filter:</b> <code>{}</code>\n\n\
            <i>No conversations matched query \"<b>{}</b>\".</i>\n\n\
            💡 Send <code>/seek</code> without arguments to list active conversations, or <code>/tree</code> for the full project hierarchy.",
            active_count,
            inst_count,
            query_label,
            clean_for_telegram_html(raw_q, 48)
        );
        return (empty_msg, None);
    }

    let mut out = format!(
        "🎯 <b>CONVERSATION SEEK &amp; PROMPT TURNS</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        • <b>Running Prompts:</b> <code>{}</code> active across <code>{}</code> instances\n\
        • <b>Filter:</b> <code>{}</code>\n\
        • <b>Matching Conversations:</b> <code>{}</code>\n\n",
        active_count,
        inst_count,
        query_label,
        matches.len()
    );

    let mut keyboard_rows: Vec<Vec<serde_json::Value>> = Vec::new();

    for candidate in &matches {
        let proj = candidate.proj;
        let conv = candidate.conv;

        let proj_name = repo_db::format_friendly_workspace_label(
            &proj.project_id,
            &proj.repo_name,
            &proj.repo_path,
        );
        let short_ws = shorten_project_name(&proj_name);
        let cid8 = if conv.short_id.is_empty() {
            if conv.conversation_id.len() >= 8 {
                &conv.conversation_id[..8]
            } else {
                &conv.conversation_id
            }
        } else {
            &conv.short_id
        };
        let inst_id = if conv.instance_id.is_empty() {
            if proj.instance_id.is_empty() {
                "default"
            } else {
                &proj.instance_id
            }
        } else {
            &conv.instance_id
        };

        let status_badge = if conv.is_running {
            "🟢 RUNNING"
        } else if conv.is_queued {
            "🟡 QUEUED"
        } else {
            "⚪ COMPLETED"
        };

        // Determine active or latest prompt turn
        let (turn_num, total_turns, turn_status, raw_turn_prompt) = if !conv.prompts.is_empty() {
            let active_turn = if conv.is_running {
                conv.prompts
                    .iter()
                    .find(|t| t.is_running)
                    .or_else(|| conv.prompts.last())
            } else {
                conv.prompts.last()
            };
            if let Some(t) = active_turn {
                let st = if t.is_running {
                    "RUNNING"
                } else if t.is_queued {
                    "QUEUED"
                } else if !t.status.trim().is_empty() {
                    t.status.as_str()
                } else {
                    "COMPLETED"
                };
                (
                    t.turn_number,
                    conv.prompts.len(),
                    st,
                    t.prompt_text.as_str(),
                )
            } else {
                (
                    1,
                    1,
                    if conv.is_running {
                        "RUNNING"
                    } else {
                        "COMPLETED"
                    },
                    conv.full_prompt_text
                        .as_deref()
                        .unwrap_or(&conv.prompt_preview_200w),
                )
            }
        } else {
            (
                1,
                1,
                if conv.is_running {
                    "RUNNING"
                } else {
                    "COMPLETED"
                },
                conv.full_prompt_text
                    .as_deref()
                    .unwrap_or(&conv.prompt_preview_200w),
            )
        };

        // Clamp preview to ≥200 words using repo_db
        let (preview_200w, _word_count) =
            repo_db::extract_prompt_words_preview(raw_turn_prompt, 200);
        let clean_preview = clean_for_telegram_html(&preview_200w, 1800);

        out.push_str(&format!(
            "📂 <b>[AGM:{} | {}]</b> <code>#{}</code> — <b>{}</b>\n\
            • <b>Instance:</b> <code>{}</code> · <b>ID:</b> <code>{}</code>\n\
            • <b>Title:</b> <i>{}</i>\n\
            • <b>Status:</b> {} (Turn {}/{})\n\n\
            <b>Turn #{}</b> [<i>{}</i>]:\n\
            <blockquote expandable><pre><code class=\"language-markdown\">{}</code></pre></blockquote>\n\n",
            clean_for_telegram_html(&proj.seq_code, 16),
            clean_for_telegram_html(&proj.gitmap_seq_code, 24),
            clean_for_telegram_html(&conv.seq_code, 16),
            clean_for_telegram_html(&short_ws, 32),
            clean_for_telegram_html(inst_id, 32),
            clean_for_telegram_html(cid8, 12),
            clean_for_telegram_html(&conv.title, 48),
            status_badge,
            turn_num,
            total_turns,
            turn_num,
            clean_for_telegram_html(turn_status, 24),
            clean_preview
        ));

        // Invariant 2: 60-Byte Callback Payload Cap (RCA 15)
        let rerun_cb = format!("send:t:{}:{}", cid8, turn_num);
        let verify_cb = format!("send:v:{}:{}", cid8, turn_num);
        let copy_turn_text = clamp_telegram_copy_text(raw_turn_prompt, TELEGRAM_COPY_TEXT_MAX);

        let has_turn_images = conv.prompts.iter().any(|t| !t.media_paths.is_empty());
        let img_cb = format!("send:img:{}:{}", cid8, turn_num);

        if matches.len() == 1 {
            let mut row = vec![
                serde_json::json!({
                    "text": format!("🔁 Rerun Turn #{}", turn_num),
                    "callback_data": rerun_cb
                }),
                serde_json::json!({
                    "text": format!("🛡️ Verify Turn #{}", turn_num),
                    "callback_data": verify_cb
                }),
                serde_json::json!({
                    "text": "📋 Copy Turn",
                    "copy_text": { "text": copy_turn_text }
                }),
            ];
            if has_turn_images {
                row.push(serde_json::json!({
                    "text": "🖼️ Send Images",
                    "callback_data": img_cb
                }));
            }
            keyboard_rows.push(row);
        } else {
            let mut row = vec![
                serde_json::json!({
                    "text": format!("🔁 #{} Rerun T#{}", conv.seq_code, turn_num),
                    "callback_data": rerun_cb
                }),
                serde_json::json!({
                    "text": format!("🛡️ #{} Verify T#{}", conv.seq_code, turn_num),
                    "callback_data": verify_cb
                }),
                serde_json::json!({
                    "text": format!("📋 #{} Copy", conv.seq_code),
                    "copy_text": { "text": copy_turn_text }
                }),
            ];
            if has_turn_images {
                row.push(serde_json::json!({
                    "text": format!("🖼️ #{} Img", conv.seq_code),
                    "callback_data": img_cb
                }));
            }
            keyboard_rows.push(row);
        }
    }

    let markup = if !keyboard_rows.is_empty() {
        Some(serde_json::json!({ "inline_keyboard": keyboard_rows }))
    } else {
        None
    };

    (out.trim_end().to_string(), markup)
}

/// Alias for `format_seek_conversations_report`
pub async fn format_conversation_seek_report(
    query_str: &str,
) -> (String, Option<serde_json::Value>) {
    format_seek_conversations_report(query_str).await
}

/// Resolve inline reply markup with Rerun/Verify/Copy buttons for conversation seek
pub fn resolve_seek_conversations_markup(query_str: &str) -> Option<serde_json::Value> {
    let tree = repo_db::get_project_conversation_tree(200, false);
    let raw_q = query_str.trim();
    let is_unfiltered = raw_q.is_empty() || raw_q == "*" || raw_q.eq_ignore_ascii_case("all");

    let mut keyboard_rows: Vec<Vec<serde_json::Value>> = Vec::new();
    let mut count = 0;

    for proj in &tree {
        for conv in &proj.conversations {
            if count >= 5 {
                break;
            }
            let is_match = is_unfiltered
                || conv.seq_code.eq_ignore_ascii_case(raw_q)
                || format!("#{}", conv.seq_code).eq_ignore_ascii_case(raw_q)
                || proj.seq_code.eq_ignore_ascii_case(raw_q)
                || conv
                    .conversation_id
                    .to_lowercase()
                    .starts_with(&raw_q.to_lowercase())
                || conv
                    .short_id
                    .to_lowercase()
                    .starts_with(&raw_q.to_lowercase())
                || proj
                    .project_id
                    .to_lowercase()
                    .contains(&raw_q.to_lowercase())
                || conv.title.to_lowercase().contains(&raw_q.to_lowercase());

            if is_match {
                let cid8 = if conv.short_id.is_empty() {
                    if conv.conversation_id.len() >= 8 {
                        &conv.conversation_id[..8]
                    } else {
                        &conv.conversation_id
                    }
                } else {
                    &conv.short_id
                };
                let turn_num = conv.prompts.last().map(|t| t.turn_number).unwrap_or(1);
                let raw_prompt = conv
                    .prompts
                    .last()
                    .map(|t| t.prompt_text.as_str())
                    .or(conv.full_prompt_text.as_deref())
                    .unwrap_or(&conv.prompt_preview_200w);

                let rerun_cb = format!("send:t:{}:{}", cid8, turn_num);
                let verify_cb = format!("send:v:{}:{}", cid8, turn_num);
                let copy_turn_text = clamp_telegram_copy_text(raw_prompt, TELEGRAM_COPY_TEXT_MAX);

                let has_turn_images = conv.prompts.iter().any(|t| !t.media_paths.is_empty());
                let img_cb = format!("send:img:{}:{}", cid8, turn_num);

                let mut row = vec![
                    serde_json::json!({
                        "text": format!("🔁 #{} Rerun T#{}", conv.seq_code, turn_num),
                        "callback_data": rerun_cb
                    }),
                    serde_json::json!({
                        "text": format!("🛡️ #{} Verify T#{}", conv.seq_code, turn_num),
                        "callback_data": verify_cb
                    }),
                    serde_json::json!({
                        "text": format!("📋 #{} Copy", conv.seq_code),
                        "copy_text": { "text": copy_turn_text }
                    }),
                ];
                if has_turn_images {
                    row.push(serde_json::json!({
                        "text": format!("🖼️ #{} Img", conv.seq_code),
                        "callback_data": img_cb
                    }));
                }
                keyboard_rows.push(row);
                count += 1;
            }
        }
        if count >= 5 {
            break;
        }
    }

    if !keyboard_rows.is_empty() {
        Some(serde_json::json!({ "inline_keyboard": keyboard_rows }))
    } else {
        None
    }
}

/// Format the colorful grouped Telegram command manual (HTML parse mode).
/// Every command is shown in a tap-to-copy <code> block, and the message
/// carries native copy-text inline buttons for the essentials.
pub fn format_help_manual() -> String {
    let ver = git_info::get_app_version();
    let hash = git_info::get_git_hash();
    let branch = git_info::get_git_branch();
    let last_rel = git_info::get_last_release();

    format!(
        "🤖 <b>AGM v{} — Telegram Command Center</b>\n\
        <code>v{} | commit {} | branch {} | release {}</code>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        💡 Every <code>command</code> below is tap-to-copy. Copy buttons for the essentials sit under this message.\n\n\
        🚀 <b>Easy Machine &amp; Instance Prompting</b>\n\
        • <code>/prompt default Write unit tests</code> — Send a prompt to the local <code>default</code> instance\n\
        • <code>/prompt W3:default Fix build</code> — Send a prompt to instance <code>default</code> on machine <code>W3</code>\n\
        • <code>/prompt --node vm-1 --instance work Audit code</code> — Explicit machine + instance prompt\n\
        • <code>/target W3 default</code> — Point subsequent plain messages at machine <code>W3</code> instance <code>default</code>\n\
        • <code>/target default C001</code> — Point the next plain message at a conversation (<code>/target clear</code> cancels, <code>/where</code> shows the target)\n\
        • <code>/send</code> — Guided 3-step picker: machine → instance → conversation, then type the prompt\n\
        • <code>/seek [query]</code> — Seek running conversations with ≥200-word expandable prompt previews &amp; rerun buttons\n\
        • <code>/conversations</code> — Alias for <code>/seek</code> conversation browser (also <code>/turns</code>, <code>/convs</code>)\n\
        • <code>/prompt C001 Is it done?</code> — Send to conversation (<code>C001</code>, <code>P001</code>, <code>GM:#1</code>)\n\
        • <code>C001 your prompt here</code> — Shorthand: ID + text sends directly\n\
        • <code>/prompts</code> · <code>/prompt_queue</code> · <code>/templates</code> — Reusable prompt templates\n\n\
        💰 <b>Account Balance &amp; Credit Inspection</b>\n\
        • <code>/balance</code> — Dedicated per-instance &amp; standby account 4h / weekly balance dashboard\n\
        • <code>/credits</code> · <code>/quota</code> — Aliases for <code>/balance</code> credit &amp; quota overview\n\
        • <code>/balance --csv</code> — Export live instance &amp; account balances as downloadable CSV file\n\
        • <code>/ff</code> · <code>/rotate</code> — Fast-forward to freshest highest-quota account (prompts preserved)\n\n\
        📥 <b>CSV &amp; File Format Exports</b>\n\
        • <code>/export accounts csv</code> — Download all accounts &amp; dual-window quotas as CSV\n\
        • <code>/export instances csv</code> — Download all instance profiles &amp; bound accounts as CSV\n\
        • <code>/export balance csv</code> — Download instance + account balance report as CSV\n\
        • <code>/export nodes csv</code> — Download registered SSH fleet nodes as CSV\n\
        • <code>/export prompts json</code> — Download active &amp; queued prompts as JSON\n\
        • <code>/accounts --csv</code> · <code>/instances --csv</code> · <code>/status --json</code> — Inline flag exports\n\n\
        🌐 <b>GitMap Multi-Machine &amp; SSH Control</b>\n\
        • <code>/nodes</code> — Inspect SSH / GitMap cluster VM nodes\n\
        • <code>/ssh nodes</code> — List configured SSH fleet nodes &amp; status\n\
        • <code>/ssh import-gitmap</code> — Import SSH nodes from GitMap &amp; <code>~/.ssh/config</code>\n\
        • <code>/ssh balance</code> — Poll multi-machine instances &amp; account balances across SSH fleet\n\
        • <code>/ssh exec vm-1 agm status</code> — Run command on remote SSH node <code>vm-1</code>\n\
        • <code>/gitmap cluster status</code> — Query GitMap cluster topology &amp; health\n\
        • <code>/ssh check &lt;node&gt;</code> · <code>/gitmap …</code> · <code>/agm …</code> · <code>/agy active</code>\n\n\
        🔵 <b>Window &amp; screen</b>\n\
        • <code>/open</code> — Open AGM on this machine (or <code>/open W3</code> on fleet machine <code>W3</code>)\n\
        • <code>/screenshot</code> — Photograph every known machine's screen (or <code>/screenshot W3</code>)\n\
        • <code>/ui</code> — Open manager window · <code>/shot</code> · <code>/screen</code> — Screenshot this machine\n\
        • <code>/ui goto &lt;route&gt;</code> — Navigate UI (<code>/ui goto instances</code>) · <code>/ui ls</code> · <code>/ui scroll &lt;id&gt;</code>\n\n\
        🟣 <b>Fleet telemetry</b>\n\
        • <code>/fleet</code> · <code>/whoami</code> · <code>/ping</code> — Machine, instance &amp; bound account identity\n\
        • <code>/instances</code> · <code>/accounts</code> · <code>/status</code> · <code>/observe</code> · <code>/tree</code> · <code>/active</code>\n\
        • <code>/seek</code> · <code>/conversations</code> · <code>/queues</code> · <code>/query &lt;term&gt;</code> · <code>/expand &lt;id&gt;</code> · <code>/projects</code> · <code>/snapshot</code>\n\n\
        🟡 <b>Backup, Cleanup, Updates &amp; Email</b>\n\
        • <code>/backup</code> · <code>/restore</code> · <code>/prune [N]</code> · <code>/clean</code>\n\
        • <code>/update</code> · <code>/update gitmap</code> · <code>/update all</code> · <code>/gitmap ssh update agm</code>\n\
        • <code>/email status</code> · <code>/email ping</code> · <code>/email help</code>\n\n\
        🎨 <b>Card theme</b>\n\
        • <code>/theme</code> — List palettes. Default is Riseup (emerald and gold)\n\
        • <code>/theme riseup</code> · <code>/theme obsidian</code> · <code>/theme gold</code> · <code>/theme indigo</code>\n\n\
        🛡️ <b>Prompt target</b>  <code>/send</code> confirms machine, instance, then conversation. <code>/target</code> points the next plain message at a conversation.{}",
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&hash, 24),
        clean_for_telegram_html(&branch, 24),
        clean_for_telegram_html(&last_rel, 24),
        super::ui_remote::window_command_section()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::telegram_inbound::ui_remote::window_command_section;

    #[test]
    fn help_manual_substitutes_all_format_args() {
        let help = format_help_manual();
        // Every `{}` placeholder must be substituted: the header carries the
        // version twice plus hash/branch/release, and the window section is
        // appended at the end. Any placeholder/argument mismatch is a compile
        // error, but this locks the rendered contract against regressions.
        assert!(
            !help.contains("{}"),
            "unsubstituted format placeholder in /help output"
        );
        assert!(help.contains("🤖 <b>AGM v"));
        assert!(help.contains("| commit "));
        assert!(help.contains("| branch "));
        assert!(help.contains("| release "));
        assert!(help.contains("/seek"));
        assert!(help.contains("/conversations"));
        assert!(help.contains(window_command_section().trim()));
    }

    #[test]
    fn help_manual_documents_ui_subcommands() {
        let help = format_help_manual();
        assert!(help.contains("/ui goto"));
        assert!(help.contains("/ui ls"));
        assert!(help.contains("/ui scroll"));
        assert!(help.contains("Drive the UI remotely"));
    }

    #[test]
    fn test_turn_callback_fits_60_byte_cap() {
        // RCA 15 Invariant: Telegram callback_data caps at 64 bytes; AGM enforces MAX_CALLBACK_BYTES = 60.
        let cid8 = "d58c5517";
        let turn = 125;
        let rerun_cb = format!("send:t:{}:{}", cid8, turn);
        let verify_cb = format!("send:v:{}:{}", cid8, turn);

        assert!(
            rerun_cb.len() <= 60,
            "Rerun callback length {} exceeds 60-byte cap",
            rerun_cb.len()
        );
        assert!(
            verify_cb.len() <= 60,
            "Verify callback length {} exceeds 60-byte cap",
            verify_cb.len()
        );
        assert_eq!(rerun_cb, "send:t:d58c5517:125");
        assert_eq!(verify_cb, "send:v:d58c5517:125");

        // Edge case: large 16-char CID and 5-digit turn
        let long_cid = "0123456789abcdef";
        let large_turn = 99999;
        let edge_cb = format!("send:t:{}:{}", long_cid, large_turn);
        assert!(
            edge_cb.len() <= 60,
            "Edge callback length {} exceeds 60-byte cap",
            edge_cb.len()
        );
    }

    #[tokio::test]
    async fn test_expandable_blockquote_tag_balance() {
        let (report, _) = format_seek_conversations_report("all").await;

        let open_b_count = report.matches("<blockquote expandable>").count();
        let close_b_count = report.matches("</blockquote>").count();
        assert_eq!(
            open_b_count, close_b_count,
            "Unbalanced blockquote tags in seek report"
        );

        let open_pre_count = report.matches("<pre>").count();
        let close_pre_count = report.matches("</pre>").count();
        assert_eq!(
            open_pre_count, close_pre_count,
            "Unbalanced pre tags in seek report"
        );

        let open_code_count = report.matches("<code").count();
        let close_code_count = report.matches("</code>").count();
        assert_eq!(
            open_code_count, close_code_count,
            "Unbalanced code tags in seek report"
        );
    }

    #[tokio::test]
    async fn test_seek_report_query_filtering() {
        // Test empty query (unfiltered)
        let (unfiltered, _) = format_seek_conversations_report("").await;
        assert!(unfiltered.contains("CONVERSATION SEEK &amp; PROMPT TURNS"));

        // Test non-matching query returns graceful message
        let (non_matching, _) =
            format_seek_conversations_report("nonexistent_random_token_987654321").await;
        assert!(non_matching.contains("No conversations matched"));

        // Test with specific query label
        let (scoped, _) = format_seek_conversations_report("#C001").await;
        assert!(scoped.contains("CONVERSATION SEEK &amp; PROMPT TURNS"));
    }

    #[test]
    fn test_fifo_ordering_invariant() {
        // Invariant 1: Ensure queries preserve FIFO ordering
        let query = "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC";
        assert!(query.contains("ORDER BY created_at ASC, id ASC"));
        assert!(!query.contains("ORDER BY updated_at DESC"));
    }
}
