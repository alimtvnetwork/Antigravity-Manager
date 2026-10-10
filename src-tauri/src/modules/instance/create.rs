//! Instance creation and the stop primitive.
use super::*;
use std::fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Create a new isolated profile with optional specific account binding
pub fn create_instance_with_account(
    name: String,
    target_acc: Option<&str>,
) -> Result<InstanceConfig, String> {
    let mut registry = load_registry()?;
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let slug: String = trimmed_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let sanitized_slug = slug.trim_matches('-').to_string();

    let now = chrono::Utc::now().timestamp();
    let instance_id = format!("{}-{}", sanitized_slug, now % 10000);

    let instances_root = get_instances_dir()?;
    let instance_data_dir = instances_root.join(&instance_id).join("data");
    let instance_home_dir = instances_root.join(&instance_id).join("home");

    fs::create_dir_all(&instance_data_dir)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    fs::create_dir_all(&instance_home_dir)
        .map_err(|e| format!("Failed to create instance home directory: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let roaming = instance_home_dir.join("AppData").join("Roaming");
        let local = instance_home_dir.join("AppData").join("Local");
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(&roaming), "create directory");
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(&local), "create directory");
    }
    let gemini_ide_dir = instance_home_dir.join(".gemini").join("antigravity-ide");
    let gemini_dir = instance_home_dir.join(".gemini").join("antigravity");
    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
    crate::error::record_ignored(fs::create_dir_all(&gemini_ide_dir), "create directory");
    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
    crate::error::record_ignored(fs::create_dir_all(&gemini_dir), "create directory");

    // Pre-seed app_storage.json with ide-install-wizard-shown: true to skip onboarding wizard
    // Justification: best-effort storage sync; re-synced on every launch and account switch
    crate::error::record_ignored(
        update_instance_app_storage(&instance_data_dir, None, false),
        "sync app_storage.json",
    );

    let user_dir = instance_data_dir.join("User");
    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
    crate::error::record_ignored(fs::create_dir_all(&user_dir), "create directory");
    let default_dir = get_default_antigravity_data_dir();
    let default_settings = default_dir.join("User").join("settings.json");
    let dest_settings = user_dir.join("settings.json");
    if default_settings.exists() && !dest_settings.exists() {
        // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
        crate::error::record_ignored(fs::copy(default_settings, dest_settings), "copy file");
    }

    // Also copy security_presets.json and antigravity_policies.json from default directory if they exist
    for file_name in &["security_presets.json", "antigravity_policies.json"] {
        let default_file = default_dir.join("User").join(file_name);
        let dest_file = user_dir.join(file_name);
        if default_file.exists() && !dest_file.exists() {
            // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
            crate::error::record_ignored(fs::copy(&default_file, &dest_file), "copy file");
        }
        #[cfg(target_os = "windows")]
        {
            let roaming_user = instance_home_dir
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User");
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(&roaming_user), "create directory");
            let roaming_dest = roaming_user.join(file_name);
            if default_file.exists() && !roaming_dest.exists() {
                // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
                crate::error::record_ignored(fs::copy(&default_file, &roaming_dest), "copy file");
            }
        }
    }

    let next_seq = registry
        .instances
        .iter()
        .filter_map(|i| i.seq_num)
        .max()
        .unwrap_or(0)
        + 1;

    let mut bound_acc_id = None;
    let mut bound_acc_email = None;
    if let Ok(accounts) = crate::modules::account::list_accounts() {
        let bound_ids: std::collections::HashSet<String> = registry
            .instances
            .iter()
            .filter_map(|i| i.bound_account_id.clone())
            .collect();

        let candidate = if let Some(query) = target_acc.map(str::trim).filter(|q| !q.is_empty()) {
            let q_lower = query.to_lowercase();
            accounts
                .iter()
                .find(|a| {
                    a.id == query
                        || a.email.to_lowercase() == q_lower
                        || a.email.to_lowercase().contains(&q_lower)
                })
                .or_else(|| accounts.first())
        } else {
            accounts
                .iter()
                .find(|a| !bound_ids.contains(&a.id))
                .or_else(|| accounts.first())
        };

        if let Some(acc) = candidate {
            bound_acc_id = Some(acc.id.clone());
            bound_acc_email = Some(acc.email.clone());

            // Justification: best-effort storage sync; re-synced on every launch and account switch
            crate::error::record_ignored(
                update_instance_app_storage(
                    &instance_data_dir,
                    Some(&acc.email),
                    acc.token.is_gcp_tos,
                ),
                "sync app_storage.json",
            );
            purge_volatile_instance_sessions(&instance_data_dir);
            write_keyring_bypass_markers(&instance_data_dir, Some(&instance_home_dir));

            // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
            crate::error::record_ignored(
                crate::modules::integration::write_to_file_credentials_at(&instance_home_dir, acc),
                "write file credentials",
            );
            // Justification: credential seeding is best-effort; the account flow re-derives credentials on demand
            crate::error::record_ignored(
                crate::modules::integration::write_to_file_credentials_at(&instance_data_dir, acc),
                "write file credentials",
            );

            let db_dir = instance_data_dir.join("User").join("globalStorage");
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(&db_dir), "create directory");
            let db_path = db_dir.join("state.vscdb");
            // Justification: best-effort token injection; auth state is re-injected on the next launch
            crate::error::record_ignored(
                crate::modules::db::inject_token(
                    &db_path,
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
            let storage_path = db_dir.join("storage.json");
            // Justification: device-profile bookkeeping; the existing profile persists if this fails
            crate::error::record_ignored(
                crate::modules::device::write_profile(&storage_path, &profile),
                "write device profile",
            );
            // Justification: device-identity bookkeeping; the existing identity persists if this fails
            crate::error::record_ignored(
                crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id),
                "write machine id",
            );

            #[cfg(target_os = "windows")]
            {
                let appdata_db_dir = instance_data_dir
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

                let home_appdata_db_dir = instance_home_dir
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

    let config = InstanceConfig {
        id: instance_id.clone(),
        name: trimmed_name.to_string(),
        data_dir: instance_data_dir.to_string_lossy().to_string(),
        executable_path: None,
        extensions_dir: None,
        bound_account_id: bound_acc_id,
        bound_email: bound_acc_email,
        created_at: now,
        last_used: now,
        is_default: false,
        pid: None,
        seq_num: Some(next_seq),
    };

    registry.instances.push(config.clone());
    save_registry(&registry)?;

    let cloned_exe = clone_instance_executable(&instance_id).ok();
    let config = if let Some(exe) = cloned_exe {
        let mut c = config;
        c.executable_path = Some(exe);
        c
    } else {
        config
    };

    // Justification: settings injection is best-effort; the instance still launches with defaults
    crate::error::record_ignored(
        inject_instance_settings(&config),
        "inject instance settings",
    );
    sync_instance_ide_parity(&instance_id).map_err(|e| e.to_string())?;

    Ok(config)
}

/// Create a new isolated profile (defaults to next available account)
pub fn create_instance(name: String) -> Result<InstanceConfig, String> {
    create_instance_with_account(name, None)
}

/// Stop an instance process cleanly
pub fn stop_instance(instance_id: &str) -> Result<(), String> {
    close_instance(instance_id)
}
