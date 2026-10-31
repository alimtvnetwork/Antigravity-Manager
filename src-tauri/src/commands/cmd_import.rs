use super::cmd_quota::internal_refresh_account_quota;
use super::*;

#[tauri::command]
pub async fn import_v1_accounts(
    app: tauri::AppHandle,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<Vec<Account>, String> {
    let accounts = modules::migration::import_from_v1().await?;

    // Try refreshing the imported account
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
    // Call the refactored custom import function
    let mut account = modules::migration::import_from_custom_db_path(path).await?;

    // Automatically set as current account
    let account_id = account.id.clone();
    modules::account::set_current_account_id(&account_id)?;

    // Automatically trigger refresh quota
    // Justification: quota auto-refresh after custom DB import is opportunistic; the import already succeeded
    crate::error::record_ignored(
        internal_refresh_account_quota(&app, &mut account).await,
        "auto-refresh quota after custom DB import",
    );

    // Refresh tray icon display
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

    // 1. Get the Refresh Token in DB
    let db_refresh_token = match modules::migration::get_refresh_token_from_db(current_target) {
        Ok(token) => token,
        Err(e) => {
            modules::logger::log_info(&format!("Automatic sync skip: {}", e));
            return Ok(None);
        }
    };

    // 1.1 Check whether it is an invalid token that failed the last known import to avoid periodic infinite loop retries.
    if let Ok(guard) = LAST_FAILED_SYNC_TOKEN.lock() {
        if let Some(ref failed_token) = *guard {
            if failed_token == &db_refresh_token {
                return Ok(None);
            }
        }
    }

    // 2. Get Manager’s current account
    let curr_account = modules::account::get_current_account()?;

    // 3. Comparison: If the Refresh Token is the same, it means that the account has not changed and there is no need to import it.
    if let Some(acc) = curr_account {
        if acc.token.refresh_token == db_refresh_token {
            // The account has not changed. Since it is already a periodic task, it will be returned directly.
            return Ok(None);
        }
        modules::logger::log_info(&format!(
            "Account switching detected ( {} -> DB new account), synchronizing...",
            acc.email
        ));
    } else {
        modules::logger::log_info("New login account detected, automatically synchronizing...");
    }

    // 4. Perform a complete import (non-fatal probe, if the local Token is invalid or there is no imported account, record and return Ok (None) to prevent pop-ups from disturbing the user)
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
            modules::logger::log_info(&format!(
                "Automatic synchronization skipped (local account import failed): {}",
                e
            ));
            return Ok(None);
        }
    };

    // Since it is imported from the database, it is automatically set to the current account of the Manager and the current target is retained.
    let account_id = account.id.clone();
    modules::account::set_current_account_id_with_target(&account_id, current_target)?;

    // Automatically trigger refresh quota
    // Justification: quota auto-refresh after DB sync is opportunistic; the sync already succeeded
    crate::error::record_ignored(
        internal_refresh_account_quota(&app, &mut account).await,
        "auto-refresh quota after account DB sync",
    );

    // Refresh tray icon display
    crate::modules::tray::update_tray_menus(&app);

    // Reload token pool
    // Justification: proxy pool reload after DB sync is best-effort sync; the sync already succeeded
    crate::error::record_ignored(
        crate::commands::proxy::reload_proxy_accounts(proxy_state).await,
        "reload proxy accounts after account DB sync",
    );

    Ok(Some(account))
}
