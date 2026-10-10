use crate::modules::email_watcher;
use crate::modules::repo_db;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;
use crate::modules::*;
use serde_json::{json, Value};
use std::process::Command;

use super::*;

pub async fn format_node_scoped_prompts(args_str: &str) -> String {
    let trimmed = args_str.trim();
    let clean_str = if let Some(stripped) = trimmed.strip_suffix("running prompts") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_suffix("prompts") {
        stripped.trim()
    } else {
        trimmed
    };
    let parts: Vec<&str> = clean_str.split_whitespace().collect();
    let target_alias = if parts.is_empty() { "local" } else { parts[0] };

    let local_config = supabase_sync::load_config().unwrap_or_default();
    let local_alias = if local_config.node_alias.trim().is_empty() {
        email_watcher::detect_machine_name()
    } else {
        local_config.node_alias.clone()
    };

    let is_local = target_alias.eq_ignore_ascii_case("local")
        || target_alias.eq_ignore_ascii_case(&local_alias)
        || target_alias.eq_ignore_ascii_case(&email_watcher::detect_machine_name());

    if is_local {
        let projects = repo_db::get_live_project_execution_info();
        let prompts = repo_db::list_all_prompts().unwrap_or_default();
        let running_prompts: Vec<_> = prompts.iter().filter(|p| p.status == "running").collect();

        let mut proj_text = String::new();
        if projects.is_empty() {
            proj_text.push_str("• No registered workspaces\n");
        } else {
            for p in &projects {
                let badge = if p.is_running { "🟢" } else { "⚪" };
                let short_id = if p.project_id.len() > 12 {
                    &p.project_id[..12]
                } else {
                    &p.project_id
                };
                proj_text.push_str(&format!(
                    "• [{}] <b>{}</b> (ID: <code>{}</code>)\n  <code>{}</code>\n",
                    badge,
                    clean_for_telegram_html(&p.repo_name, 36),
                    short_id,
                    clean_for_telegram_html(&p.repo_path, 60)
                ));
            }
        }

        let mut prompt_text = String::new();
        if running_prompts.is_empty() {
            prompt_text.push_str("• No actively executing prompts (Node is idle)\n");
        } else {
            for p in &running_prompts {
                let short_pid = if p.id.len() > 10 { &p.id[..10] } else { &p.id };
                let clean_content = repo_db::extract_clean_user_prompt(&p.prompt_content);
                prompt_text.push_str(&format!(
                    "• [🟢] <code>{}</code> (Proj: <b>{}</b>)\n  <i>\"{}\"</i>\n",
                    short_pid,
                    clean_for_telegram_html(&p.project_id, 32),
                    clean_for_telegram_html(&clean_content, 120)
                ));
            }
        }

        format!(
            "⚡ <b>Node Telemetry &amp; Running Prompts: <code>{}</code></b>\n\n\
            📂 <b>Workspaces:</b>\n{}\n\
            📝 <b>Running Prompts ({} active):</b>\n{}\n\
            💡 Send <code>/prompt {} &lt;project-id&gt; &lt;text&gt;</code> to dispatch instructions.",
            clean_for_telegram_html(&local_alias, 48),
            proj_text,
            running_prompts.len(),
            prompt_text,
            clean_for_telegram_html(target_alias, 24)
        )
    } else {
        // Query remote VM node via GitMap cluster
        let mut remote_prompts_info = String::new();
        let cmd_out = Command::new("gitmap")
            .args(["cluster", "exec", target_alias, "agm wpr --json"])
            .output();

        let mut has_remote_data = false;
        if let Ok(out) = cmd_out {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.trim().is_empty() && (stdout.contains('[') || stdout.contains('{')) {
                has_remote_data = true;
                remote_prompts_info = format!(
                    "<pre>{}</pre>",
                    clean_for_telegram_html(stdout.trim(), 1500)
                );
            }
        }

        if !has_remote_data {
            // Check secondary DB queue for pending prompts for target node
            let mut queued_count = 0;
            for ep in &local_config.endpoints {
                if ep.is_enabled && ep.role == "secondary" {
                    if let Ok(c) = SupabaseClient::new(ep) {
                        let query = format!("target_node=eq.{}&status=eq.pending", target_alias);
                        if let Ok(val) = c.select("command_queue", &query).await {
                            if let Some(arr) = val.as_array() {
                                queued_count = arr.len();
                            }
                        }
                    }
                }
            }

            remote_prompts_info = format!(
                "• Remote Node Status: 🟢 Registered in Fleet\n\
                • Active Queued Prompts: <b>{}</b> pending in Supabase Secondary DB\n\
                • GitMap Cluster Link: Connected",
                queued_count
            );
        }

        format!(
            "⚡ <b>Remote VM Node Prompts: <code>{}</code></b>\n\n\
            {}\n\n\
            💡 Send <code>/prompt {} &lt;project-id&gt; &lt;text&gt;</code> to dispatch a prompt to this node.",
            clean_for_telegram_html(target_alias, 48),
            remote_prompts_info,
            clean_for_telegram_html(target_alias, 24)
        )
    }
}

