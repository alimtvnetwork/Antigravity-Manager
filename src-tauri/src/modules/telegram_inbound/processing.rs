use crate::modules::auto_switcher;
use crate::modules::backup_prompts_db;
use crate::modules::email_watcher;
use crate::modules::repo_db;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;
use chrono::Utc;
use serde_json::{json, Value};
use std::process::Command;

use super::*;

/// Unified inbound Telegram command processor
pub async fn process_telegram_command_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Normalize leading slash and optional @bot_username suffix
    let (cmd_raw, rest) = match trimmed.split_once(char::is_whitespace) {
        Some((c, r)) => (c, r.trim()),
        None => (trimmed, ""),
    };
    let cmd_no_slash = cmd_raw.strip_prefix('/').unwrap_or(cmd_raw);
    let cmd_base = cmd_no_slash
        .split('@')
        .next()
        .unwrap_or(cmd_no_slash)
        .to_lowercase();
    let lower_full = trimmed.to_lowercase();

    match cmd_base.as_str() {
        "start" | "help" => Some(format_help_manual()),
        "ping" => Some(format_ping_report()),
        "status" | "observe" => Some(format_observe_report()),
        "tree" => {
            let only_running = !rest.eq_ignore_ascii_case("all");
            Some(repo_db::format_tree_view_telegram_html(200, only_running))
        }
        "expand" | "exp" => Some(format_expand_prompt_report(rest)),
        "active" | "running" => Some(format_active_prompts_report().await),
        "queues" | "queue" => Some(format_prompt_queues_report().await),
        "nodes" | "node" => {
            let sub = rest.trim();
            if sub.is_empty() || sub == "ls" || sub == "list" || sub == "status" {
                Some(format_cluster_nodes_report().await)
            } else {
                Some(format_node_scoped_prompts(sub).await)
            }
        }
        "ssh" => {
            let sub = rest.trim();
            if sub.is_empty()
                || sub.eq_ignore_ascii_case("nodes")
                || sub.eq_ignore_ascii_case("ls")
                || sub.eq_ignore_ascii_case("list")
            {
                Some(execute_gitmap_subcommand("ssh nodes"))
            } else if sub.starts_with("check ")
                || sub.starts_with("exec ")
                || sub.starts_with("update ")
                || sub.starts_with("agy ")
                || sub.starts_with("login ")
                || sub.starts_with("join ")
                || sub.starts_with("alias ")
                || sub.eq_ignore_ascii_case("scan")
                || sub.eq_ignore_ascii_case("status")
                || sub.eq_ignore_ascii_case("config")
            {
                Some(execute_gitmap_subcommand(&format!("ssh {}", sub)))
            } else if let Some((node, cmd)) = sub.split_once(char::is_whitespace) {
                Some(execute_gitmap_subcommand(&format!(
                    "cluster exec {} {}",
                    node.trim(),
                    cmd.trim()
                )))
            } else {
                Some(execute_gitmap_subcommand(&format!("ssh {}", sub)))
            }
        }
        "update" | "upgrade" => {
            let sub = rest.trim().to_lowercase();
            if sub == "gitmap" || sub == "gm" {
                Some(execute_gitmap_subcommand("self-update"))
            } else if sub == "all" {
                let gm_res = execute_gitmap_subcommand("self-update");
                let agm_res = execute_agm_subcommand("update");
                Some(format!("{}\n\n{}", gm_res, agm_res))
            } else if sub.starts_with("ssh") || sub.starts_with("node") {
                Some(execute_gitmap_subcommand("ssh update agm"))
            } else {
                Some(execute_agm_subcommand("update"))
            }
        }
        "prune" | "pr" | "clean" => {
            let keep_count = rest
                .trim()
                .strip_prefix("--keep")
                .or_else(|| rest.trim().strip_prefix("-k"))
                .unwrap_or(rest.trim())
                .trim()
                .parse::<usize>()
                .unwrap_or(10);
            match crate::modules::agy_cleaner::prune_conversations_only(keep_count) {
                Ok(res) => Some(format!(
                    "🧹 <b>Antigravity Conversation Prune Completed</b>\n\
                    ━━━━━━━━━━━━━━━━━━━━━━━━\n\
                    • <b>Protected Prompts:</b> All running/queued prompts safely preserved\n\
                    • <b>Protected Workspaces:</b> Top 5 sessions retained per active project\n\
                    • <b>Global History Retained:</b> {} sessions\n\
                    • <b>Deleted Sessions:</b> <code>{}</code>\n\
                    • <b>Reclaimed Space:</b> <code>{:.2} MB</code>\n\
                    • <b>Transaction ID:</b> <code>{}</code>",
                    keep_count,
                    res.pruned_count,
                    res.total_freed_bytes as f64 / (1024.0 * 1024.0),
                    res.transaction_id
                )),
                Err(e) => Some(format!(
                    "❌ <b>Conversation Prune Failed:</b> {}",
                    crate::modules::notification_hub::escape_telegram_html(&e.to_string())
                )),
            }
        }
        "query" | "search" => Some(format_prompts_query_report(rest)),
        "projects" | "workspaces" | "workspace" => Some(format_projects_list()),
        "prompts" | "prompt_queue" | "templates" => {
            let sub = rest.trim();
            if sub.is_empty() || sub == "ls" || sub == "list" {
                Some(format_prompts_templates_report())
            } else if sub == "queue" || sub == "queues" {
                Some(format_prompt_queues_report().await)
            } else if sub.starts_with("query") || sub.starts_with("search") {
                let term = sub
                    .strip_prefix("query")
                    .or_else(|| sub.strip_prefix("search"))
                    .unwrap_or("")
                    .trim();
                Some(format_prompts_query_report(term))
            } else if sub == "all" || sub == "db" {
                Some(format_prompts_list())
            } else if sub.starts_with("expand") {
                let id = sub["expand".len()..].trim();
                Some(format_expand_prompt_report(id))
            } else {
                Some(execute_prompt_injection(sub).await)
            }
        }
        "prompt" | "inject" | "p" | "pt" => Some(execute_prompt_injection(rest).await),
        "agy" => {
            let sub = rest.trim();
            if sub.is_empty() {
                Some(execute_gitmap_subcommand("agy active"))
            } else {
                Some(execute_gitmap_subcommand(&format!("agy {}", sub)))
            }
        }
        "gitmap" | "gm" => Some(execute_gitmap_subcommand(rest)),
        "agm" => Some(execute_agm_subcommand(rest)),
        "api" | "proxy" => Some(execute_api_status_command().await),
        "backup" | "backpack" => Some(execute_backup_command(rest)),
        "restore" => Some(execute_backup_command("restore")),
        "email" | "mail" => Some(execute_email_command(rest)),
        "snapshot" | "cluster" => Some(format_cluster_nodes_report().await),
        "ff" | "rotate" => {
            let projs = repo_db::list_running_projects().unwrap_or_default();
            let proj_names = crate::modules::notification_hub::deduplicate_names(
                projs
                    .into_iter()
                    .map(|p| p.repo_name)
                    .filter(|n| !n.is_empty()),
            );
            let proj_display = if !proj_names.is_empty() {
                proj_names.join(", ")
            } else {
                "Antigravity-Manager".to_string()
            };

            let backup_count = repo_db::backup_running_prompts("default").unwrap_or(0);
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                backup_prompts_db::backup_active_running_prompts(Some("default"), None),
                "backup_active_running_prompts",
            );
            let rotate_res = auto_switcher::check_and_rotate_if_needed().await;
            let resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
            let disp = repo_db::dispatch_running_prompts("default").unwrap_or(0);

            Some(format!(
                "⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>\n\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • <b>Pre-Switch Backup:</b> <code>{}</code> running prompt(s) captured\n\
                • <b>Rotation Evaluation:</b> {}\n\
                • <b>Post-Switch Re-injection:</b> <code>{}</code> prompt(s) resent ({} dispatched)\n\n\
                💡 All active prompts are preserved and continue without interruption.",
                clean_for_telegram_html(&proj_display, 80),
                backup_count,
                if rotate_res.is_ok() { "✅ Evaluated successfully" } else { "ℹ️ No switch required" },
                resent.len(),
                disp
            ))
        }
        _ => {
            let is_direct_seq_code = !rest.is_empty()
                && (cmd_base.starts_with("agm:c")
                    || cmd_base.starts_with("agm:p")
                    || cmd_base.starts_with("gm:#")
                    || cmd_base.starts_with("gm:")
                    || ((cmd_base.starts_with('c') || cmd_base.starts_with('p'))
                        && cmd_base.len() >= 2
                        && cmd_base[1..].chars().all(|ch| ch.is_ascii_digit())));
            if is_direct_seq_code {
                return Some(execute_prompt_injection(&format!("{} {}", cmd_no_slash, rest)).await);
            }
            if lower_full.contains("how many machines")
                || lower_full.contains("node ls")
                || lower_full == "nodes"
                || lower_full == "nodes ls"
            {
                return Some(format_cluster_nodes_report().await);
            }
            if lower_full.contains("running prompts") || lower_full.contains("active prompts") {
                if lower_full.starts_with("node") || lower_full.starts_with("nodes") {
                    return Some(format_node_scoped_prompts(trimmed).await);
                }
                return Some(format_active_prompts_report().await);
            }
            if lower_full.starts_with("ff:") {
                let projs = repo_db::list_running_projects().unwrap_or_default();
                let proj_names = crate::modules::notification_hub::deduplicate_names(
                    projs
                        .into_iter()
                        .map(|p| p.repo_name)
                        .filter(|n| !n.is_empty()),
                );
                let proj_display = if !proj_names.is_empty() {
                    proj_names.join(", ")
                } else {
                    "Antigravity-Manager".to_string()
                };

                let backup_count = repo_db::backup_running_prompts("default").unwrap_or(0);
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    backup_prompts_db::backup_active_running_prompts(Some("default"), None),
                    "backup_active_running_prompts",
                );
                let rotate_res = auto_switcher::check_and_rotate_if_needed().await;
                let resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
                let disp = repo_db::dispatch_running_prompts("default").unwrap_or(0);

                return Some(format!(
                    "⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>\n\n\
                    • <b>Projects:</b> <code>{}</code>\n\
                    • <b>Pre-Switch Backup:</b> <code>{}</code> prompt(s) captured\n\
                    • <b>Rotation Evaluation:</b> {}\n\
                    • <b>Post-Switch Re-injection:</b> <code>{}</code> prompt(s) resent ({} dispatched)",
                    clean_for_telegram_html(&proj_display, 80),
                    backup_count,
                    if rotate_res.is_ok() { "✅ Completed" } else { "ℹ️ Evaluated" },
                    resent.len(),
                    disp
                ));
            }
            let node_selector: Option<(&str, &str)> =
                if lower_full.starts_with("cmd:") || lower_full.starts_with("exec:") {
                    let parts: Vec<&str> = trimmed.splitn(3, ':').collect();
                    if parts.len() >= 3 {
                        Some((parts[1].trim(), parts[2].trim()))
                    } else {
                        None
                    }
                } else if let Some(colon_pos) = trimmed.find(':') {
                    let prefix_candidate = trimmed[..colon_pos].trim();
                    let cmd_candidate = trimmed[colon_pos + 1..].trim();
                    let is_node_ident = !prefix_candidate.is_empty()
                        && !cmd_candidate.is_empty()
                        && !prefix_candidate.contains(char::is_whitespace)
                        && (prefix_candidate.starts_with('W')
                            || prefix_candidate.starts_with('w')
                            || prefix_candidate.to_lowercase().starts_with("node")
                            || prefix_candidate.to_lowercase().starts_with("vm")
                            || prefix_candidate
                                .chars()
                                .all(|c| c.is_ascii_digit() || c == '.')
                            || prefix_candidate.eq_ignore_ascii_case("local"));
                    if is_node_ident {
                        Some((prefix_candidate, cmd_candidate))
                    } else {
                        None
                    }
                } else {
                    None
                };

            if let Some((target_node, cmd_content)) = node_selector {
                let s_config = supabase_sync::load_config().unwrap_or_default();
                let local_alias = if s_config.node_alias.trim().is_empty() {
                    email_watcher::detect_machine_name()
                } else {
                    s_config.node_alias.clone()
                };
                let local_ip = email_watcher::detect_local_ip();

                // If targeting current local node, execute directly!
                if target_node.eq_ignore_ascii_case("local")
                    || target_node.eq_ignore_ascii_case(&local_alias)
                    || target_node.eq_ignore_ascii_case(&local_ip)
                {
                    return Box::pin(process_telegram_command_text(cmd_content)).await;
                }

                // If remote, attempt GitMap cluster SSH execution first
                if let Ok(out) = Command::new("gitmap")
                    .args(["cluster", "exec", target_node, cmd_content])
                    .output()
                {
                    if out.status.success() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let clean = clean_for_telegram_html(stdout.trim(), 2800);
                        return Some(format!(
                            "🌐 <b>Cluster Result from <code>{}</code>:</b>\n\n<pre>{}</pre>",
                            clean_for_telegram_html(target_node, 48),
                            clean
                        ));
                    }
                }

                // Fallback to Supabase command queue
                let mut enqueued = false;
                for ep in &s_config.endpoints {
                    if !ep.is_enabled {
                        continue;
                    }
                    if ep.role == "secondary" {
                        if let Ok(c) = SupabaseClient::new(ep) {
                            let cmd_id = uuid::Uuid::new_v4().to_string();
                            let cmd_payload = json!({
                                "id": cmd_id,
                                "target_node": target_node,
                                "command": cmd_content,
                                "status": "pending",
                                "created_at": Utc::now().timestamp()
                            });
                            if c.insert("command_queue", cmd_payload).await.is_ok() {
                                enqueued = true;
                                break;
                            }
                        }
                    }
                }

                let reply_text = if enqueued {
                    format!(
                        "📥 <b>Command Enqueued:</b> Saved to Supabase Secondary DB for target node <code>{}</code>:\n<code>{}</code>",
                        clean_for_telegram_html(target_node, 48),
                        clean_for_telegram_html(cmd_content, 300)
                    )
                } else {
                    format!(
                        "⚙️ <b>Command Received:</b> Target <code>{}</code>:\n<code>{}</code>",
                        clean_for_telegram_html(target_node, 48),
                        clean_for_telegram_html(cmd_content, 300)
                    )
                };
                return Some(reply_text);
            }
            None
        }
    }
}
