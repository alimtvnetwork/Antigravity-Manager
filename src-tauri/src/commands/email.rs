//! Tauri IPC Commands for Email Dispatch and Mailbox Management
//! Exposes email accounts, credentials, import/export, and watcher commands with AppError.

#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use crate::modules::email_inbound;
use crate::modules::email_io::{self, ImportSummary};
use crate::modules::email_sender;
use crate::modules::email_vault_db::{
    self, EmailAccount, EmailAccountInput, EmailNotificationSettings, NotifyRecipient,
    NotifyRecipientInput,
};
use crate::modules::email_watcher::{self, WatcherStatus};
use crate::utils::command::CommandExtWrapper;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn get_email_settings() -> AppResult<EmailNotificationSettings> {
    email_vault_db::get_notification_settings().map_err(AppError::Email)
}

#[tauri::command]
pub async fn save_email_settings(settings: EmailNotificationSettings) -> AppResult<()> {
    let is_enabled = settings.is_enabled;
    email_vault_db::save_notification_settings(settings).map_err(AppError::Email)?;

    if is_enabled {
        email_watcher::start_email_watcher();
    } else {
        email_watcher::stop_email_watcher();
    }
    Ok(())
}

#[tauri::command]
pub async fn list_email_accounts() -> AppResult<Vec<EmailAccount>> {
    email_vault_db::list_email_accounts().map_err(AppError::Email)
}

#[tauri::command]
pub async fn add_email_account(account: EmailAccountInput) -> AppResult<EmailAccount> {
    email_vault_db::upsert_email_account(account).map_err(AppError::Email)
}

#[tauri::command]
pub async fn update_email_account(account: EmailAccountInput) -> AppResult<EmailAccount> {
    email_vault_db::upsert_email_account(account).map_err(AppError::Email)
}

#[tauri::command]
pub async fn delete_email_account(id: String) -> AppResult<()> {
    email_vault_db::delete_email_account(&id).map_err(AppError::Email)
}

#[tauri::command]
pub async fn set_default_email_account(id: String) -> AppResult<()> {
    email_vault_db::set_default_email_account(&id).map_err(AppError::Email)
}

#[tauri::command]
pub async fn list_notify_recipients() -> AppResult<Vec<NotifyRecipient>> {
    email_vault_db::list_notify_recipients().map_err(AppError::Email)
}

#[tauri::command]
pub async fn add_notify_recipient(recipient: NotifyRecipientInput) -> AppResult<NotifyRecipient> {
    email_vault_db::add_notify_recipient(recipient).map_err(AppError::Email)
}

#[tauri::command]
pub async fn delete_notify_recipient(id: String) -> AppResult<()> {
    email_vault_db::delete_notify_recipient(&id).map_err(AppError::Email)
}

#[tauri::command]
pub async fn test_smtp_connection(account_id: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let accounts = email_vault_db::list_email_accounts().map_err(AppError::Email)?;
        let account = accounts
            .into_iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| AppError::Email("Account not found".to_string()))?;

        let (subj, body) = email_sender::render_help_email(
            &email_watcher::detect_machine_name(),
            &email_watcher::detect_local_ip(),
        );

        email_sender::send_via_account(&account, &subj, &body, &[account.email.clone()])
            .map_err(AppError::Email)?;

        Ok("SMTP connection successful! Test message delivered.".to_string())
    })
    .await
    .unwrap_or_else(|_| Err(AppError::Email("Test task panicked".to_string())))
}

#[tauri::command]
pub async fn test_imap_connection(account_id: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let accounts = email_vault_db::list_email_accounts().map_err(AppError::Email)?;
        let account = accounts
            .into_iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| AppError::Email("Account not found".to_string()))?;

        let messages = email_inbound::poll_unread_messages(&account, 1).map_err(AppError::Email)?;
        Ok(format!(
            "IMAP connection successful! Found {} unread messages.",
            messages.len()
        ))
    })
    .await
    .unwrap_or_else(|_| Err(AppError::Email("Test task panicked".to_string())))
}

#[tauri::command]
pub async fn test_direct_email_connection(account: EmailAccountInput) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let password = if let Some(ref p) = account.password {
            if !p.trim().is_empty() {
                p.clone()
            } else if let Some(ref id) = account.id {
                email_vault_db::get_account_secret(id).unwrap_or_default()
            } else {
                String::new()
            }
        } else if let Some(ref id) = account.id {
            email_vault_db::get_account_secret(id).unwrap_or_default()
        } else {
            String::new()
        };

        let temp_account = EmailAccount {
            id: account.id.clone().unwrap_or_default(),
            alias: if account.alias.trim().is_empty() {
                "Test Mailer".to_string()
            } else {
                account.alias.clone()
            },
            email: account.email.clone(),
            smtp_host: account.smtp_host.clone(),
            smtp_port: account.smtp_port,
            imap_host: account.imap_host.clone(),
            imap_port: account.imap_port,
            encryption_type: account.encryption_type.clone(),
            is_default: account.is_default,
            is_active: account.is_active,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        let (subj, body) = email_sender::render_self_test_email(
            &temp_account.email,
            &email_watcher::detect_machine_name(),
            &email_watcher::detect_local_ip(),
        );

        email_sender::send_via_account_credentials(
            &temp_account,
            &password,
            &subj,
            &body,
            &[temp_account.email.clone()],
        )
        .map_err(AppError::Email)?;

        Ok(format!(
            "SMTP connection successful! Self-test email delivered to {}.",
            temp_account.email
        ))
    })
    .await
    .unwrap_or_else(|_| Err(AppError::Email("Test task panicked".to_string())))
}