/// Format discovered workspaces and project IDs (deduplicated by workspace repository path)
pub fn format_projects_list() -> String {
    let projects = repo_db::get_live_project_execution_info();
    if projects.is_empty() {
        return "📂 <b>Workspaces:</b> No registered workspaces found.".to_string();
    }

    // Deduplicate and group by canonical normalized workspace path (case-insensitive)
    let mut deduped_map: std::collections::HashMap<String, Vec<&repo_db::ProjectExecutionInfo>> =
        std::collections::HashMap::new();
    let mut path_order: Vec<String> = Vec::new();

    for p in &projects {
        let clean_path = p.repo_path.trim().replace('\\', "/").to_lowercase();
        let key = if clean_path.is_empty() {
            p.project_id.to_lowercase()
        } else {
            clean_path
        };
        if !deduped_map.contains_key(&key) {
            path_order.push(key.clone());
        }
        deduped_map.entry(key).or_default().push(p);
    }

    let mut rows = String::new();
    let mut display_idx = 0;

    for key in path_order {
        let entries = match deduped_map.get(&key) {
            Some(e) => e,
            None => continue,
        };

        let is_any_running = entries.iter().any(|p| p.is_running);
        let badge = if is_any_running { "🟢" } else { "⚪" };

        // Pick best representative entry (running one preferred, else latest)
        let rep = entries
            .iter()
            .find(|p| p.is_running)
            .unwrap_or_else(|| &entries[0]);

        let prompt_preview = entries
            .iter()
            .find_map(|p| p.active_prompt.as_ref())
            .map(|pr| {
                let clean = repo_db::extract_smart_prompt_summary(pr, 90);
                format!(
                    "\n   • <i>Prompt: \"{}\"</i>",
                    clean_for_telegram_html(&clean, 90)
                )
            })
            .unwrap_or_default();

        let friendly_label = repo_db::format_friendly_workspace_label(
            &rep.project_id,
            &rep.repo_name,
            &rep.repo_path,
        );

        let conv_summary = if entries.len() > 1 {
            let running_count = entries.iter().filter(|p| p.is_running).count();
            if running_count > 0 {
                format!(
                    " · <b>({} running, {} total convs)</b>",
                    running_count,
                    entries.len()
                )
            } else {
                format!(" · <b>({} convs)</b>", entries.len())
            }
        } else {
            String::new()
        };

        display_idx += 1;
        rows.push_str(&format!(
            "{}. {} <b>{}</b>{}\n   • <b>ID:</b> <code>{}</code>\n   • <b>Path:</b> <code>{}</code>{}\n\n",
            display_idx,
            badge,
            clean_for_telegram_html(&friendly_label, 48),
            conv_summary,
            clean_for_telegram_html(&rep.project_id, 48),
            clean_for_telegram_html(&rep.repo_path, 60),
            prompt_preview
        ));
    }

    format!(
        "📂 <b>Discovered Workspaces &amp; Projects ({} Unique Projects)</b>\n\n\
        {}\
        💡 <b>Sample Prompt Invocations:</b>\n\
        • <b>Local workspace:</b> <code>/prompt &lt;project-id&gt; &lt;prompt text&gt;</code>\n\
        • <b>Remote VM node:</b> <code>/prompt &lt;node-alias&gt; &lt;project-id&gt; &lt;prompt text&gt;</code>\n\
        • <b>Active conversation:</b> <code>/prompt &lt;conversation-id&gt; &lt;prompt text&gt;</code>\n\
        • <b>Prompt template:</b> <code>/prompt &lt;project-id&gt; read-all</code>",
        display_idx,
        rows
    )
}

