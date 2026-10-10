use crate::modules::account;
use crate::modules::backup_prompts_db;
use crate::modules::email_sender;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::repo_db;
use chrono::Utc;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use super::*;

/// Execute GitMap CLI command and format output for Telegram
pub fn execute_gitmap_subcommand(args_str: &str) -> String {
    let trimmed = args_str.trim();
    let parts: Vec<&str> = if trimmed.is_empty() {
        vec!["pe"]
    } else {
        trimmed.split_whitespace().collect()
    };

    let output_res = Command::new("gitmap").args(&parts).output();
    match output_res {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = if stderr.trim().is_empty() {
                stdout.to_string()
            } else if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                format!("{}\n{}", stdout.trim_end(), stderr.trim_end())
            };
            let cleaned = clean_for_telegram_html(combined.trim(), 3200);
            format!(
                "🧭 <b>GitMap CLI:</b> <code>gitmap {}</code>\n<pre>{}</pre>",
                clean_for_telegram_html(&parts.join(" "), 64),
                if cleaned.is_empty() {
                    "(completed with no output)".to_string()
                } else {
                    cleaned
                }
            )
        }
        Err(e) => format!(
            "⚠️ <b>GitMap Execution Error:</b> <code>{}</code>",
            clean_for_telegram_html(&e.to_string(), 300)
        ),
    }
}

/// Resolve the path to the `agm` CLI binary
pub(crate) fn resolve_agm_binary() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(stem) = exe.file_stem() {
            if stem.to_string_lossy().eq_ignore_ascii_case("agm") {
                return exe;
            }
        }
        if let Some(parent) = exe.parent() {
            let candidate = if cfg!(windows) {
                parent.join("agm.exe")
            } else {
                parent.join("agm")
            };
            if candidate.exists() {
                return candidate;
            }
        }
    }
    let debug_candidate = PathBuf::from("src-tauri/target/debug/agm.exe");
    if debug_candidate.exists() {
        return debug_candidate;
    }
    PathBuf::from("agm")
}

/// Execute AGM CLI command and format output for Telegram
pub fn execute_agm_subcommand(args_str: &str) -> String {
    let trimmed = args_str.trim();
    let parts: Vec<&str> = if trimmed.is_empty() {
        vec!["status"]
    } else {
        trimmed.split_whitespace().collect()
    };

    // Prevent recursive telegram polling
    if parts
        .first()
        .map(|s| s.eq_ignore_ascii_case("telegram"))
        .unwrap_or(false)
    {
        return format_observe_report();
    }

    let bin = resolve_agm_binary();
    match Command::new(&bin).args(&parts).output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = if stderr.trim().is_empty() {
                stdout.to_string()
            } else if stdout.trim().is_empty() {
                stderr.to_string()
            } else {
                format!("{}\n{}", stdout.trim_end(), stderr.trim_end())
            };
            let cleaned = clean_for_telegram_html(combined.trim(), 3200);
            format!(
                "⚡ <b>AGM CLI:</b> <code>agm {}</code>\n<pre>{}</pre>",
                clean_for_telegram_html(&parts.join(" "), 64),
                if cleaned.is_empty() {
                    "(completed with no output)".to_string()
                } else {
                    cleaned
                }
            )
        }
        Err(e) => format!(
            "⚠️ <b>AGM CLI Error:</b> <code>{}</code>",
            clean_for_telegram_html(&e.to_string(), 300)
        ),
    }
}

/// Query API Proxy health and active bindings
pub async fn execute_api_status_command() -> String {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok();

    let mut proxy_online = false;
    let mut proxy_detail = String::new();
    if let Some(c) = client {
        if let Ok(resp) = c.get("http://127.0.0.1:8045/accounts/current").send().await {
            if resp.status().is_success() {
                proxy_online = true;
                proxy_detail = resp.text().await.unwrap_or_default();
            }
        }
    }

    let active_acc = account::load_account_index()
        .ok()
        .and_then(|idx| {
            let cid = idx.current_account_id?;
            idx.accounts.into_iter().find(|a| a.id == cid)
        })
        .map(|a| format!("{} ({})", a.email, a.id))
        .unwrap_or_else(|| "None".to_string());

    let status_badge = if proxy_online {
        "🟢 ONLINE (http://127.0.0.1:8045)"
    } else {
        "⚪ STANDBY / DIRECT MODE (Port 8045 idle)"
    };

    let mut msg = format!(
        "🔌 <b>API Gateway &amp; Account Bindings</b>\n\n\
        • <b>Gateway State:</b> <code>{}</code>\n\
        • <b>Active Bound Account:</b> <code>{}</code>\n",
        status_badge,
        clean_for_telegram_html(&active_acc, 80)
    );

    if !proxy_detail.trim().is_empty() {
        msg.push_str(&format!(
            "\n<b>Live Loopback Payload:</b>\n<pre>{}</pre>",
            clean_for_telegram_html(proxy_detail.trim(), 1200)
        ));
    }

    msg
}

