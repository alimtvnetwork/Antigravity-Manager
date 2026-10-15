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
            let duration = format_running_duration(p.created_at);
            let badge = if p.status == "running" || p.status == "dispatched" {
                "🟢"
            } else {
                "⚪"
            };
            let m_name = crate::modules::email_watcher::detect_machine_name();
            let m_ip = crate::modules::email_watcher::detect_local_ip();
            let inst_id = if p.instance_id.is_empty() {
                "default"
            } else {
                &p.instance_id
            };
            let (_, detected_images) = repo_db::extract_image_payload_or_path(&p.prompt_content);
            let (_, word_count) = repo_db::extract_prompt_words_preview(&p.prompt_content, 200);

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

            format!(
                "📄 <b>Expanded Prompt: <code>{}</code></b> ⚡\n\
                ━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                🖥️ <b>Machine:</b> <code>{}</code> (<code>{}</code>) | 🏷️ <b>Instance:</b> <code>{}</code>\n\
                ━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                <b>📋 PROMPT METADATA &amp; CONTEXT</b>\n\
                <blockquote>\
                • <b>Workspace Project:</b> <b>{}</b>\n\
                • <b>Lifecycle Status:</b> {} <code>{}</code> ({})\n\
                • <b>Workspace Path:</b> <code>{}</code>\n\
                • <b>Word Count:</b> <code>{} words</code>\
                </blockquote>{}\n\
                <blockquote expandable><b>💬 Instructions:</b>\n\
                <code>{}</code></blockquote>",
                clean_for_telegram_html(&p.id, 48),
                clean_for_telegram_html(&m_name, 48),
                clean_for_telegram_html(&m_ip, 48),
                clean_for_telegram_html(inst_id, 32),
                clean_for_telegram_html(&short_ws, 32),
                badge,
                clean_for_telegram_html(&p.status, 24),
                duration,
                clean_for_telegram_html(&p.repo_path, 80),
                word_count,
                attached_images_section,
                clean_for_telegram_html(&p.prompt_content, 3500)
            )
        }
        None => format!(
            "⚠️ <b>Prompt Not Found:</b> No prompt matched <code>{}</code>.\n\
            💡 Send <code>/active</code> to view running prompts or <code>/prompts all</code> for all IDs.",
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
            "🔍 <b>Prompt Query:</b> No prompts matched <code>{}</code>.\n\
            💡 Send <code>/query</code> to view recent prompts or <code>/observe</code> for live state.",
            clean_for_telegram_html(term, 32)
        );
    }

    let mut out = format!(
        "🔍 <b>Prompts Query Results:</b> (Found {} matches)\n\n",
        matches.len()
    );

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
            "<b>#{}. {} {}</b> (<code>{}</code> | {} words)\n\
            {}\n\
            <i>Details: <code>/expand {}</code></i>\n\n",
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
        • <code>/prompt default Write unit tests</code> — Prompt local <code>default</code> instance directly\n\
        • <code>/prompt W3:default Fix build</code> — Prompt remote machine <code>W3</code> instance <code>default</code>\n\
        • <code>/prompt --node vm-1 --instance work Audit code</code> — Explicit machine + instance prompt\n\
        • <code>/target W3 default</code> — Aim subsequent plain messages at machine <code>W3</code> instance <code>default</code>\n\
        • <code>/target default C001</code> — Aim next plain message at a conversation (<code>/target clear</code> forgets, <code>/where</code> shows aim)\n\
        • <code>/send</code> — Guided 3-step picker: machine → instance → conversation, then type the prompt\n\
        • <code>/prompt C001 Is it done?</code> — Inject into conversation (<code>C001</code>, <code>P001</code>, <code>GM:#1</code>)\n\
        • <code>C001 your prompt here</code> — Shorthand: Seq ID + text injects directly\n\
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
        • <code>/queues</code> · <code>/query &lt;term&gt;</code> · <code>/expand &lt;id&gt;</code> · <code>/projects</code> · <code>/snapshot</code>\n\n\
        🟡 <b>Backup, Cleanup, Updates &amp; Email</b>\n\
        • <code>/backup</code> · <code>/restore</code> · <code>/prune [N]</code> · <code>/clean</code>\n\
        • <code>/update</code> · <code>/update gitmap</code> · <code>/update all</code> · <code>/gitmap ssh update agm</code>\n\
        • <code>/email status</code> · <code>/email ping</code> · <code>/email help</code>\n\n\
        <blockquote>🛡️ Prompts are never guessed: <code>/send</code> locks machine → instance → conversation first; <code>/target</code> aims the next plain message at one machine/instance/conversation.</blockquote>{}",
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
    use super::format_help_manual;
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
}
