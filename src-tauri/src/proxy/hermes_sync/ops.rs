use super::*;

pub fn sync_hermes_provider(
    proxy_url: String,
    api_key: String,
    discover_models: bool,
    models: Vec<String>,
    activate: bool,
    default_model: Option<String>,
) -> Result<(), String> {
    let _lock = acquire_hermes_config_lock();
    let normalized_url = normalize_base_url(&proxy_url);
    if normalized_url.trim().is_empty() || normalized_url == "/v1" || api_key.trim().is_empty() {
        return Err("Hermes base URL and API key are required".to_string());
    }
    let models: Vec<String> = models
        .into_iter()
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .collect();
    if !discover_models && models.is_empty() {
        return Err("Select at least one model or enable automatic model discovery".to_string());
    }
    let selected_default = default_model
        .as_deref()
        .filter(|model| !model.trim().is_empty())
        .or_else(|| models.first().map(String::as_str));
    if activate && selected_default.is_none() {
        return Err("Select a default model before activating Antigravity Manager".to_string());
    }
    if activate
        && !discover_models
        && selected_default.is_some_and(|default| !models.iter().any(|model| model == default))
    {
        return Err("The default model must be included in the selected Hermes models".to_string());
    }

    let path = get_config_path().ok_or("Failed to get Hermes config directory")?;
    let source = read_hermes_source(&path)?;
    create_backup(&path)?;
    let backup = get_backup_path()
        .filter(|path| path.exists())
        .map(|path| read_hermes_source(&path))
        .transpose()?;
    let updated = apply_sync_losslessly(
        &source,
        &normalized_url,
        api_key.trim(),
        discover_models,
        &models,
        activate,
        selected_default,
        backup.as_deref(),
    )?;
    atomically_write_source(&path, &updated)
}

pub fn restore_hermes_config() -> Result<(), String> {
    let _lock = acquire_hermes_config_lock();
    let path = get_config_path().ok_or("Failed to get Hermes config directory")?;
    let backup_path = get_backup_path().ok_or("Failed to get Hermes config directory")?;
    if !backup_path.exists() {
        return Err("No backup file found".to_string());
    }
    let restored = apply_restore_losslessly(
        &read_hermes_source(&path)?,
        &read_hermes_source(&backup_path)?,
    )?;
    atomically_write_source(&path, &restored)?;
    fs::remove_file(backup_path).map_err(|error| format!("Failed to remove backup: {error}"))
}

pub fn clear_hermes_config() -> Result<(), String> {
    let _lock = acquire_hermes_config_lock();
    let path = get_config_path().ok_or("Failed to get Hermes config directory")?;
    if !path.exists() {
        return Ok(());
    }
    let source = read_hermes_source(&path)?;
    let backup = get_backup_path()
        .filter(|path| path.exists())
        .map(|path| read_hermes_source(&path))
        .transpose()?;
    let (updated, changed) = apply_clear_losslessly(&source, backup.as_deref())?;
    if !changed {
        return Ok(());
    }
    create_backup(&path)?;
    atomically_write_source(&path, &updated)
}

pub fn read_hermes_config_content() -> Result<String, String> {
    let _lock = acquire_hermes_config_lock();
    let path = get_config_path().ok_or("Failed to get Hermes config directory")?;
    if !path.exists() {
        return Err(format!("Config file does not exist: {path:?}"));
    }
    redact_sensitive_source(&read_hermes_source(&path)?)
}

#[tauri::command]
pub async fn get_hermes_sync_status(proxy_url: Option<String>) -> Result<HermesStatus, String> {
    // CLI startup can be slow or hang; never block configuration operations on it.
    let (installed, version) = check_hermes_installed().await;
    tokio::task::spawn_blocking(move || {
        let _lock = acquire_hermes_config_lock();
        let state = read_config_state(proxy_url);
        Ok(HermesStatus {
            installed,
            version,
            is_synced: state.is_synced,
            has_backup: state.has_backup,
            current_base_url: state.current_base_url,
            files: vec![HERMES_CONFIG_FILE.to_string()],
            discover_models: state.discover_models,
            configured_models: state.configured_models,
            is_active: state.is_active,
            default_model: state.default_model,
        })
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute check".to_string()))
}

#[tauri::command]
pub async fn execute_hermes_sync(
    proxy_url: String,
    api_key: String,
    discover_models: bool,
    models: Vec<String>,
    activate: bool,
    default_model: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        sync_hermes_provider(
            proxy_url,
            api_key,
            discover_models,
            models,
            activate,
            default_model,
        )
    })
    .await
    .unwrap_or_else(|_| Err("Failed to execute sync".to_string()))
}

#[tauri::command]
pub async fn execute_hermes_restore() -> Result<(), String> {
    tokio::task::spawn_blocking(restore_hermes_config)
        .await
        .unwrap_or_else(|_| Err("Failed to execute restore".to_string()))
}

#[tauri::command]
pub async fn execute_hermes_clear() -> Result<(), String> {
    tokio::task::spawn_blocking(clear_hermes_config)
        .await
        .unwrap_or_else(|_| Err("Failed to execute clear".to_string()))
}

#[tauri::command]
pub async fn get_hermes_config_content() -> Result<String, String> {
    tokio::task::spawn_blocking(read_hermes_config_content)
        .await
        .unwrap_or_else(|_| Err("Failed to read config".to_string()))
}
