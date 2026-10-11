use super::*;
use super::cmd_quota::internal_refresh_account_quota;

#[tauri::command]
pub async fn list_accounts(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Vec<Account>, String> {
    let mut accounts = tokio::task::spawn_blocking(move || modules::list_accounts())
        .await
        .unwrap_or_else(|_| Err("Task panicked".to_string()))?;

    // [FIX] Blend in-memory TokenManager rate limit status into the UI quota display
    let instance_lock = proxy_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        for account in &mut accounts {
            if let Some(reset_secs) = instance
                .token_manager
                .get_rate_limit_reset_seconds(&account.id)
            {
                if reset_secs > 0 {
                    let reset_iso = chrono::DateTime::<chrono::Utc>::from_timestamp(
                        chrono::Utc::now().timestamp() + reset_secs as i64,
                        0,
                    )
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default();

                    if let Some(ref mut quota_data) = account.quota {
                        for model in &mut quota_data.models {
                            model.percentage = 0;
                            model.reset_time = reset_iso.clone();
                        }
                    }
                }
            }
        }
    }

    Ok(accounts)
}

#[tauri::command]
pub async fn add_account(
    app: tauri::AppHandle,
    _email: String,
    refresh_token: String,
) -> Result<Account, String> {
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app.clone()),
    );

    let mut account = service.add_account(&refresh_token).await?;

    // 自动刷新配额
    // Justification: quota auto-refresh after account add is opportunistic; the account was already added
    crate::error::record_ignored(
        internal_refresh_account_quota(&app, &mut account).await,
        "auto-refresh quota after account add",
    );

    // 重载账号池
    // Justification: proxy pool reload after account add is best-effort sync; the account was already added
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(
            app.state::<crate::commands::proxy::ProxyServiceState>(),
        )
        .await,
        "reload proxy accounts after account add",
    );

    Ok(account)
}

/// 删除账号
/// 删除账号
#[tauri::command]
pub async fn delete_account(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
) -> Result<(), String> {
    let service = modules::account_service::AccountService::new(
        crate::modules::integration::SystemManager::Desktop(app.clone()),
    );
    service.delete_account(&account_id)?;

    // Reload token pool
    // Justification: proxy pool reload after account delete is best-effort sync; the delete already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after account delete",
    );

    Ok(())
}

/// 批量删除账号
#[tauri::command]
pub async fn delete_accounts(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_ids: Vec<String>,
) -> Result<(), String> {
    modules::logger::log_info(&format!(
        "收到批量删除请求，共 {} 个账号",
        account_ids.len()
    ));
    modules::account::delete_accounts(&account_ids).map_err(|e| {
        modules::logger::log_error(&format!("批量删除失败: {}", e));
        e
    })?;

    // 强制同步托盘
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    // Justification: proxy pool reload after batch delete is best-effort sync; the delete already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after batch delete",
    );

    Ok(())
}

/// 重新排序账号列表
/// 根据传入的账号ID数组顺序更新账号排列
#[tauri::command]
pub async fn reorder_accounts(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_ids: Vec<String>,
) -> Result<(), String> {
    modules::logger::log_info(&format!(
        "收到账号重排序请求，共 {} 个账号",
        account_ids.len()
    ));
    modules::account::reorder_accounts(&account_ids).map_err(|e| {
        modules::logger::log_error(&format!("账号重排序失败: {}", e));
        e
    })?;

    // Reload pool to reflect new order if running
    // Justification: proxy pool reload after reorder is best-effort sync; the reorder already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after reorder",
    );
    Ok(())
}