#[tauri::command]
pub async fn export_email_data(format: String) -> AppResult<String> {
    match format.to_lowercase().as_str() {
        "json" => email_io::export_to_json().map_err(AppError::Email),
        "csv" => email_io::export_to_csv().map_err(AppError::Email),
        "xlsx" | "excel" => email_io::export_to_excel().map_err(AppError::Email),
        _ => Err(AppError::Email(
            "Unsupported format. Use json, csv, or xlsx.".to_string(),
        )),
    }
}

#[tauri::command]
pub async fn import_email_data(format: String, payload: String) -> AppResult<ImportSummary> {
    match format.to_lowercase().as_str() {
        "json" => email_io::import_from_json(&payload).map_err(AppError::Email),
        "csv" => email_io::import_from_csv(&payload).map_err(AppError::Email),
        "xlsx" | "excel" | "xml" => email_io::import_from_excel(&payload).map_err(AppError::Email),
        _ => Err(AppError::Email(
            "Unsupported format. Use json, csv, or excel/xml.".to_string(),
        )),
    }
}

#[tauri::command]
pub async fn backup_email_db(target_path: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let path = std::path::Path::new(&target_path);
        email_io::backup_vault_db(path)
            .map(|_| format!("Backup successfully written to {}", target_path))
            .map_err(AppError::Email)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::Email("Backup task panicked".to_string())))
}

#[tauri::command]
pub async fn restore_email_db(source_path: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        let path = std::path::Path::new(&source_path);
        email_io::restore_vault_db(path)
            .map(|_| format!("Vault successfully restored from {}", source_path))
            .map_err(AppError::Email)
    })
    .await
    .unwrap_or_else(|_| Err(AppError::Email("Restore task panicked".to_string())))
}

#[tauri::command]
pub async fn get_email_watcher_status() -> AppResult<WatcherStatus> {
    Ok(email_watcher::get_watcher_status().await)
}

fn get_active_recipient_emails() -> Result<Vec<String>, AppError> {
    let recipients = email_vault_db::list_notify_recipients().map_err(AppError::Email)?;
    let active_emails: Vec<String> = recipients
        .into_iter()
        .filter(|r| r.is_active)
        .map(|r| r.email)
        .collect();
    if active_emails.is_empty() {
        return Err(AppError::Email(
            "No active notification recipients configured".to_string(),
        ));
    }
    Ok(active_emails)
}

#[tauri::command]
pub async fn trigger_manual_email_check() -> AppResult<String> {
    let active_emails = get_active_recipient_emails()?;
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let (subj, body) = email_sender::render_help_email(&m_name, &m_ip);

    email_sender::dispatch_email_with_failover(&subj, &body, &active_emails)
        .map(|res| format!("Dispatched alert via account '{}'", res.used_account_email))
        .map_err(AppError::Email)
}

#[tauri::command]
pub async fn dispatch_email_test_ping(project_name: Option<String>) -> AppResult<String> {
    let active_emails = get_active_recipient_emails()?;
    let proj = project_name.unwrap_or_else(|| "Antigravity-Workspace".to_string());
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let now = chrono::Utc::now().timestamp();
    let (subj, body) = email_sender::render_test_ping_email(&proj, &m_name, &m_ip, now);

    let res = email_sender::dispatch_email_with_failover(&subj, &body, &active_emails)
        .map_err(AppError::Email)?;
    email_watcher::activate_awaiting_reply(300);

    Ok(format!(
        "Dispatched test ping for '{}' via '{}'. Fast polling active (5m).",
        proj, res.used_account_email
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub machine_name: String,
    pub machine_ip: String,
}

#[tauri::command]
pub async fn test_execute_cli_command(command: String) -> AppResult<CliExecResult> {
    let cmd_str = command.trim();
    if cmd_str.is_empty() {
        return Err(AppError::Config("Command cannot be empty".to_string()));
    }

    #[cfg(target_os = "windows")]
    let output = {
        let mut cmd = std::process::Command::new("powershell.exe");
        cmd.creation_flags_windows().args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            cmd_str,
        ]);
        cmd.output().map_err(|e| {
            AppError::Process(format!("Failed to execute PowerShell on Windows: {}", e))
        })?
    };

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("sh")
        .args(["-c", cmd_str])
        .output()
        .map_err(|e| AppError::Process(format!("Failed to execute command on Unix: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();
    let machine_name = email_watcher::detect_machine_name();
    let machine_ip = email_watcher::detect_local_ip();

    Ok(CliExecResult {
        exit_code,
        stdout,
        stderr,
        success,
        machine_name,
        machine_ip,
    })
}
