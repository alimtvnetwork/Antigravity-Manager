//! Full instance clone with options.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Copy/clone an existing profile with options (optionally copying open projects and recent paths)
pub fn copy_instance_with_options(
    source_id: &str,
    target_name: String,
    clone_mode: Option<&str>,
    copy_projects: bool,
) -> Result<InstanceConfig, String> {
    let resolved_source_id =
        resolve_instance_id(source_id).unwrap_or_else(|_| source_id.to_string());
    let registry = load_registry()?;
    let source = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_source_id || i.id == source_id)
        .ok_or_else(|| format!("Source instance {} not found", source_id))?
        .clone();

    let mut new_instance = create_instance(target_name)?;

    if source.extensions_dir.is_some() {
        new_instance.extensions_dir = source.extensions_dir.clone();
        if let Ok(mut reg) = load_registry() {
            if let Some(inst) = reg.instances.iter_mut().find(|i| i.id == new_instance.id) {
                inst.extensions_dir = source.extensions_dir.clone();
                // Justification: registry persistence is a write-through cache of already-updated in-memory state; retried on the next registry touch
                crate::error::record_ignored(save_registry(&reg), "persist instance registry");
            }
        }
    }

    let src_path = PathBuf::from(&source.data_dir);
    let dst_path = PathBuf::from(&new_instance.data_dir);

    let is_profile_only = clone_mode
        .map(|m| m.eq_ignore_ascii_case("profile"))
        .unwrap_or(false);

    let has_src = src_path.exists();
    if has_src {
        if is_profile_only {
            let user_settings_src = src_path.join("User");
            let has_user_dir = user_settings_src.exists();
            if has_user_dir {
                let user_settings_dst = dst_path.join("User");
                // Justification: best-effort tree copy; parity sync completes it on launch
                crate::error::record_ignored(
                    copy_dir_recursive(&user_settings_src, &user_settings_dst),
                    "copy directory tree",
                );
            }
        } else {
            // Full directory copy (default): copies entire instance data tree while skipping volatile locks/caches
            if let Err(err) = copy_dir_recursive(&src_path, &dst_path) {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Data-dir copy failed for {}: {}",
                    source.id, err
                ));
            }
        }

        if let Err(err) = copy_required_ide_files(&src_path, &dst_path) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Required settings copy failed: {}",
                err
            ));
        }

        // Specifically ensure state.vscdb is cloned using safe_clone_sqlite_db
        let src_vscdb = src_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        let dst_vscdb = dst_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        if src_vscdb.exists() {
            if let Err(e) = safe_clone_sqlite_db(&src_vscdb, &dst_vscdb) {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Failed to safe-clone state.vscdb: {}",
                    e
                ));
            }
        } else if source.is_default || source.id == "default" {
            let default_vscdb = get_default_antigravity_data_dir()
                .join("User")
                .join("globalStorage")
                .join("state.vscdb");
            if default_vscdb.exists() {
                // Justification: best-effort db clone; the instance re-derives state if the clone is missing
                crate::error::record_ignored(
                    safe_clone_sqlite_db(&default_vscdb, &dst_vscdb),
                    "clone sqlite db",
                );
            }
        }

        // Copy security_presets.json and antigravity_policies.json across all target directories
        let mut dst_user_targets = vec![dst_path.join("User")];
        #[cfg(target_os = "windows")]
        {
            if let Ok(dst_home) = get_instance_home_dir(&new_instance.id) {
                let dst_appdata_user = dst_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User");
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(
                    fs::create_dir_all(&dst_appdata_user),
                    "create directory",
                );
                dst_user_targets.push(dst_appdata_user);
            }
        }

        for file_name in &["security_presets.json", "antigravity_policies.json"] {
            let mut candidate_srcs = vec![src_path.join("User").join(file_name)];
            if let Some(src_home) = source_profile_home(&source) {
                #[cfg(target_os = "windows")]
                candidate_srcs.push(
                    src_home
                        .join("AppData")
                        .join("Roaming")
                        .join("Antigravity")
                        .join("User")
                        .join(file_name),
                );
            }
            candidate_srcs.push(
                get_default_antigravity_data_dir()
                    .join("User")
                    .join(file_name),
            );

            if let Some(found_src) = candidate_srcs.iter().find(|p| p.is_file()) {
                for target_dir in &dst_user_targets {
                    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                    crate::error::record_ignored(
                        fs::create_dir_all(target_dir),
                        "create directory",
                    );
                    // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
                    crate::error::record_ignored(
                        fs::copy(found_src, target_dir.join(file_name)),
                        "copy file",
                    );
                }
                crate::modules::logger::log_info(&format!(
                    "[Instance] Copied {} from {} to {} target user directories",
                    file_name,
                    found_src.display(),
                    dst_user_targets.len()
                ));
            }
        }

        purge_volatile_instance_sessions(&dst_path);

        let is_tos = new_instance
            .bound_account_id
            .as_ref()
            .and_then(|id| crate::modules::account::load_account(id).ok())
            .map(|a| a.token.is_gcp_tos)
            .unwrap_or(false);
        // Justification: best-effort storage sync; re-synced on every launch and account switch
        crate::error::record_ignored(
            update_instance_app_storage(&dst_path, new_instance.bound_email.as_deref(), is_tos),
            "sync app_storage.json",
        );
        // Justification: best-effort executable clone; launch falls back to the base executable
        crate::error::record_ignored(
            clone_instance_executable(&new_instance.id),
            "clone instance executable",
        );

        // Sanitize cloned session and reseed with newly bound account credentials
        let cloned_db = dst_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        let has_cloned_db = cloned_db.exists();
        if has_cloned_db {
            // Justification: defensive session cleanup; leftover session data is tolerated
            crate::error::record_ignored(
                crate::modules::db::sanitize_session(&cloned_db),
                "sanitize session db",
            );
            if let Some(ref acc_id) = new_instance.bound_account_id {
                if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                    if let Ok(new_home) = get_instance_home_dir(&new_instance.id) {
                        // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
                        crate::error::record_ignored(
                            crate::modules::integration::write_to_file_credentials_at(
                                &new_home, &acc,
                            ),
                            "write file credentials",
                        );
                        write_keyring_bypass_markers(&dst_path, Some(&new_home));
                    } else {
                        write_keyring_bypass_markers(&dst_path, None);
                    }
                    // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
                    crate::error::record_ignored(
                        crate::modules::integration::write_to_file_credentials_at(&dst_path, &acc),
                        "write file credentials",
                    );

                    // Justification: best-effort token injection; auth state is re-injected on the next launch
                    crate::error::record_ignored(
                        crate::modules::db::inject_token(
                            &cloned_db,
                            &acc.token.access_token,
                            &acc.token.refresh_token,
                            acc.token.expiry_timestamp,
                            &acc.email,
                            acc.token.is_gcp_tos,
                            acc.token.project_id.as_deref(),
                            acc.token.id_token.as_deref(),
                            acc.token.oauth_client_key.as_deref(),
                            None,
                        ),
                        "inject token into db",
                    );
                    let profile = acc
                        .device_profile
                        .clone()
                        .unwrap_or_else(crate::modules::device::generate_profile);
                    let storage_path = dst_path
                        .join("User")
                        .join("globalStorage")
                        .join("storage.json");
                    // Justification: device-profile bookkeeping; the existing profile persists if this fails
                    crate::error::record_ignored(
                        crate::modules::device::write_profile(&storage_path, &profile),
                        "write device profile",
                    );
                    // Justification: device-identity bookkeeping; the existing identity persists if this fails
                    crate::error::record_ignored(
                        crate::modules::db::write_service_machine_id(
                            &cloned_db,
                            &profile.mac_machine_id,
                        ),
                        "write machine id",
                    );

                    #[cfg(target_os = "windows")]
                    {
                        let appdata_db_dir = dst_path
                            .join("AppData")
                            .join("Roaming")
                            .join("Antigravity")
                            .join("User")
                            .join("globalStorage");
                        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                        crate::error::record_ignored(
                            fs::create_dir_all(&appdata_db_dir),
                            "create directory",
                        );
                        let appdata_db_path = appdata_db_dir.join("state.vscdb");
                        // Justification: best-effort token injection; auth state is re-injected on the next launch
                        crate::error::record_ignored(
                            crate::modules::db::inject_token(
                                &appdata_db_path,
                                &acc.token.access_token,
                                &acc.token.refresh_token,
                                acc.token.expiry_timestamp,
                                &acc.email,
                                acc.token.is_gcp_tos,
                                acc.token.project_id.as_deref(),
                                acc.token.id_token.as_deref(),
                                acc.token.oauth_client_key.as_deref(),
                                None,
                            ),
                            "inject token into db",
                        );
                        let storage_path = appdata_db_dir.join("storage.json");
                        // Justification: device-profile bookkeeping; the existing profile persists if this fails
                        crate::error::record_ignored(
                            crate::modules::device::write_profile(&storage_path, &profile),
                            "write device profile",
                        );
                        // Justification: device-identity bookkeeping; the existing identity persists if this fails
                        crate::error::record_ignored(
                            crate::modules::db::write_service_machine_id(
                                &appdata_db_path,
                                &profile.mac_machine_id,
                            ),
                            "write machine id",
                        );

                        if let Ok(new_home) = get_instance_home_dir(&new_instance.id) {
                            let home_appdata_db_dir = new_home
                                .join("AppData")
                                .join("Roaming")
                                .join("Antigravity")
                                .join("User")
                                .join("globalStorage");
                            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                            crate::error::record_ignored(
                                fs::create_dir_all(&home_appdata_db_dir),
                                "create directory",
                            );
                            let home_appdata_db_path = home_appdata_db_dir.join("state.vscdb");
                            // Justification: best-effort token injection; auth state is re-injected on the next launch
                            crate::error::record_ignored(
                                crate::modules::db::inject_token(
                                    &home_appdata_db_path,
                                    &acc.token.access_token,
                                    &acc.token.refresh_token,
                                    acc.token.expiry_timestamp,
                                    &acc.email,
                                    acc.token.is_gcp_tos,
                                    acc.token.project_id.as_deref(),
                                    acc.token.id_token.as_deref(),
                                    acc.token.oauth_client_key.as_deref(),
                                    None,
                                ),
                                "inject token into db",
                            );
                            let home_storage_path = home_appdata_db_dir.join("storage.json");
                            // Justification: device-profile bookkeeping; the existing profile persists if this fails
                            crate::error::record_ignored(
                                crate::modules::device::write_profile(&home_storage_path, &profile),
                                "write device profile",
                            );
                            // Justification: device-identity bookkeeping; the existing identity persists if this fails
                            crate::error::record_ignored(
                                crate::modules::db::write_service_machine_id(
                                    &home_appdata_db_path,
                                    &profile.mac_machine_id,
                                ),
                                "write machine id",
                            );
                        }
                    }
                }
            }
        }
    }

    if let Err(err) = copy_source_ide_trees(&source, &new_instance) {
        crate::modules::logger::log_warn(&format!(
            "[Instance] IDE settings/repo tree copy failed: {}",
            err
        ));
    }

    if let Err(err) = copy_instance_settings(&source.id, &new_instance.id) {
        crate::modules::logger::log_warn(&format!(
            "[Instance] Deep settings copy failed from '{}' to '{}': {}",
            source.id, new_instance.id, err
        ));
    }

    if copy_projects {
        // Justification: background detection; re-runs on the next refresh tick
        crate::error::record_ignored(
            crate::modules::repo_db::detect_running_projects(&source.id),
            "detect running projects",
        );
        if let Err(err) =
            crate::modules::repo_db::clone_instance_repo_rows(&source.id, &new_instance.id)
        {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Repo database clone failed: {}",
                err
            ));
        }
        if let Err(err) = copy_instance_projects(&source.id, &new_instance.id) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Workspace projects copy failed: {}",
                err
            ));
        }
    } else {
        let ws_main = dst_path.join("User").join("workspaceStorage");
        let has_ws = ws_main.exists();
        if has_ws {
            // Justification: cleanup of an optional directory tree; absence is the normal case
            crate::error::record_ignored(fs::remove_dir_all(&ws_main), "remove directory tree");
        }
        #[cfg(target_os = "windows")]
        {
            let ws_appdata = dst_path
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage");
            if ws_appdata.exists() {
                // Justification: cleanup of an optional directory tree; absence is the normal case
                crate::error::record_ignored(
                    fs::remove_dir_all(&ws_appdata),
                    "remove directory tree",
                );
            }
            if let Ok(dst_home) = get_instance_home_dir(&new_instance.id) {
                let ws_home = dst_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("workspaceStorage");
                if ws_home.exists() {
                    // Justification: cleanup of an optional directory tree; absence is the normal case
                    crate::error::record_ignored(
                        fs::remove_dir_all(&ws_home),
                        "remove directory tree",
                    );
                }
            }
        }
        purge_recent_project_paths(&new_instance);
    }

    // Justification: post-clone hygiene; stale summaries are regenerated on demand
    crate::error::record_ignored(
        sanitize_cloned_instance_summaries(&new_instance.id),
        "sanitize cloned summaries",
    );
    // Justification: settings injection is best-effort; the instance still launches with defaults
    crate::error::record_ignored(
        inject_instance_settings(&new_instance),
        "inject instance settings",
    );
    sync_instance_ide_parity(&new_instance.id).map_err(|e| e.to_string())?;

    Ok(new_instance)
}
