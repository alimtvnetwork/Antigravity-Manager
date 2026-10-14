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
                    links.join(" &bull; ")
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
        🟢 <b>Send prompts</b>\n\
        • <code>/send</code> — Guided send: pick machine → instance → conversation, then type the prompt. The destination is always explicit, never guessed\n\
        • <code>/prompt C001 Is it done?</code> — Inject into a conversation (<code>C001</code>, <code>P001</code>, <code>GM:#1</code>, <code>GM:&lt;cid&gt;</code>)\n\
        • <code>/prompt C001 --instance #2 --node W3 Fix it</code> — Target remote machine + instance + conversation\n\
        • <code>C001 your prompt here</code> — Shorthand: Seq ID + text injects directly\n\
        • <code>/inject …</code> · <code>/p …</code> · <code>/pt …</code> — Aliases of <code>/prompt</code>\n\
        • <code>/prompts</code> · <code>/prompt_queue</code> · <code>/templates</code> — Reusable prompt templates (<code>read-all</code>, <code>is-done</code>, <code>ci-cd-fix</code>)\n\
        • <code>/target default C001</code> — Aim the next plain message at a conversation (<code>/target clear</code> forgets, <code>/where</code> shows the aim)\n\n\
        🔵 <b>Window &amp; screen</b>\n\
        • <code>/open</code> — Open AGM on this machine (already running → brought to the foreground)\n\
        • <code>/open W3</code> — Open AGM on fleet machine <code>W3</code> (SSH, Supabase queue fallback)\n\
        • <code>/screenshot</code> — Photograph every known machine's screen\n\
        • <code>/screenshot W3</code> — Photograph one machine (<code>local</code> = this machine)\n\
        • <code>/ui</code> — Open the manager window here · <code>/shot</code> · <code>/screen</code> — Screenshot this machine\n\n\
        🟣 <b>Fleet telemetry</b>\n\
        • <code>/fleet</code> · <code>/whoami</code> — Machine, instance &amp; bound account identity card\n\
        • <code>/ping</code> — Node connectivity, IP, Git build &amp; uptime\n\
        • <code>/instances</code> · <code>/inst</code> — Instance profiles &amp; live process states\n\
        • <code>/accounts</code> · <code>/acc</code> — Registered accounts, tiers &amp; quota headroom\n\
        • <code>/status</code> · <code>/observe</code> — Live workspaces, active account quota &amp; prompts\n\
        • <code>/tree</code> — Project → conversation → prompt tree (<code>/tree all</code> for everything)\n\
        • <code>/active</code> · <code>/running</code> — Running prompts + dual-sequence tree view\n\
        • <code>/queues</code> · <code>/queue</code> — Pending workspace prompt queues\n\
        • <code>/query &lt;term&gt;</code> · <code>/search &lt;term&gt;</code> — Search cached prompts (200-word preview)\n\
        • <code>/expand &lt;id&gt;</code> · <code>/exp &lt;id&gt;</code> — Full untruncated prompt instructions\n\
        • <code>/projects</code> · <code>/workspaces</code> — Registered workspaces, AGM/GitMap Seq IDs &amp; syntax\n\
        • <code>/snapshot</code> · <code>/cluster</code> — Multi-node cluster status snapshot\n\
        • <code>/nodes</code> · <code>/node</code> — SSH / cluster VM nodes (<code>/nodes &lt;alias&gt; prompts</code> inspects a node)\n\n\
        🟠 <b>Fleet execution</b>\n\
        • <code>/ssh &lt;node&gt; &lt;cmd&gt;</code> — Run a command on one machine (e.g. <code>/ssh vm-01 agm status</code>)\n\
        • <code>/ssh check &lt;node&gt;</code> — Probe connectivity &amp; health · <code>/ssh nodes</code> — Node inventory\n\
        • <code>/ssh exec \"&lt;cmd&gt;\"</code> — Run across all remote SSH fleet nodes\n\
        • <code>CMD:&lt;node&gt;:&lt;command&gt;</code> — Direct node routing · <code>W3: agm status</code> — Shorthand\n\
        • <code>/gitmap …</code> · <code>/gm …</code> — Any GitMap CLI (<code>/gitmap pe</code> = pipeline status)\n\
        • <code>/agm …</code> — Any AGM CLI (<code>/agm tree</code>, <code>/agm wpr</code>, <code>/agm accounts</code>)\n\
        • <code>/agy active</code> — Antigravity active prompts via GitMap\n\
        • <code>/agy prompt -n read-all -t \"…\"</code> — Dispatch a prompt template\n\
        • <code>/agy prompt-project P001 -n is-done -t \"Check build\"</code> — Template at a specific project\n\
        • <code>/agy prompt-txt \"Quick hotfix\"</code> — Dispatch raw prompt text\n\
        • <code>/agy prompt ls</code> — List templates · <code>/agy fpug</code> — Loop until queues clear &amp; CI green\n\
        • <code>/agy sug</code> — Watch &amp; shut down when green · <code>/agy rerun 1</code> — Restart IDE &amp; replay\n\
        • <code>/api</code> · <code>/proxy</code> — Local API proxy (port 8045) &amp; account bindings\n\n\
        🟡 <b>Backup &amp; cleanup</b>\n\
        • <code>/backup</code> · <code>/backpack</code> — Snapshot running prompts (<code>/backup ls</code> lists batches)\n\
        • <code>/restore</code> — Restore &amp; resume backed-up prompts\n\
        • <code>/prune [N]</code> · <code>/pr [N]</code> — Prune conversations, keep N (default 10; guards active prompts)\n\
        • <code>/clean</code> — Clean runtime &amp; build caches\n\
        • <code>/agy ccko</code> — Keep 1 conversation · <code>/agy cckf</code> — Keep top 5 · <code>/agy cc --keep 10</code> — Prune history\n\
        • <code>/agy running-prompts ls</code> · <code>backup</code> · <code>restore</code> — AGY storage via GitMap\n\n\
        🔴 <b>Updates</b>\n\
        • <code>/update</code> · <code>/update agm</code> · <code>/upgrade</code> — Self-update Antigravity-Manager\n\
        • <code>/update gitmap</code> — Update GitMap CLI · <code>/update all</code> — Update both\n\
        • <code>/gitmap ssh update agm</code> — Update AGM across all SSH nodes\n\n\
        📧 <b>Email</b>\n\
        • <code>/email status</code> · <code>/email ping</code> · <code>/email help</code> — Vault status, test ping, full guide by email (<code>/mail</code> alias)\n\n\
        ⚡ <b>Fast tools</b>\n\
        • <code>/ff</code> · <code>/rotate</code> — Fast-forward to the freshest highest-quota account (prompts preserved)\n\n\
        <blockquote>🛡️ Prompts are never guessed: <code>/send</code> locks machine → instance → conversation first; <code>/target</code> aims the next plain message at one conversation.</blockquote>{}",
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&hash, 24),
        clean_for_telegram_html(&branch, 24),
        clean_for_telegram_html(&last_rel, 24),
        super::ui_remote::window_command_section()
    )
}
