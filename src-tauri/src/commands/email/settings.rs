use super::*;

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
