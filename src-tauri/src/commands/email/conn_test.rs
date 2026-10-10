use super::*;

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