/// Format active running prompts in compact GitMap style + AGM Tree View (Project → Conv → 200w Prompt)
pub async fn format_active_prompts_report() -> String {
    let agm_tree = repo_db::format_tree_view_telegram_html(200, true);

    if let Ok(out) = Command::new("gitmap").args(["agy", "active"]).output() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty()
            && (stdout.contains("Active Running Prompts") || stdout.contains("CONVERSATION ID"))
        {
            let cleaned = clean_for_telegram_html(stdout.trim(), 1800);
            return format!(
                "⚡ <b>Antigravity Active Running Prompts (GitMap)</b>\n\
                <pre>{}</pre>\n\n\
                {}",
                cleaned, agm_tree
            );
        }
    }

    agm_tree
}

/// Format workspace prompt queues in GitMap style
pub async fn format_prompt_queues_report() -> String {
    if let Ok(out) = Command::new("gitmap").args(["agy", "queues"]).output() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty()
            && (stdout.contains("Prompt Queues") || stdout.contains("queued"))
        {
            let cleaned = clean_for_telegram_html(stdout.trim(), 2800);
            return format!(
                "📥 <b>Antigravity Workspace Prompt Queues</b>\n\n\
                <pre>{}</pre>\n\n\
                💡 Send <code>/active</code> to view running prompts or <code>/restore</code> to re-queue backed-up prompts.",
                cleaned
            );
        }
    }

    match repo_db::list_all_prompts() {
        Ok(prompts) => {
            let queued: Vec<_> = prompts
                .into_iter()
                .filter(|p| p.status == "queued" || p.status == "pending")
                .collect();
            if queued.is_empty() {
                return "📥 <b>Prompt Queues:</b> All workspace queues are currently empty."
                    .to_string();
            }
            let mut rows = String::new();
            for (i, p) in queued.iter().take(10).enumerate() {
                let friendly_ws =
                    repo_db::format_friendly_workspace_label(&p.project_id, "", &p.repo_path);
                let clean = repo_db::extract_smart_prompt_summary(&p.prompt_content, 80);
                rows.push_str(&format!(
                    "{}. [⏳ QUEUED] <code>{}</code>\n   • <b>Project:</b> <b>{}</b> (<code>{}</code>)\n   • <i>\"{}\"</i>\n\n",
                    i + 1,
                    if p.id.len() > 12 { &p.id[..12] } else { &p.id },
                    clean_for_telegram_html(&friendly_ws, 36),
                    clean_for_telegram_html(&p.project_id, 32),
                    clean_for_telegram_html(&clean, 80)
                ));
            }
            format!(
                "📥 <b>Workspace Prompt Queues ({} Queued)</b>\n\n\
                {}\
                💡 Send <code>/restore</code> to resume prompt queue.",
                queued.len(),
                rows
            )
        }
        Err(e) => format!(
            "⚠️ <b>Queue Check Failed:</b> <code>{}</code>",
            clean_for_telegram_html(&e, 200)
        ),
    }
}
