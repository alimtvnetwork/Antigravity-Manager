use super::*;

/// 获取反代服务状态
#[tauri::command]
pub async fn get_proxy_status(state: State<'_, ProxyServiceState>) -> Result<ProxyStatus, String> {
    // 优先检查启动标志，避免被写锁阻塞
    if state.starting.load(Ordering::SeqCst) {
        return Ok(ProxyStatus {
            running: false, // 逻辑上还没运行
            port: 0,
            base_url: "starting".to_string(), // 给前端标识
            active_accounts: 0,
        });
    }

    // 使用 try_read 避免在该命令中产生产生排队延迟
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
            // 如果拿不到锁，说明正在进行写操作（可能是正在启动或停止中）
            Ok(ProxyStatus {
                running: false,
                port: 0,
                base_url: "busy".to_string(),
                active_accounts: 0,
            })
        }
    }
}

/// 获取反代服务统计
#[tauri::command]
pub async fn get_proxy_stats(state: State<'_, ProxyServiceState>) -> Result<ProxyStats, String> {
    let monitor_lock = state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        Ok(monitor.get_stats().await)
    } else {
        Ok(ProxyStats::default())
    }
}

/// 获取反代请求日志
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

/// 设置监控开启状态
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

/// 设置捕获健康检查日志状态
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

/// 清除反代请求日志
#[tauri::command]
pub async fn clear_proxy_logs(state: State<'_, ProxyServiceState>) -> Result<(), String> {
    let monitor_lock = state.monitor.read().await;
    if let Some(monitor) = monitor_lock.as_ref() {
        monitor.clear().await;
    }
    Ok(())
}

/// 清空所有思考块缓存与持久化数据 (包含 RAM 内存滑动窗口与 SQLite 数据库，但不删除任何请求日志)
#[tauri::command]
pub async fn clear_thinking_store() -> Result<usize, String> {
    // 1. 清空内存中 ThinkingStore 实例与 SignatureCache
    crate::proxy::thinking_store::ThinkingStore::global().clear();
    crate::proxy::SignatureCache::global().clear();

    // 2. 清空 SQLite 数据库中所有的 thinking_records 与 thinking_sessions
    tokio::task::spawn_blocking(crate::modules::proxy_db::clear_all_thinking_data)
        .await
        .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

/// 获取当前思考块存储的记录总数
#[tauri::command]
pub async fn get_thinking_store_count() -> Result<usize, String> {
    tokio::task::spawn_blocking(crate::modules::proxy_db::get_thinking_records_count)
        .await
        .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

/// 获取代理数据库磁盘占用字节数
#[tauri::command]
pub async fn get_proxy_db_disk_size() -> Result<u64, String> {
    tokio::task::spawn_blocking(crate::modules::proxy_db::get_proxy_db_disk_bytes)
        .await
        .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

/// 获取反代请求日志 (分页)
#[tauri::command]
pub async fn get_proxy_logs_paginated(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ProxyRequestLog>, String> {
    crate::modules::proxy_db::get_logs_summary(limit.unwrap_or(20), offset.unwrap_or(0))
}

/// 获取单条日志的完整详情
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

/// 获取日志总数
#[tauri::command]
pub async fn get_proxy_logs_count() -> Result<u64, String> {
    crate::modules::proxy_db::get_logs_count()
}

/// 导出所有日志到指定文件
#[tauri::command]
pub async fn export_proxy_logs(file_path: String) -> Result<usize, String> {
    let logs = crate::modules::proxy_db::get_all_logs_for_export()?;
    let count = logs.len();

    let json = serde_json::to_string_pretty(&logs)
        .map_err(|e| format!("Failed to serialize logs: {}", e))?;

    std::fs::write(&file_path, json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(count)
}

/// 导出指定的日志JSON到文件
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

/// 获取带搜索条件的日志数量
#[tauri::command]
pub async fn get_proxy_logs_count_filtered(
    filter: String,
    errors_only: bool,
) -> Result<u64, String> {
    crate::modules::proxy_db::get_logs_count_filtered(&filter, errors_only)
}

/// 获取带搜索条件的分页日志
#[tauri::command]
pub async fn get_proxy_logs_filtered(
    filter: String,
    errors_only: bool,
    limit: usize,
    offset: usize,
) -> Result<Vec<crate::proxy::monitor::ProxyRequestLog>, String> {
    crate::modules::proxy_db::get_logs_filtered(&filter, errors_only, limit, offset)
}
