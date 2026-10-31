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

    let mut cmd = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output_res = cmd.args(&parts).output();
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
    let mut cmd = Command::new(&bin);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    match cmd.args(&parts).output() {
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

fn check_instance_active_status(instance_id: &str) -> &'static str {
    if let Ok(reg) = crate::modules::instance::load_registry() {
        if let Some(inst) = reg
            .instances
            .iter()
            .find(|i| i.id.eq_ignore_ascii_case(instance_id))
        {
            let pids =
                crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
            let is_run = !pids.is_empty()
                || crate::modules::instance::is_instance_running(
                    &inst.id,
                    &inst.data_dir,
                    inst.pid,
                );
            if is_run {
                return "🟢 Active & Running";
            }
        }
    }
    "⚪ Standby / Ready"
}

/// Execute Prompt Backup ("Backpack") or Restore command
pub fn execute_backup_command(args_str: &str) -> String {
    let trimmed = args_str.trim();
    let normalized = if let Some(stripped) = trimmed.strip_prefix("backup prompt ") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_prefix("backup prompts ") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_prefix("backup running ") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_prefix("brp ") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_prefix("backup ") {
        stripped.trim()
    } else if trimmed.eq_ignore_ascii_case("backup prompt")
        || trimmed.eq_ignore_ascii_case("backup prompts")
        || trimmed.eq_ignore_ascii_case("backup running")
        || trimmed.eq_ignore_ascii_case("backup")
        || trimmed.eq_ignore_ascii_case("brp")
    {
        ""
    } else {
        trimmed
    };

    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let first = tokens.first().copied().unwrap_or("").to_lowercase();

    if first == "ls" || first == "list" || first == "status" {
        let limit = tokens
            .get(1)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(8);
        match backup_prompts_db::list_backup_batches(None) {
            Ok(batches) => {
                if batches.is_empty() {
                    return "📦 <b>No prompt backups stored.</b>".to_string();
                }
                let mut rows = String::new();
                for (i, b) in batches.iter().take(limit).enumerate() {
                    let id_short = b.id.get(..8).unwrap_or(&b.id);
                    let restored_badge = if b.is_fully_restored {
                        "✅ restored"
                    } else {
                        "⏳ pending"
                    };
                    rows.push_str(&format!(
                        "{}. <code>{}</code> — <b>{}</b> prompt(s) [{}]\n",
                        i + 1,
                        clean_for_telegram_html(id_short, 16),
                        b.prompts_count,
                        restored_badge
                    ));
                }
                format!(
                    "📦 <b>Prompt Backups ({}):</b>\n\n{}\n💡 <i>Send <code>/restore [instance]</code> or <code>/rrp</code> anytime to resume prompts.</i>",
                    batches.len(),
                    rows
                )
            }
            Err(e) => format!(
                "❌ <b>Couldn't read prompt backups:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else if first == "restore" || first == "rrp" || first == "resend" {
        let raw_target = tokens
            .get(1)
            .map(|s| s.trim_matches(|c: char| c == '\'' || c == '"' || c == '/' || c == '\\'))
            .filter(|s| !s.is_empty());
        let is_all = raw_target
            .map(|s| s.eq_ignore_ascii_case("all"))
            .unwrap_or(false);
        let target_inst = if is_all {
            "all"
        } else {
            raw_target.unwrap_or("default")
        };
        let scope_filter = if is_all { None } else { Some(target_inst) };

        let repo_resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
        if !is_all {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                repo_db::dispatch_running_prompts(target_inst),
                "dispatch_running_prompts",
            );
        } else {
            crate::error::record_ignored(
                repo_db::dispatch_running_prompts("default"),
                "dispatch_running_prompts_default",
            );
        }
        match backup_prompts_db::restore_running_prompts(scope_filter, false, None) {
            Ok(records) => {
                let mut restored_image_count = 0;
                for p in &records {
                    if let Some(payload) = &p.images_payload {
                        let uris = crate::modules::repo_db::detection::parse_image_uris(payload);
                        if !uris.is_empty() {
                            restored_image_count += uris.len();
                        } else if p.has_images {
                            restored_image_count += 1;
                        }
                    } else if p.has_images {
                        let (uris, paths) =
                            crate::modules::repo_db::detection::extract_image_uris_or_paths(
                                &p.prompt_text,
                            );
                        let c = uris.len() + paths.len();
                        restored_image_count += if c > 0 { c } else { 1 };
                    }
                }
                let status_badge = if is_all {
                    check_instance_active_status("default")
                } else {
                    check_instance_active_status(target_inst)
                };

                format!(
                    "♻️ <b>Prompt Restore Completed</b>\n\n\
                    • <b>Target Instance:</b> <code>{}</code>\n\
                    • <b>Instance Status:</b> {}\n\
                    • <b>Restored Prompts:</b> <b>{}</b>\n\
                    • <b>Attached Media:</b> <b>{}</b> image(s)\n\
                    • <b>Re-sent Commands:</b> <b>{}</b> command(s) queued\n\
                    • <b>Timestamp:</b> <code>{}</code>\n\n\
                    💡 <i>Execution resumed in target workspace.</i>",
                    clean_for_telegram_html(target_inst, 64),
                    status_badge,
                    records.len(),
                    restored_image_count,
                    repo_resent.len(),
                    Utc::now().to_rfc3339()
                )
            }
            Err(e) => format!(
                "❌ <b>Restore Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else {
        let raw_target = tokens.first().copied().unwrap_or("default");
        let target_inst =
            raw_target.trim_matches(|c: char| c == '\'' || c == '"' || c == '/' || c == '\\');
        let target_inst = if target_inst.is_empty() {
            "default"
        } else {
            target_inst
        };

        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            repo_db::backup_running_prompts(target_inst),
            "backup_running_prompts",
        );
        match backup_prompts_db::backup_active_running_prompts(Some(target_inst), None) {
            Ok((batch, records)) => {
                let count = records.len();
                let mut image_count = 0;
                for p in &records {
                    if let Some(payload) = &p.images_payload {
                        let uris = crate::modules::repo_db::detection::parse_image_uris(payload);
                        if !uris.is_empty() {
                            image_count += uris.len();
                        } else if p.has_images {
                            image_count += 1;
                        }
                    } else if p.has_images {
                        let (uris, paths) =
                            crate::modules::repo_db::detection::extract_image_uris_or_paths(
                                &p.prompt_text,
                            );
                        let c = uris.len() + paths.len();
                        image_count += if c > 0 { c } else { 1 };
                    }
                }
                let batch_id_short = batch.id.get(..8).unwrap_or(&batch.id);
                let iso_time = Utc::now().to_rfc3339();
                let status_badge = check_instance_active_status(target_inst);

                format!(
                    "📦 <b>Prompt Backup Successful</b>\n\n\
                    • <b>Target Instance:</b> <code>{}</code>\n\
                    • <b>Instance Status:</b> {}\n\
                    • <b>Backup Batch ID:</b> <code>{}</code>\n\
                    • <b>Backed Up Prompts:</b> <b>{}</b>\n\
                    • <b>Multimodal Images:</b> <b>{}</b> image(s) attached\n\
                    • <b>Timestamp:</b> <code>{}</code>\n\n\
                    💡 <i>Prompts are safely preserved in offline SQLite storage. Send <code>/restore {}</code> or <code>/rrp</code> anytime to resume execution.</i>",
                    clean_for_telegram_html(target_inst, 64),
                    status_badge,
                    clean_for_telegram_html(batch_id_short, 32),
                    count,
                    image_count,
                    iso_time,
                    clean_for_telegram_html(target_inst, 64)
                )
            }
            Err(e) => format!(
                "❌ <b>Backup Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    }
}

/// Handle `/open [machine]` — open the AGM UI on the target machine.
///
/// No argument targets this machine; a fleet node alias (see `/nodes`)
/// dispatches via GitMap Cluster SSH with a Supabase queue fallback.
pub async fn execute_open_command(args_str: &str) -> String {
    super::open_shot::execute_open_command(args_str).await
}

/// Handle `/screenshot [machine]` — capture the target machine's screen and
/// send the image(s) back to the Telegram chat.
///
/// No argument captures every known machine; AGM is opened first when needed.
/// Returns the confirmation text; photos are sent to the chat directly.
pub async fn execute_screenshot_command(args_str: &str) -> String {
    super::open_shot::execute_screenshot_command(args_str).await
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
