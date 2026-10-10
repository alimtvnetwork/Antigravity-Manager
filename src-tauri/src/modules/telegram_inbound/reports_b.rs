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

            format!(
                "📄 <b>Expanded Prompt Details:</b>\n\n\
                • <b>ID:</b> <code>{}</code>\n\
                • <b>Project:</b> <b>{}</b>\n\
                • <b>Status:</b> {} {}\n\
                • <b>Path:</b> <code>{}</code>\n\n\
                <b>Full Prompt Instructions:</b>\n{}\n",
                clean_for_telegram_html(&p.id, 48),
                clean_for_telegram_html(&short_ws, 32),
                badge,
                duration,
                clean_for_telegram_html(&p.repo_path, 80),
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

/// Format comprehensive Telegram command manual
pub fn format_help_manual() -> String {
    let ver = git_info::get_app_version();
    let hash = git_info::get_git_hash();
    let branch = git_info::get_git_branch();
    let last_rel = git_info::get_last_release();

    format!(
        "🤖 <b>AGM v{} Telegram Remote Manual</b>\n\
        <code>v{} | commit {} | branch {} | release {}</code>\n\n\
        📌 <b>Core Telemetry &amp; Dual-Sequence Tree View (AGM + GitMap):</b>\n\
        • <code>/help</code> or <code>/start</code> — Display this full interactive command manual\n\
        • <code>/ping</code> — Check node connectivity, IP, Git build &amp; uptime\n\
        • <code>/tree</code> — Running Project → Conversation → [≤200w Prompt] tree (<code>[AGM:P001 | GM:#1]</code>, <code>[AGM:C001 | GM:&lt;cid&gt;]</code>)\n\
        • <code>/tree all</code> — Full tree of all workspaces &amp; conversations across all instances\n\
        • <code>/active</code> or <code>/running</code> — Active running prompts + Dual-Sequence Tree View\n\
        • <code>/status</code> or <code>/observe</code> — Live workspaces, active account quota &amp; prompts\n\
        • <code>/query &lt;term&gt;</code> or <code>/prompts query</code> — Search SQLite cached prompts (≥200w preview)\n\
        • <code>/expand &lt;id&gt;</code> — View full untruncated prompt instructions\n\
        • <code>/snapshot</code> — Multi-node cluster status snapshot\n\
        • <code>/projects</code> — List registered workspaces, AGM/GitMap Seq IDs &amp; sample syntax\n\
        • <code>/queues</code> — Inspect pending workspace prompt queues\n\n\
        🎯 <b>Prompt Injection (By Dual Seq ID, Instance &amp; Remote Machine):</b>\n\
        • <code>/prompt C001 Is it done?</code> — Target specific conversation sequence <code>C001</code> (or <code>GM:&lt;cid&gt;</code>)\n\
        • <code>/prompt P001 Run cargo clippy</code> — Target project sequence <code>P001</code> (or <code>GM:#1</code>)\n\
        • <code>/prompt C001 --instance #2 Check status</code> — Target specific instance (<code>#1</code>, <code>#2</code>, <code>default</code>) &amp; conv\n\
        • <code>/prompt C001 --instance #2 --node worker-1 Fix test</code> — Target remote machine + instance + conv\n\
        • <code>/prompts</code> — List reusable prompt templates (<code>read-all</code>, <code>is-done</code>, <code>ci-cd-fix</code>)\n\
        • <code>/agy prompt -n read-all -t \"Read memory and continue\"</code> — Dispatch template via GitMap AGY\n\
        • <code>/agy prompt -n is-done -t \"Verify if all tasks are complete\"</code> — Dispatch verification prompt\n\
        • <code>/agy prompt-project P001 -n is-done -t \"Check build\"</code> — Target specific project with template\n\
        • <code>/agy prompt-txt \"Quick hotfix instruction\"</code> — Dispatch direct raw prompt text\n\
        • <code>/agy prompt ls</code> — List all prompt templates in formatted table\n\
        • <code>/agy fpug</code> — Loop projects until prompt queues clear &amp; CI/CD green\n\
        • <code>/agy sug</code> — Monitor projects &amp; trigger OS shutdown when green\n\
        • <code>/agy rerun 1</code> — Restart IDE &amp; replay prompt + media + queued checks\n\n\
        🎒 <b>AGY Running Storage Backup &amp; Restore:</b>\n\
        • <code>/backup</code> or <code>/backpack</code> — Snapshot running prompts to AGM split SQLite DB\n\
        • <code>/backup ls</code> — List saved AGM prompt backup batches\n\
        • <code>/restore</code> — Restore &amp; resume backed-up AGM prompts\n\
        • <code>/gitmap backup-running-prompts</code> — GitMap snapshot of active AGY storage prompts\n\
        • <code>/gitmap restore-running-prompts</code> — GitMap restore &amp; re-inject backed-up prompts\n\
        • <code>/agy running-prompts ls</code> — List live AGY running prompts via GitMap\n\
        • <code>/agy running-prompts backup</code> — Backup active AGY storage prompts via GitMap\n\
        • <code>/agy running-prompts restore</code> — Restore active AGY storage prompts via GitMap\n\
        • <code>/agy ccko</code> — Clean runtime cache keeping only 1 conversation\n\
        • <code>/agy cckf</code> — Clean runtime cache keeping top 5 conversations\n\
        • <code>/agy cc --keep 10</code> — Clean runtime cache and prune conversation history\n\
        • <code>/prune [N]</code> or <code>/pr [N]</code> — Safely prune conversations (default: keep 10, guards active prompts &amp; ≥5 sessions)\n\
        • <code>/clean</code> — Clean application runtime and build caches\n\n\
        🔄 <b>AGM &amp; GitMap Update Commands:</b>\n\
        • <code>/update</code> or <code>/update agm</code> — Self-update Antigravity-Manager (delegated updater)\n\
        • <code>/update gitmap</code> — Update GitMap CLI to latest release (<code>gitmap self-update</code>)\n\
        • <code>/update all</code> — Concurrently update both AGM and GitMap CLI\n\
        • <code>/agm update</code> — Run AGM CLI update checker &amp; installer\n\
        • <code>/gitmap agm update -y</code> — Update AGM via GitMap installer pipeline\n\
        • <code>/gitmap agm update-all</code> — Distribute AGM update across all SSH nodes\n\
        • <code>/gitmap ssh update agm</code> — Update AGM across all SSH cluster machines\n\n\
        🖥️ <b>SSH &amp; Multi-Node Fleet Execution:</b>\n\
        • <code>/nodes</code> or <code>/ssh nodes</code> — List all registered SSH / cluster VM nodes\n\
        • <code>/ssh check &lt;node&gt;</code> — Probe connectivity, port 22, and health of remote machine\n\
        • <code>/ssh &lt;node&gt; &lt;cmd&gt;</code> — Execute command on specific machine via GitMap SSH (e.g. <code>/ssh vm-01 agm status</code>)\n\
        • <code>/ssh exec \"&lt;cmd&gt;\"</code> — Run command across all remote SSH fleet nodes\n\
        • <code>/ssh update agm</code> — Update AGM across all remote SSH nodes\n\
        • <code>/ssh agy active</code> — Run <code>gitmap agy active</code> across remote SSH fleet\n\
        • <code>/gitmap ssh nodes</code> — Inspect GitMap SSH node inventory &amp; reachability\n\
        • <code>CMD:&lt;node-alias&gt;:&lt;command&gt;</code> — Direct node command routing\n\n\
        🧭 <b>GitMap, AGM &amp; Multi-Instance Rotation:</b>\n
        • <code>/gitmap pe</code> — Check CI/CD pipeline execution status\n\
        • <code>/agy active</code> or <code>/gitmap agy active</code> — Check Antigravity active prompts via GitMap\n\
        • <code>/agm tree</code> / <code>/agm wpr</code> / <code>/agm accounts</code> — Run AGM CLI views\n\
        • <code>/api</code> — Inspect local API proxy (port 8045) &amp; account bindings\n\
        • <code>/ff</code> — Fast-forward switch to freshest highest-quota account\n\
        • <code>/email status</code> | <code>/email ping</code> | <code>/email help</code> — Email notifications",
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&hash, 24),
        clean_for_telegram_html(&branch, 24),
        clean_for_telegram_html(&last_rel, 24)
    )
}