/// Execute Prompt Backup ("Backpack") or Restore command
pub fn execute_backup_command(args_str: &str) -> String {
    let sub = args_str.trim().to_lowercase();
    if sub == "ls" || sub == "list" || sub == "status" {
        match backup_prompts_db::list_backup_batches(None) {
            Ok(batches) => {
                if batches.is_empty() {
                    return "🎒 <b>Prompt Backup Vault:</b> No backup batches stored.".to_string();
                }
                let mut rows = String::new();
                for (i, b) in batches.iter().take(8).enumerate() {
                    rows.push_str(&format!(
                        "{}. <code>{}</code> — {} prompt(s) [restored: {}]\n",
                        i + 1,
                        clean_for_telegram_html(&b.id, 32),
                        b.prompts_count,
                        b.is_fully_restored
                    ));
                }
                format!(
                    "🎒 <b>Split SQLite Prompt Backups ({} total):</b>\n\n{}",
                    batches.len(),
                    rows
                )
            }
            Err(e) => format!(
                "⚠️ <b>Backup Query Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else if sub == "restore" || sub == "rrp" || sub == "resend" {
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

        let repo_resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            repo_db::dispatch_running_prompts("default"),
            "dispatch_running_prompts",
        );
        match backup_prompts_db::restore_running_prompts(Some("default"), false, None) {
            Ok(records) => format!(
                "♻️ <b>Prompt Restoration Complete:</b>\n\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • Restored and re-queued <b>{}</b> prompt(s) from split SQLite backup.\n\
                • Re-injected <b>{}</b> in-flight prompt(s) directly into workspaces.",
                clean_for_telegram_html(&proj_display, 80),
                records.len(),
                repo_resent.len()
            ),
            Err(e) => format!(
                "⚠️ <b>Prompt Restore Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else {
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

        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            repo_db::backup_running_prompts("default"),
            "backup_running_prompts",
        );
        match backup_prompts_db::backup_active_running_prompts(Some("default"), None) {
            Ok((batch, records)) => format!(
                "🎒 <b>Running Prompts Backed Up Successfully!</b>\n\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • <b>Batch ID:</b> <code>{}</code>\n\
                • <b>Captured Prompts:</b> <b>{}</b>\n\
                • <b>Vault Path:</b> <code>{}</code>\n\n\
                💡 Send <code>/restore</code> anytime to resume backed-up prompts.",
                clean_for_telegram_html(&proj_display, 80),
                clean_for_telegram_html(&batch.id, 40),
                records.len(),
                clean_for_telegram_html(&batch.file_path, 120)
            ),
            Err(e) => format!(
                "⚠️ <b>Backup Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    }
}

/// Execute Email status / ping / help command from Telegram
pub fn execute_email_command(args_str: &str) -> String {
    let sub = args_str.trim().to_lowercase();
    let accounts = email_vault_db::list_email_accounts().unwrap_or_default();
    let recipients = email_vault_db::list_notify_recipients().unwrap_or_default();
    let active_recipients: Vec<String> = recipients
        .iter()
        .filter(|r| r.is_active)
        .map(|r| r.email.clone())
        .collect();

    if sub.is_empty() || sub == "status" || sub == "ls" {
        let default_sender = accounts
            .iter()
            .find(|a| a.is_default && a.is_active)
            .or_else(|| accounts.iter().find(|a| a.is_active))
            .map(|a| a.email.clone())
            .unwrap_or_else(|| "None".to_string());

        return format!(
            "📧 <b>Email Telemetry &amp; Vault Status</b>\n\n\
            • <b>Configured Senders:</b> <code>{}</code> (Default: <code>{}</code>)\n\
            • <b>Active Recipients:</b> <code>{}</code> ({})\n\n\
            💡 Send <code>/email ping</code> to dispatch a test email or <code>/email help</code> to send the full HTML command guide.",
            accounts.len(),
            clean_for_telegram_html(&default_sender, 48),
            active_recipients.len(),
            clean_for_telegram_html(&active_recipients.join(", "), 120)
        );
    }

    let targets = if active_recipients.is_empty() {
        accounts
            .iter()
            .filter(|a| a.is_active)
            .map(|a| a.email.clone())
            .collect::<Vec<_>>()
    } else {
        active_recipients
    };

    if targets.is_empty() {
        return "⚠️ <b>Email Dispatch Skipped:</b> No active sender or recipient accounts configured in Email Vault.".to_string();
    }

    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();

    if sub == "help" {
        let (subj, body) = email_sender::render_help_email(&m_name, &m_ip);
        match email_sender::dispatch_email_with_failover(&subj, &body, &targets) {
            Ok(res) => format!(
                "✅ <b>Help Manual Email Sent!</b>\n\
                • <b>Sender:</b> <code>{}</code>\n\
                • <b>Recipients ({}):</b> <code>{}</code>",
                clean_for_telegram_html(&res.used_account_email, 48),
                targets.len(),
                clean_for_telegram_html(&targets.join(", "), 120)
            ),
            Err(e) => format!(
                "⚠️ <b>Email Dispatch Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else {
        let now = Utc::now().timestamp();
        let (subj, body) =
            email_sender::render_test_ping_email("Telegram-Remote", &m_name, &m_ip, now);
        match email_sender::dispatch_email_with_failover(&subj, &body, &targets) {
            Ok(res) => format!(
                "✅ <b>Test Ping Email Delivered!</b>\n\
                • <b>Sender:</b> <code>{}</code>\n\
                • <b>Recipients ({}):</b> <code>{}</code>",
                clean_for_telegram_html(&res.used_account_email, 48),
                targets.len(),
                clean_for_telegram_html(&targets.join(", "), 120)
            ),
            Err(e) => format!(
                "⚠️ <b>Email Ping Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    }
}