/// 切换账号
#[tauri::command]
pub async fn switch_account(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
    target_ide: Option<String>,
) -> Result<(), String> {
    let active_instance =
        modules::instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let instance_target = if let Some(ref target) = target_ide {
        if target.starts_with("instance:") {
            Some(target.trim_start_matches("instance:").to_string())
        } else {
            None
        }
    } else if active_instance != "default" {
        Some(active_instance)
    } else {
        None
    };

    if let Some(inst_id) = instance_target {
        modules::instance::switch_account_to_instance(&account_id, Some(&inst_id)).await?;
    } else {
        if let Ok(Some(prev_acc)) = modules::account::get_current_account() {
            if !prev_acc.email.is_empty() {
                modules::notification_hub::record_previous_email(&prev_acc.email);
            }
        }

        let service = modules::account_service::AccountService::new(
            crate::modules::integration::SystemManager::Desktop(app.clone()),
        );

        service
            .switch_account(&account_id, target_ide.as_deref())
            .await?;

        if let Ok(acc) = modules::account::load_account(&account_id) {
            // Justification: instance binding is bookkeeping; the account switch already succeeded
            crate::error::record_ignored(
                modules::instance::bind_account_to_instance("default", &acc.id, &acc.email),
                "bind account to default instance after switch",
            );
            if !acc.email.is_empty() {
                modules::notification_hub::notify_account_switched(
                    &acc.email,
                    "default",
                    "Smart Fast-Forward / Account Switch",
                    false,
                );
            }
        }
    }

    // 同步托盘
    crate::modules::tray::update_tray_menus(&app);

    // [FIX #820] Notify proxy to clear stale session bindings and reload accounts
    // Justification: proxy pool reload after account switch is best-effort sync; the switch already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after account switch",
    );

    Ok(())
}

/// 获取当前账号
#[tauri::command]
pub async fn get_current_account() -> Result<Option<Account>, String> {
    // println!("🚀 Backend Command: get_current_account called"); // Commented out to reduce noise for frequent calls, relies on frontend log for frequency
    // Actually user WANTS to see it.
    modules::logger::log_info("Backend Command: get_current_account called");

    let account_id = modules::get_current_account_id()?;

    if let Some(id) = account_id {
        if let Ok(account) = modules::load_account(&id) {
            return Ok(Some(account));
        }
    }

    // Fallback: Check if editor DB has an active refresh token matching one of our saved accounts
    if let Ok(index) = modules::account::load_account_index() {
        let current_target = index.current_target_ide.as_deref();
        if let Ok(db_token) = modules::migration::get_refresh_token_from_db(current_target) {
            let token_trimmed = db_token.trim();
            let has_token = token_trimmed.len() > 0;
            if has_token {
                if let Ok(accounts) = modules::list_accounts() {
                    if let Some(matching) = accounts
                        .into_iter()
                        .find(|a| a.token.refresh_token == token_trimmed)
                    {
                        modules::logger::log_info(&format!(
                            "   Auto-bound current account from editor DB: {}",
                            matching.email
                        ));
                        // Justification: current-account persistence is best-effort; the account is returned to the caller regardless
                        crate::error::record_ignored(
                            modules::account::set_current_account_id_with_target(
                                &matching.id,
                                current_target,
                            ),
                            "persist auto-bound current account",
                        );
                        return Ok(Some(matching));
                    }
                }
            }
        }
    }

    modules::logger::log_info("   No current account set");
    Ok(None)
}

/// 导出账号（包含 refresh_token）
use crate::models::AccountExportResponse;

#[tauri::command]
pub async fn export_accounts(account_ids: Vec<String>) -> Result<AccountExportResponse, String> {
    tokio::task::spawn_blocking(move || modules::account::export_accounts_by_ids(&account_ids))
        .await
        .unwrap_or_else(|_| Err("Task panicked".to_string()))
}

