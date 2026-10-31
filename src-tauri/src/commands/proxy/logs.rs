use super::*;

/// Get anti-generation service status
#[tauri::command]
pub async fn get_proxy_status(state: State<'_, ProxyServiceState>) -> Result<ProxyStatus, String> {
    // Prioritize checking the startup flag to avoid being blocked by write locks
    if state.starting.load(Ordering::SeqCst) {
        return Ok(ProxyStatus {
            running: false, // Logically it has not been run yet
            port: 0,
            base_url: "starting".to_string(), // Identify the front-end
            active_accounts: 0,
        });
    }

    // Use try_read to avoid queuing delays in this command
    let lock_res = state.instance.try_read();

    match lock_res {
        Ok(instance_lock) => match instance_lock.as_ref() {
            Some(instance) => Ok(ProxyStatus {
                running: true,
                port: instance.config.port,
                base_url: format!("http://127.0.0.1:{}", instance.config.port),
                active_accounts: instance.token_manager.len(),
            }),
            None => Ok(ProxyStatus {
                running: false,
                port: 0,
                base_url: String::new(),
                active_accounts: 0,
            }),
        },
        Err(_) => {
            // If the lock cannot be obtained, it means that the write operation is in progress (it may be starting or stopping)
            Ok(ProxyStatus {
                running: false,
                port: 0,
                base_url: "busy".to_string(),
                active_accounts: 0,
            })
        }
    }
}

/// Get anti-generation service statistics
#[tauri::command]
pub async fn get_proxy_stats(state: State<'_, ProxyServiceState>) -> Result<ProxyStats, String> {
    let monitor_lock = state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        Ok(monitor.get_stats().await)
    } else {
        Ok(ProxyStats::default())
    }
}

/// Get reverse generation request logs
#[tauri::command]
pub async fn get_proxy_logs(
    state: State<'_, ProxyServiceState>,
    limit: Option<usize>,
) -> Result<Vec<ProxyRequestLog>, String> {
    let monitor_lock = state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        Ok(monitor.get_logs(limit.unwrap_or(100)).await)
    } else {
        Ok(Vec::new())
    }
}

/// Set monitoring on state
#[tauri::command]
pub async fn set_proxy_monitor_enabled(
    state: State<'_, ProxyServiceState>,
    enabled: bool,
) -> Result<(), String> {
    let monitor_lock = state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        monitor.set_enabled(enabled);
    }
    Ok(())
}

/// Set capture health check log status
#[tauri::command]
pub async fn set_proxy_capture_health_logs(
    state: State<'_, ProxyServiceState>,
    enabled: bool,
) -> Result<(), String> {
    let monitor_lock = state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        monitor.set_capture_health_logs(enabled);
    }
    Ok(())
}

/// Clear reverse generation request log
#[tauri::command]
pub async fn clear_proxy_logs(state: State<'_, ProxyServiceState>) -> Result<(), String> {
    let monitor_lock = state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        monitor.clear().await;
    }
    Ok(())
}

/// Clear all think block cache and persistent data (including RAM memory sliding window and SQLite database, but do not delete any request logs)
#[tauri::command]
pub async fn clear_thinking_store() -> Result<usize, String> {
    // 1. Clear the ThinkingStore instance and SignatureCache in memory
    crate::proxy::thinking_store::ThinkingStore::global().clear();
    crate::proxy::SignatureCache::global().clear();

    // 2. Clear all thinking_records and thinking_sessions in the SQLite database
    tokio::task::spawn_blocking(crate::modules::proxy_db::clear_all_thinking_data)
        .await
        .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

/// Get the total number of records stored in the current thinking block
#[tauri::command]
pub async fn get_thinking_store_count() -> Result<usize, String> {
    tokio::task::spawn_blocking(crate::modules::proxy_db::get_thinking_records_count)
        .await
        .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

/// Get the number of bytes occupied by the proxy database disk
#[tauri::command]
pub async fn get_proxy_db_disk_size() -> Result<u64, String> {
    tokio::task::spawn_blocking(crate::modules::proxy_db::get_proxy_db_disk_bytes)
        .await
        .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

/// Get reverse generation request log (paginated)
#[tauri::command]
pub async fn get_proxy_logs_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ProxyRequestLog>, String> {
    crate::modules::proxy_db::get_logs_summary(limit.unwrap_or(20), offset.unwrap_or(0))
}

/// Get complete details of a single log
#[tauri::command]
pub async fn get_proxy_log_detail(
    log_id: Option<String>,
    #[allow(non_snake_case)] logId: Option<String>,
) -> Result<ProxyRequestLog, String> {
    let id = log_id
        .or(logId)
        .ok_or_else(|| "Missing log_id parameter".to_string())?;
    crate::modules::proxy_db::get_log_detail(&id)
}

/// Get the total number of logs
#[tauri::command]
pub async fn get_proxy_logs_count() -> Result<u64, String> {
    crate::modules::proxy_db::get_logs_count()
}

/// Export all logs to the specified file
#[tauri::command]
pub async fn export_proxy_logs(file_path: String) -> Result<usize, String> {
    let logs = crate::modules::proxy_db::get_all_logs_for_export()?;
    let count = logs.len();

    let json = serde_json::to_string_pretty(&logs)
        .map_err(|e| format!("Failed to serialize logs: {}", e))?;

    std::fs::write(&file_path, json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(count)
}

/// Export the specified log JSON to a file
#[tauri::command]
pub async fn export_proxy_logs_json(file_path: String, json_data: String) -> Result<usize, String> {
    // Parse to count items
    let logs: Vec<serde_json::Value> =
        serde_json::from_str(&json_data).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    let count = logs.len();

    // Pretty print
    let pretty_json =
        serde_json::to_string_pretty(&logs).map_err(|e| format!("Failed to serialize: {}", e))?;

    std::fs::write(&file_path, pretty_json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(count)
}

/// Get the number of logs with search conditions
#[tauri::command]
pub async fn get_proxy_logs_count_filtered(
    filter: String,
    errors_only: bool,
) -> Result<u64, String> {
    crate::modules::proxy_db::get_logs_count_filtered(&filter, errors_only)
}

/// Get paginated logs with search conditions
#[tauri::command]
pub async fn get_proxy_logs_filtered(
    filter: String,
    errors_only: bool,
    limit: usize,
    offset: usize,
) -> Result<Vec<crate::proxy::monitor::ProxyRequestLog>, String> {
    crate::modules::proxy_db::get_logs_filtered(&filter, errors_only, limit, offset)
}
