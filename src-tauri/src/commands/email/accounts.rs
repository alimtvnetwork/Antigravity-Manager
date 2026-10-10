use super::*;

#[tauri::command]
pub async fn list_email_accounts() -> AppResult<Vec<EmailAccount>> {
    email_vault_db::list_email_accounts().map_err(AppError::Email)
}

#[tauri::command]
pub async fn add_email_account(account: EmailAccountInput) -> AppResult<EmailAccount> {
    let result = email_vault_db::upsert_email_account(account).map_err(AppError::Email)?;
    let val = serde_json::to_value(&result).unwrap_or_default();
    crate::modules::notification_hub::notify_email_config_added(
        &format!("Mailbox Account Added ({})", result.email),
        val,
    );
    Ok(result)
}

#[tauri::command]
pub async fn update_email_account(account: EmailAccountInput) -> AppResult<EmailAccount> {
    let result = email_vault_db::upsert_email_account(account).map_err(AppError::Email)?;
    let val = serde_json::to_value(&result).unwrap_or_default();
    crate::modules::notification_hub::notify_email_config_added(
        &format!("Mailbox Account Updated ({})", result.email),
        val,
    );
    Ok(result)
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
    let result = email_vault_db::add_notify_recipient(recipient).map_err(AppError::Email)?;
    let val = serde_json::to_value(&result).unwrap_or_default();
    crate::modules::notification_hub::notify_email_config_added(
        &format!("Notifier Recipient Added ({})", result.email),
        val,
    );
    Ok(result)
}

#[tauri::command]
pub async fn delete_notify_recipient(id: String) -> AppResult<()> {
    email_vault_db::delete_notify_recipient(&id).map_err(AppError::Email)
}
