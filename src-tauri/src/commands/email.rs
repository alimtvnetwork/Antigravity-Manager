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

#[tauri::command]
pub async fn trigger_manual_email_check() -> AppResult<String> {
    let m_name = email_watcher::detect_machine_name();
    let m_ip = email_watcher::detect_local_ip();
    let (subj, body) = email_sender::render_help_email(&m_name, &m_ip);

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

    email_sender::dispatch_email_with_failover(&subj, &body, &active_emails)
        .map(|res| format!("Dispatched alert via account '{}'", res.used_account_email))
        .map_err(AppError::Email)
}