/// 内部辅助功能：在添加或导入账号后自动刷新一次额度
#[tauri::command]
pub async fn toggle_proxy_status(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
    enable: bool,
    reason: Option<String>,
) -> Result<(), String> {
    modules::logger::log_info(&format!(
        "切换账号反代状态: {} -> {}",
        account_id,
        if enable { "启用" } else { "禁用" }
    ));

    // 1-3. 更新账号文件并同步更新 index.json
    modules::account::toggle_proxy_status(&account_id, enable, reason.as_deref())?;

    modules::logger::log_info(&format!(
        "账号反代状态已更新: {} ({})",
        account_id,
        if enable { "已启用" } else { "已禁用" }
    ));

    // 4. 如果反代服务正在运行,立刻同步到内存池（避免禁用后仍被选中）
    {
        let instance_lock = proxy_state.instance.read().await;
        if let Some(instance) = instance_lock.as_ref() {
            // 如果禁用的是当前固定账号，则自动关闭固定模式（内存 + 配置持久化）
            if !enable {
                let pref_id = instance.token_manager.get_preferred_account().await;
                if pref_id.as_deref() == Some(&account_id) {
                    instance.token_manager.set_preferred_account(None).await;

                    if let Ok(mut cfg) = crate::modules::config::load_app_config() {
                        if cfg.proxy.preferred_account_id.as_deref() == Some(&account_id) {
                            cfg.proxy.preferred_account_id = None;
                            // Justification: config persistence of the cleared preferred-account; the in-memory token manager was already updated
                            crate::error::record_ignored(
                                crate::modules::config::save_app_config(&cfg),
                                "persist cleared preferred account to config",
                            );
                        }
                    }
                }
            }

            instance
                .token_manager
                .reload_account(&account_id)
                .await
                .map_err(|e| format!("同步账号失败: {}", e))?;
        }
    }

    // 5. 更新托盘菜单
    crate::modules::tray::update_tray_menus(&app);

    Ok(())
}

/// 预热所有可用账号
#[tauri::command]
pub async fn warm_up_all_accounts() -> Result<String, String> {
    modules::quota::warm_up_all_accounts().await
}

/// 预热指定账号
#[tauri::command]
pub async fn warm_up_account(account_id: String) -> Result<String, String> {
    modules::quota::warm_up_account(&account_id).await
}

/// Save account priority and apply it to the running proxy.
#[tauri::command]
pub async fn update_account_priority(
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    account_id: String,
    priority: u8,
) -> Result<(), String> {
    modules::account::update_account_priority(&account_id, priority)?;
    if let Some(instance) = proxy_state.instance.read().await.as_ref() {
        instance
            .token_manager
            .update_account_priority(&account_id, priority);
    }
    Ok(())
}

/// 更新账号自定义标签
#[tauri::command]
pub async fn update_account_label(account_id: String, label: String) -> Result<(), String> {
    // 验证标签长度（按字符数计算，支持中文）
    if label.chars().count() > 15 {
        return Err("标签长度不能超过15个字符".to_string());
    }

    modules::logger::log_info(&format!(
        "更新账号标签: {} -> {:?}",
        account_id,
        if label.is_empty() { "无" } else { &label }
    ));

    // 1. 读取账号文件
    let data_dir = modules::account::get_data_dir()?;
    let account_path = data_dir
        .join("accounts")
        .join(format!("{}.json", account_id));

    if !account_path.exists() {
        return Err(format!("账号文件不存在: {}", account_id));
    }

    let content =
        std::fs::read_to_string(&account_path).map_err(|e| format!("读取账号文件失败: {}", e))?;

    let mut account_json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析账号文件失败: {}", e))?;

    // 2. 更新 custom_label 字段
    if label.is_empty() {
        account_json["custom_label"] = serde_json::Value::Null;
    } else {
        account_json["custom_label"] = serde_json::Value::String(label.clone());
    }

    // 3. 保存到磁盘
    let json_str = serde_json::to_string_pretty(&account_json)
        .map_err(|e| format!("序列化账号数据失败: {}", e))?;
    std::fs::write(&account_path, json_str).map_err(|e| format!("写入账号文件失败: {}", e))?;

    modules::logger::log_info(&format!(
        "账号标签已更新: {} ({})",
        account_id,
        if label.is_empty() {
            "已清除".to_string()
        } else {
            label
        }
    ));

    Ok(())
}
