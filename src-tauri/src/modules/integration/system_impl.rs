use crate::models::Account;
use crate::modules::{db, device, process, version};
use std::fs;

use super::*;

impl SystemIntegration for DesktopIntegration {
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String> {
        note_prompt_reinjected(false);
        crate::modules::logger::log_info(&format!(
            "[Desktop] Executing unified 5-step account switch for: {} (target_ide: {:?})",
            account.email, target_ide
        ));

        if let Some(target) = target_ide {
            if let Some(inst_id) = target.strip_prefix("instance:") {
                let res = Box::pin(crate::modules::instance::switch_account_to_instance(
                    &account.id,
                    Some(inst_id),
                ))
                .await;
                if let Some(ref h) = self.app_handle {
                    // Justification: update_tray_menus returns (); there is no error to surface
                    let _ = crate::modules::tray::update_tray_menus(h);
                }
                return res;
            }
        }

        if target_ide == Some("agy") {
            write_to_system_keyring(account)?;

            if let Ok(storage_path) = device::get_storage_path(target_ide) {
                if let Some(ref profile) = account.device_profile {
                    // Justification: device profile sync is auxiliary; the keyring write above is the authoritative op
                    crate::error::record_ignored(
                        device::write_profile(&storage_path, profile),
                        "sync device profile for agy",
                    );
                }
            }

            let is_running = process::is_process_running_by_name("agy");
            let msg = if is_running {
                format!(
                    "Account {} activated. Agy is running, token will be picked up automatically.",
                    account.email
                )
            } else {
                format!(
                    "Account {} activated. Token is ready for your next CLI command.",
                    account.email
                )
            };
            self.show_notification("Antigravity CLI", &msg);
            self.update_tray();

            return Ok(());
        }

        // Resolve target environment without misclassifying classic Antigravity.exe due to child language_server
        let ide_running = process::is_antigravity_running(Some("ide"));
        let classic_running = process::is_antigravity_running(None);
        let classic_exe = process::get_antigravity_executable_path(None);
        let ide_exe = process::get_antigravity_executable_path(Some("ide"));
        let ide_exe_str = ide_exe.as_ref().map(|p| p.to_string_lossy().to_string());

        let (is_ide, effective_target) = resolve_effective_target(
            target_ide,
            classic_running,
            ide_running,
            classic_exe.is_some(),
            ide_exe_str.as_deref(),
        );

        let active_exe_path = process::get_antigravity_executable_path(effective_target)
            .or_else(|| process::get_antigravity_executable_path(None));
        let active_args = process::get_args_from_running_process(effective_target)
            .or_else(|| process::get_args_from_running_process(None));

        // =========================================================================
        // STEP 1: Backup Running Prompts via AGM (before closing Antigravity IDE)
        // =========================================================================
        crate::modules::logger::log_info(
            "[Desktop] [Step 1/5] Backing up running prompts via AGM before closing Antigravity IDE...",
        );
        // Justification: prompt backup is best-effort; the switch proceeds and the reinject decision below is recomputed from the actual backed-up count
        crate::error::record_ignored(
            crate::modules::repo_db::backup_running_prompts("default"),
            "back up running prompts before account switch",
        );
        // Justification: prompt backup is best-effort; the switch proceeds and the reinject decision below is recomputed from the actual backed-up count
        crate::error::record_ignored(
            crate::modules::backup_prompts_db::backup_active_running_prompts_for_instance(
                Some("default"),
                None,
            ),
            "back up active running prompts before account switch",
        );
        let needs_reinject = crate::modules::repo_db::needs_prompt_channel_wait(
            crate::modules::repo_db::count_backed_up_prompts("default"),
        );

        // =========================================================================
        // STEP 2: Close the Antigravity IDE
        // =========================================================================
        crate::modules::logger::log_info(
            "[Desktop] [Step 2/5] Closing running Antigravity IDE processes...",
        );
        // Justification: IDE close is best-effort; the credential injection and relaunch below drive the outcome
        crate::error::record_ignored(
            crate::modules::instance::close_instance("default"),
            "close Antigravity IDE before account switch",
        );

        // =========================================================================
        // STEP 3: Switch the Account Credentials (OS Keyring + state.vscdb + storage.json)
        // =========================================================================
        crate::modules::logger::log_info(&format!(
            "[Desktop] [Step 3/5] Injecting switched account credentials for '{}'...",
            account.email
        ));
        apply_account_credentials(
            account,
            effective_target,
            is_ide,
            active_exe_path.as_deref(),
        )?;
        // Justification: instance binding is bookkeeping; the credentials were already injected
        crate::error::record_ignored(
            crate::modules::instance::bind_account_to_instance(
                "default",
                &account.id,
                &account.email,
            ),
            "bind account to default instance",
        );

        // Purge stale lockfiles in data_dir before process restart
        let mut target_data_dirs =
            vec![crate::modules::instance::get_default_antigravity_data_dir()];
        if let Ok(reg) = crate::modules::instance::load_registry() {
            if let Some(def) = reg
                .instances
                .iter()
                .find(|i| i.is_default || i.id == "default")
            {
                let p = std::path::PathBuf::from(&def.data_dir);
                if !target_data_dirs.contains(&p) {
                    target_data_dirs.push(p);
                }
            }
        }
        for dir in &target_data_dirs {
            for lock in &["lockfile", "code.lock", "DevToolsActivePort"] {
                let lock_file = dir.join(lock);
                if lock_file.exists() {
                    // Justification: stale lockfile removal is best-effort; the relaunch tolerates leftovers
                    crate::error::record_ignored(
                        fs::remove_file(&lock_file),
                        "purge stale lockfile",
                    );
                    crate::modules::logger::log_info(&format!(
                        "[Desktop] Purged stale lockfile: {:?}",
                        lock_file
                    ));
                }
            }
        }

        // =========================================================================
        // STEP 4: Re-Open / Re-Run the Antigravity IDE
        // =========================================================================
        crate::modules::logger::log_info(
            "[Desktop] [Step 4/5] Re-launching Antigravity IDE with workspace arguments...",
        );
        process::start_antigravity_with_fallback_path(
            effective_target,
            active_exe_path.as_deref(),
            active_args.as_deref(),
        )?;

        // =========================================================================
        // STEP 5: Re-Inject the Backed-Up Running Prompts (Asynchronous 5-Second Post-Launch Restoration)
        // =========================================================================
        if needs_reinject {
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                crate::modules::logger::log_info(
                    "[Desktop] [Step 5/5] 5s post-launch delay elapsed. Restoring prompts for instance default",
                );
                let workspace_roots =
                    crate::modules::instance::get_instance_workspace_paths("default");
                // Justification: post-launch prompt restore runs detached; failures are logged and retried by the scheduler
                crate::error::record_ignored(
                    crate::modules::instance::restore_and_inject_prompts_for_instance(
                        "default",
                        &workspace_roots,
                    ),
                    "restore and inject prompts after relaunch",
                );
                // Justification: post-launch prompt restore runs detached; failures are logged and retried by the scheduler
                crate::error::record_ignored(
                    crate::modules::backup_prompts_db::restore_running_prompts_for_instance(
                        Some("default"),
                        false,
                        None,
                    ),
                    "restore running prompts from backup after relaunch",
                );
                // Justification: post-launch prompt restore runs detached; failures are logged and retried by the scheduler
                crate::error::record_ignored(
                    crate::modules::repo_db::dispatch_running_prompts("default"),
                    "dispatch running prompts after relaunch",
                );
            });
            note_prompt_reinjected(true);
        } else {
            crate::modules::logger::log_info(
                "[Desktop] [Step 5/5] No backed-up prompt; skipping prompt restoration",
            );
            note_prompt_reinjected(false);
        }
        // Justification: focus returns a found-flag, not a Result; false is expected on headless and there is no recovery
        let _ = crate::modules::process::focus_antigravity_window(effective_target);

        if let Some(ref h) = self.app_handle {
            // Justification: update_tray_menus returns (); there is no error to surface
            let _ = crate::modules::tray::update_tray_menus(h);
        }

        Ok(())
    }

    pub(crate) fn update_tray(&self) {
        if let Some(ref h) = self.app_handle {
            // Justification: update_tray_menus returns (); there is no error to surface
            let _ = crate::modules::tray::update_tray_menus(h);
        }
    }

    pub(crate) fn show_notification(&self, title: &str, body: &str) {
        // 使用 tauri-plugin-dialog 或原生通知（此处简化）
        crate::modules::logger::log_info(&format!("[Notification] {}: {}", title, body));
    }
}
