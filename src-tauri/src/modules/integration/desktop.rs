use crate::models::Account;
use crate::modules::{db, device, process, version};
use std::fs;

use super::*;

/// Desktop version implementation: includes complete process control and UI synchronization
pub struct DesktopIntegration {
    pub app_handle: Option<tauri::AppHandle>,
}

/// Write account credentials: Native applications >= 2.0.0 use system Keyring, old architecture and customized IDE use SQLite injection.
///
/// The purpose of extracting a separate function is to allow the "eager number" to complete writing the credentials before terminating the language_server child process.
/// (See the order description of `on_account_switch`), while keeping the complete restart path in the original "kill first, write later" order.
pub(crate) fn apply_account_credentials(
    account: &Account,
    effective_target: Option<&str>,
    is_ide: bool,
    active_exe_path: Option<&std::path::Path>,
) -> Result<(), String> {
    let mut use_keyring = false;

    if !is_ide {
        // Classic native version: automatically detects the version number (preferring to use the pre-snapshot path)
        match version::get_antigravity_version_with_path(
            effective_target,
            active_exe_path.as_deref(),
        ) {
            Ok(ver) => {
                // If version number >= 2.0.0
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
                // If the detection fails, first check whether there is an available local SQLite database (state.vscdb)
                // If a database exists, it means it is a classic VS Code/IDE architecture. SQLite injection is preferred to prevent errors without secret-tool.
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
        // ================== Latest version of Antigravity native application logic (>= 2.0.0) ==================
        // 2.1 Write to system Keychain/Keyring
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

        // 2.2 Synchronously write storage.json and state.vscdb (if it exists) to ensure that the new version of the client is completely consistent with the local extension state
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
        // ================== Legacy Antigravity legacy or custom IDE logic (< 2.0.0) ==================
        // Justification: opportunistic keyring write for old IDEs; the SQLite injection below is the authoritative path
        crate::error::record_ignored(
            write_to_system_keyring(account),
            "write account to system keyring",
        );

        // 2.1 Get the storage path
        let storage_path = device::get_storage_path(effective_target)?;

        // 2.2 Write device Profile
        if let Some(ref profile) = account.device_profile {
            device::write_profile(&storage_path, profile)?;
        }

        // 2.3 Database processing and Token injection
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

        // 2.4 Synchronize Service Machine ID to database
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
