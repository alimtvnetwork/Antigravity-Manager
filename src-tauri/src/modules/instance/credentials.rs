//! Shared credential-injection pipeline for instance launches and account switches.
//!
//! Previously `switch_account_to_instance` (via its `inject_all_credentials`
//! closure) and `launch_instance_inner_with_extra_workspaces` each carried a
//! near-identical ~200-line injection block. They are unified here so a fix in
//! one path (e.g. the DB-lock retry) applies to both.

use super::*;
use crate::error::{AppError, AppResult};
use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Controls the small behavioral differences between the switch and launch
/// credential-injection call sites. The core injection sequence is identical;
/// only these call-site-specific steps differ.
pub(crate) struct CredentialInjectOptions {
    /// Launch path: seed system keyring + credential files unconditionally and
    /// adopt the account as current for the default instance.
    /// Switch path (`false`): keyring seeding is default-instance-only, the
    /// per-home `.gemini` scaffolding runs, and the global IDE storage sync
    /// runs for the default instance instead.
    pub launch_mode: bool,
}

impl CredentialInjectOptions {
    pub(crate) fn for_switch() -> Self {
        Self { launch_mode: false }
    }

    pub(crate) fn for_launch() -> Self {
        Self { launch_mode: true }
    }
}

/// Inject one account's credentials into every location an instance reads:
/// `state.vscdb` (primary + Windows AppData mirror + instance-home mirror),
/// `storage.json` device profile, OS keyring / credential files, and the
/// keyring-bypass markers. Stale `Local Storage` / `Session Storage` is wiped
/// so old cached sessions cannot persist.
///
/// The primary `state.vscdb` injection retries: a dying IDE may still hold the
/// SQLite lock for a moment after the kill. Callers decide severity: the
/// switch path propagates the error, the launch path treats it as best-effort.
pub(crate) fn inject_account_credentials(
    instance_id: &str,
    data_dir: &str,
    is_default: bool,
    account: &crate::models::Account,
    options: &CredentialInjectOptions,
) -> Result<(), String> {
    let target_data_path = PathBuf::from(data_dir);
    let is_tos = account.token.is_gcp_tos;

    // Justification: best-effort storage sync; re-synced on every launch and account switch
    crate::error::record_ignored(
        update_instance_app_storage(&target_data_path, Some(&account.email), is_tos),
        "sync app_storage.json",
    );
    purge_volatile_instance_sessions(&target_data_path);

    if options.launch_mode {
        // Launch path only: unconditionally sync credentials to system keyring
        // and credential file so the active Antigravity instance reads the bound account.
        // Justification: keyring seeding is best-effort; the file-credentials fallback covers failures
        crate::error::record_ignored(
            crate::modules::integration::write_to_system_keyring(account),
            "write system keyring",
        );
        // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
        crate::error::record_ignored(
            crate::modules::integration::write_to_file_credentials(account),
            "write file credentials",
        );
        if is_default {
            // Justification: account-state bookkeeping; in-memory state is already applied and re-persisted on next change
            crate::error::record_ignored(
                crate::modules::account::set_current_account_id(&account.id),
                "set current account id",
            );
        }
    }

    let inst_home_opt = get_instance_home_dir(instance_id).ok();
    if let Some(ref inst_home) = inst_home_opt {
        // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
        crate::error::record_ignored(
            crate::modules::integration::write_to_file_credentials_at(inst_home, account),
            "write file credentials",
        );
        if !options.launch_mode {
            #[cfg(target_os = "windows")]
            {
                let roaming = inst_home.join("AppData").join("Roaming");
                let local = inst_home.join("AppData").join("Local");
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(&roaming), "create directory");
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(&local), "create directory");
            }
            let gemini_ide_dir = inst_home.join(".gemini").join("antigravity-ide");
            let gemini_dir = inst_home.join(".gemini").join("antigravity");
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(&gemini_ide_dir), "create directory");
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(&gemini_dir), "create directory");
        }
        write_keyring_bypass_markers(&target_data_path, Some(inst_home));
    } else {
        write_keyring_bypass_markers(&target_data_path, None);
    }
    // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
    crate::error::record_ignored(
        crate::modules::integration::write_to_file_credentials_at(&target_data_path, account),
        "write file credentials",
    );
    if !options.launch_mode && is_default {
        // Justification: keyring seeding is best-effort; the file-credentials fallback covers failures
        crate::error::record_ignored(
            crate::modules::integration::write_to_system_keyring(account),
            "write system keyring",
        );
        // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
        crate::error::record_ignored(
            crate::modules::integration::write_to_file_credentials(account),
            "write file credentials",
        );
    }

    let db_dir = target_data_path.join("User").join("globalStorage");
    if !db_dir.exists() {
        fs::create_dir_all(&db_dir).map_err(|e| format!("Failed to create db dir: {}", e))?;
    }
    let db_path = db_dir.join("state.vscdb");

    // Retry the primary token injection: the dying IDE may still hold the
    // SQLite lock on state.vscdb for a moment after the kill.
    let mut inject_result = inject_token_once(&db_path, account, None);
    for retry in 1..=3 {
        if inject_result.is_ok() {
            break;
        }
        crate::modules::logger::log_warn(&format!(
            "[CREDENTIALS] Token injection attempt {} failed ({}); retrying after lock release",
            retry,
            inject_result
                .as_ref()
                .err()
                .map(|e| e.as_str())
                .unwrap_or("unknown"),
        ));
        std::thread::sleep(std::time::Duration::from_millis(400));
        inject_result = inject_token_once(&db_path, account, None);
    }
    inject_result?;

    if let Some(ref profile) = account.device_profile {
        // Justification: device-identity bookkeeping; the existing identity persists if this fails
        crate::error::record_ignored(
            crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id),
            "write machine id",
        );
        let storage_path = db_dir.join("storage.json");
        // Justification: device-profile bookkeeping; the existing profile persists if this fails
        crate::error::record_ignored(
            crate::modules::device::write_profile(&storage_path, profile),
            "write device profile",
        );
    }

    #[cfg(target_os = "windows")]
    if !is_default {
        inject_windows_appdata_credentials(&target_data_path, inst_home_opt.as_deref(), account);
    }

    if !options.launch_mode && is_default {
        // Only sync profile & token to global IDE storage when switching the default instance!
        for target_hint in [None, Some("ide")] {
            if let Ok(storage_path) = crate::modules::device::get_storage_path(target_hint) {
                if let Some(ref profile) = account.device_profile {
                    // Justification: device-profile bookkeeping; the existing profile persists if this fails
                    crate::error::record_ignored(
                        crate::modules::device::write_profile(&storage_path, profile),
                        "write device profile",
                    );
                }
            }
            if let Ok(global_db_path) = crate::modules::db::get_db_path(target_hint) {
                if global_db_path != db_path
                    && global_db_path.parent().map(|p| p.exists()).unwrap_or(false)
                {
                    // Justification: best-effort token injection; auth state is re-injected on the next launch
                    crate::error::record_ignored(
                        inject_token_once(&global_db_path, account, target_hint),
                        "inject token into db",
                    );
                    if let Some(ref profile) = account.device_profile {
                        // Justification: device-identity bookkeeping; the existing identity persists if this fails
                        crate::error::record_ignored(
                            crate::modules::db::write_service_machine_id(
                                &global_db_path,
                                &profile.mac_machine_id,
                            ),
                            "write machine id",
                        );
                    }
                }
            }
        }
    }

    // Wipe stale Local Storage / Session Storage to prevent old cached sessions from persisting
    for storage_name in ["Local Storage", "Session Storage"] {
        let storage_dir = target_data_path.join(storage_name);
        if storage_dir.exists() {
            // Justification: cleanup of an optional directory tree; absence is the normal case
            crate::error::record_ignored(fs::remove_dir_all(&storage_dir), "remove directory tree");
        }
    }

    Ok(())
}

