use super::*;

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
