use crate::models::Account;
use crate::modules::{db, device, process, version};
use std::fs;

use super::*;

/// 桌面版实现：包含完整的进程控制 and UI 同步
pub struct DesktopIntegration {
    pub app_handle: Option<tauri::AppHandle>,
}

/// 写入账号凭据：>= 2.0.0 的原生应用走系统 Keyring，旧架构与定制 IDE 走 SQLite 注入。
///
/// 抽成独立函数是为了让「热切号」能在终止 language_server 子进程**之前**完成凭据写入
/// （见 `on_account_switch` 的顺序说明），同时让完整重启路径保持原有的「先杀后写」顺序。
pub(crate) fn apply_account_credentials(
    account: &Account,
    effective_target: Option<&str>,
    is_ide: bool,
    active_exe_path: Option<&std::path::Path>,
) -> Result<(), String> {
    let mut use_keyring = false;

    if !is_ide {
        // 经典原生版：自动探测版本号（优先使用预快照路径）
        match version::get_antigravity_version_with_path(
            effective_target,
            active_exe_path.as_deref(),
        ) {
            Ok(ver) => {
                // 如果版本号 >= 2.0.0
                if version::compare_version(&ver.short_version, "2.0.0") != std::cmp::Ordering::Less
                {
                    use_keyring = true;
                    crate::modules::logger::log_info(&format!(
                        "[Desktop] Detected Antigravity version {} >= 2.0.0, using system Keyring.",
                        ver.short_version
                    ));
                } else {
                    crate::modules::logger::log_info(&format!(
                        "[Desktop] Detected Antigravity version {} < 2.0.0, falling back to legacy SQLite injection.",
                        ver.short_version
                    ));
                }
            }
            Err(e) => {
                // 如果探测失败，优先检查本地是否存在可用的 SQLite 数据库 (state.vscdb)
                // 若存在数据库，说明是经典的 VS Code/IDE 架构，优先使用 SQLite 注入，防止无 secret-tool 时报错
                let has_sqlite_db = db::get_db_path(effective_target)
                    .map(|p| p.exists())
                    .unwrap_or(false);

                if has_sqlite_db {
                    use_keyring = false;
                    crate::modules::logger::log_info(&format!(
                        "[Desktop] Failed to detect Antigravity version ({}), but detected existing SQLite database. Falling back to SQLite injection.",
                        e
                    ));
                } else {
                    use_keyring = true;
                    crate::modules::logger::log_warn(&format!(
                        "[Desktop] Failed to detect Antigravity version ({}) and no SQLite database found, defaulting to system Keyring.",
                        e
                    ));
                }
            }
        }
    }

    if use_keyring {
        // ================== 最新版 Antigravity 原生应用逻辑 (>= 2.0.0) ==================
        // 2.1 写入系统 Keychain/Keyring
        if let Err(keyring_err) = write_to_system_keyring(account) {
            let db_fallback = if let Ok(db_path) = db::get_db_path(effective_target) {
                if db_path.exists() {
                    crate::modules::logger::log_warn(&format!(
                        "[Desktop] Keyring write failed ({}), but found SQLite DB at {:?}. Falling back to SQLite token injection.",
                        keyring_err, db_path
                    ));
                    let backup_path = db_path.with_extension("vscdb.backup");
                    // Justification: pre-injection database backup is best-effort; the injection proceeds regardless
                    crate::error::record_ignored(
                        fs::copy(&db_path, &backup_path),
                        "back up state.vscdb before fallback token injection",
                    );
                    db::inject_token(
                        &db_path,
                        &account.token.access_token,
                        &account.token.refresh_token,
                        account.token.expiry_timestamp,
                        &account.email,
                        account.token.is_gcp_tos,
                        account.token.project_id.as_deref(),
                        account.token.id_token.as_deref(),
                        account.token.oauth_client_key.as_deref(),
                        effective_target,
                    )
                    .map_err(|e| {
                        format!(
                            "Keyring write failed ({keyring_err}); SQLite fallback injection also failed: {e}"
                        )
                    })?;
                    if let Some(ref profile) = account.device_profile {
                        // Justification: service machine ID sync is auxiliary; the token injection already succeeded
                        crate::error::record_ignored(
                            db::write_service_machine_id(&db_path, &profile.mac_machine_id),
                            "sync service machine ID after fallback injection",
                        );
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if !db_fallback {
                return Err(keyring_err);
            }
        }

        // 2.2 同步写入 storage.json 与 state.vscdb（若存在），确保新版客户端与本地扩展状态完全一致
        if let Ok(storage_path) = device::get_storage_path(effective_target) {
            if let Some(ref profile) = account.device_profile {
                // Justification: device profile sync keeps local state consistent; the account is already in the system keyring
                crate::error::record_ignored(
                    device::write_profile(&storage_path, profile),
                    "sync device profile after keyring write",
                );
            }
        }
        if let Ok(db_path) = db::get_db_path(effective_target) {
            if db_path.exists() {
                let backup_path = db_path.with_extension("vscdb.backup");
                // Justification: pre-injection database backup is best-effort; the injection proceeds regardless
                crate::error::record_ignored(
                    fs::copy(&db_path, &backup_path),
                    "back up state.vscdb before token injection",
                );
                // Justification: secondary SQLite sync after a successful keyring write; the account is already injected
                crate::error::record_ignored(
                    db::inject_token(
                        &db_path,
                        &account.token.access_token,
                        &account.token.refresh_token,
                        account.token.expiry_timestamp,
                        &account.email,
                        account.token.is_gcp_tos,
                        account.token.project_id.as_deref(),
                        account.token.id_token.as_deref(),
                        account.token.oauth_client_key.as_deref(),
                        effective_target,
                    ),
                    "inject token into state.vscdb",
                );
                if let Some(ref profile) = account.device_profile {
                    // Justification: service machine ID sync is auxiliary; the keyring write already succeeded
                    crate::error::record_ignored(
                        db::write_service_machine_id(&db_path, &profile.mac_machine_id),
                        "sync service machine ID",
                    );
                }
            }
        }
    } else {
        // ================== 原有 Antigravity 旧版或定制 IDE 逻辑 (< 2.0.0) ==================
        // Justification: opportunistic keyring write for old IDEs; the SQLite injection below is the authoritative path
        crate::error::record_ignored(
            write_to_system_keyring(account),
            "write account to system keyring",
        );

        // 2.1 获取存储路径
        let storage_path = device::get_storage_path(effective_target)?;

        // 2.2 写入设备 Profile
        if let Some(ref profile) = account.device_profile {
            device::write_profile(&storage_path, profile)?;
        }

        // 2.3 数据库处理与 Token 注入
        let db_path = db::get_db_path(effective_target)?;
        if db_path.exists() {
            let backup_path = db_path.with_extension("vscdb.backup");
            // Justification: pre-injection database backup is best-effort; the injection proceeds regardless
            crate::error::record_ignored(
                fs::copy(&db_path, &backup_path),
                "back up state.vscdb before token injection",
            );
        }

        db::inject_token(
            &db_path,
            &account.token.access_token,
            &account.token.refresh_token,
            account.token.expiry_timestamp,
            &account.email,
            account.token.is_gcp_tos,
            account.token.project_id.as_deref(),
            account.token.id_token.as_deref(),
            account.token.oauth_client_key.as_deref(),
            effective_target,
        )?;

        // 2.4 同步 Service Machine ID 到数据库
        if let Some(ref profile) = account.device_profile {
            // Justification: service machine ID sync is auxiliary; the token injection already succeeded
            crate::error::record_ignored(
                db::write_service_machine_id(&db_path, &profile.mac_machine_id),
                "sync service machine ID",
            );
        }
    }

    Ok(())
}