/// Single attempt at injecting the OAuth token into one `state.vscdb`.
fn inject_token_once(
    db_path: &PathBuf,
    account: &crate::models::Account,
    target_hint: Option<&str>,
) -> Result<(), String> {
    crate::modules::db::inject_token(
        db_path,
        &account.token.access_token,
        &account.token.refresh_token,
        account.token.expiry_timestamp,
        &account.email,
        account.token.is_gcp_tos,
        account.token.project_id.as_deref(),
        account.token.id_token.as_deref(),
        account.token.oauth_client_key.as_deref(),
        target_hint,
    )
}

/// Windows-only: mirror the token + device profile into the per-instance
/// `AppData/Roaming/Antigravity/.../state.vscdb` and the instance-home mirror.
#[cfg(target_os = "windows")]
fn inject_windows_appdata_credentials(
    target_data_path: &Path,
    inst_home: Option<&Path>,
    account: &crate::models::Account,
) {
    let appdata_db_dir = target_data_path
        .join("AppData")
        .join("Roaming")
        .join("Antigravity")
        .join("User")
        .join("globalStorage");
    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
    crate::error::record_ignored(fs::create_dir_all(&appdata_db_dir), "create directory");
    let appdata_db_path = appdata_db_dir.join("state.vscdb");
    // Justification: best-effort token injection; auth state is re-injected on the next launch
    crate::error::record_ignored(
        inject_token_once(&appdata_db_path, account, None),
        "inject token into db",
    );
    if let Some(ref profile) = account.device_profile {
        // Justification: device-identity bookkeeping; the existing identity persists if this fails
        crate::error::record_ignored(
            crate::modules::db::write_service_machine_id(&appdata_db_path, &profile.mac_machine_id),
            "write machine id",
        );
        let storage_path = appdata_db_dir.join("storage.json");
        // Justification: device-profile bookkeeping; the existing profile persists if this fails
        crate::error::record_ignored(
            crate::modules::device::write_profile(&storage_path, profile),
            "write device profile",
        );
    }

    if let Some(inst_home) = inst_home {
        let home_appdata_db_dir = inst_home
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("globalStorage");
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(&home_appdata_db_dir), "create directory");
        let home_appdata_db_path = home_appdata_db_dir.join("state.vscdb");
        // Justification: best-effort token injection; auth state is re-injected on the next launch
        crate::error::record_ignored(
            inject_token_once(&home_appdata_db_path, account, None),
            "inject token into db",
        );
        if let Some(ref profile) = account.device_profile {
            // Justification: device-identity bookkeeping; the existing identity persists if this fails
            crate::error::record_ignored(
                crate::modules::db::write_service_machine_id(
                    &home_appdata_db_path,
                    &profile.mac_machine_id,
                ),
                "write machine id",
            );
            let home_storage_path = home_appdata_db_dir.join("storage.json");
            // Justification: device-profile bookkeeping; the existing profile persists if this fails
            crate::error::record_ignored(
                crate::modules::device::write_profile(&home_storage_path, profile),
                "write device profile",
            );
        }
    }
}
