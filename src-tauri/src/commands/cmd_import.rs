use super::*;
use super::cmd_quota::internal_refresh_account_quota;

#[tauri::command]
pub async fn import_v1_accounts(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Vec<Account>, String> {
    let accounts = modules::migration::import_from_v1().await?;

    // 对导入的账号尝试刷新一波
    for mut account in accounts.clone() {
        // Justification: quota auto-refresh after v1 import is opportunistic; the import already succeeded
        crate::error::record_ignored(
            internal_refresh_account_quota(&app, &mut account).await,
            "auto-refresh quota after v1 import",
        );
    }

    // Reload token pool
    // Justification: proxy pool reload after v1 import is best-effort sync; the import already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after v1 import",
    );

    Ok(accounts)
}

#[tauri::command]
pub async fn import_from_db(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    target_ide: Option<String>,
) -> Result<Vec<Account>, String> {
    let imported_accounts =
        modules::migration::import_all_local_accounts(target_ide.as_deref()).await?;

    if let Some(first_acc) = imported_accounts.first() {
        let account_id = first_acc.id.clone();
        // Justification: current-account persistence is best-effort; the import already succeeded
        crate::error::record_ignored(
            modules::account::set_current_account_id_with_target(
                &account_id,
                target_ide.as_deref(),
            ),
            "set current account after DB import",
        );
    }

    for mut account in imported_accounts.clone() {
        // Justification: quota auto-refresh after DB import is opportunistic; the import already succeeded
        crate::error::record_ignored(
            internal_refresh_account_quota(&app, &mut account).await,
            "auto-refresh quota after DB import",
        );
    }

    crate::modules::tray::update_tray_menus(&app);
    // Justification: proxy pool reload after DB import is best-effort sync; the import already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after DB import",
    );

    Ok(imported_accounts)
}

#[tauri::command]
#[allow(dead_code)]
pub async fn import_custom_db(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    path: String,
) -> Result<Account, String> {
    // 调用重构后的自定义导入函数
    let mut account = modules::migration::import_from_custom_db_path(path).await?;

    // 自动设为当前账号
    let account_id = account.id.clone();
    modules::account::set_current_account_id(&account_id)?;

    // 自动触发刷新额度
    // Justification: quota auto-refresh after custom DB import is opportunistic; the import already succeeded
    crate::error::record_ignored(
        internal_refresh_account_quota(&app, &mut account).await,
        "auto-refresh quota after custom DB import",
    );

    // 刷新托盘图标展示
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    // Justification: proxy pool reload after custom DB import is best-effort sync; the import already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after custom DB import",
    );

    Ok(account)
}

static LAST_FAILED_SYNC_TOKEN: once_cell::sync::Lazy<std::sync::Mutex<Option<String>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

#[tauri::command]
pub async fn sync_account_from_db(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Option<Account>, String> {
    // Check if the current target is one we should not sync (like agy CLI)
    let index = modules::account::load_account_index()?;
    let current_target = index.current_target_ide.as_deref();
    if current_target == Some("agy") {
        modules::logger::log_info("Auto-sync skipped: current target is agy CLI");
        return Ok(None);
    }

    // 1. 获取 DB 中的 Refresh Token
    let db_refresh_token = match modules::migration::get_refresh_token_from_db(current_target) {
        Ok(token) => token,
        Err(e) => {
            modules::logger::log_info(&format!("自动同步跳过: {}", e));
            return Ok(None);
        }
    };

    // 1.1 检查是否是上一次已知导入失败的失效 Token，避免周期性死循环重试
    if let Ok(guard) = LAST_FAILED_SYNC_TOKEN.lock() {
        if let Some(ref failed_token) = *guard {
            if failed_token == &db_refresh_token {
                return Ok(None);
            }
        }
    }

    // 2. 获取 Manager 当前账号
    let curr_account = modules::account::get_current_account()?;

    // 3. 对比：如果 Refresh Token 相同，说明账号没变，无需导入
    if let Some(acc) = curr_account {
        if acc.token.refresh_token == db_refresh_token {
            // 账号未变，由于已经是周期性任务，直接返回
            return Ok(None);
        }
        modules::logger::log_info(&format!(
            "检测到账号切换 ({} -> DB新账号)，正在同步...",
            acc.email
        ));
    } else {
        modules::logger::log_info("检测到新登录账号，正在自动同步...");
    }

    // 4. 执行完整导入（非致命探针，若本地 Token 失效或无可导入账号，记录并返回 Ok(None)，杜绝弹窗打扰用户）
    let mut account = match modules::migration::import_from_db(current_target).await {
        Ok(acc) => {
            if let Ok(mut guard) = LAST_FAILED_SYNC_TOKEN.lock() {
                *guard = None;
            }
            acc
        }
        Err(e) => {
            if let Ok(mut guard) = LAST_FAILED_SYNC_TOKEN.lock() {
                *guard = Some(db_refresh_token);
            }
            modules::logger::log_info(&format!("自动同步跳过 (本地账号导入失败): {}", e));
            return Ok(None);
        }
    };

    // 既然是从数据库导入，自动将其设为 Manager 的当前账号并保留当前 target
    let account_id = account.id.clone();
    modules::account::set_current_account_id_with_target(&account_id, current_target)?;

    // 自动触发刷新额度
    // Justification: quota auto-refresh after DB sync is opportunistic; the sync already succeeded
    crate::error::record_ignored(
        internal_refresh_account_quota(&app, &mut account).await,
        "auto-refresh quota after account DB sync",
    );

    // 刷新托盘图标展示
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    // Justification: proxy pool reload after DB sync is best-effort sync; the sync already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after account DB sync",
    );

    Ok(Some(account))
}
