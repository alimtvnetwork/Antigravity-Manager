use super::*;

pub fn run() {
    // Check for terminal CLI profile management arguments
    if modules::cli::handle_cli_arguments() {
        return;
    }

    // Disable Windows background throttling/EcoQoS and Chromium background occlusion
    #[cfg(target_os = "windows")]
    {
        windows_api::disable_efficiency_mode();
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--disable-features=CalculateNativeWinOcclusion,CalculateNativeWindowOcclusion --disable-backgrounding-occluded-windows --disable-renderer-backgrounding --disable-background-timer-throttling",
        );
    }

    // Check for headless mode
    let args: Vec<String> = std::env::args().collect();
    let is_headless = args.iter().any(|arg| arg == "--headless");

    // Increase file descriptor limit (macOS only)
    #[cfg(target_os = "macos")]
    increase_nofile_limit();

    // Initialize logger
    logger::init_logger();

    #[cfg(target_os = "linux")]
    configure_linux_graphics();

    init_databases();
    if is_headless {
        run_headless();
        return;
    }

    let tray_enabled = should_enable_tray();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::all()
                        .difference(tauri_plugin_window_state::StateFlags::VISIBLE),
                )
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                modules::lightweight::exit_lightweight_mode(app),
                "exit_lightweight_mode",
            );
            // Justification: runs only when the main window exists; the Option result is intentionally unused.
            let _ = app.get_webview_window("main").map(|window| {
                restore_and_focus_window(&window);
            });
        }))
        .manage(commands::proxy::ProxyServiceState::new())
        .manage(commands::cloudflared::CloudflaredState::new())
        .manage(AppRuntimeFlags { tray_enabled })
        .setup(|app| setup_app(app, tray_enabled))
        .on_window_event(handle_window_event)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Account management commands
            commands::list_accounts,
            commands::list_task_history,
            commands::get_task_history_detail,
            commands::add_account,
            commands::delete_account,
            commands::delete_accounts,
            commands::reorder_accounts,
            commands::switch_account,
            commands::export_accounts,
            // Device fingerprint
            commands::get_device_profiles,
            commands::bind_device_profile,
            commands::bind_device_profile_with_profile,
            commands::preview_generate_profile,
            commands::apply_device_profile,
            commands::restore_original_device,
            commands::list_device_versions,
            commands::restore_device_version,
            commands::delete_device_version,
            commands::open_device_folder,
            commands::get_current_account,
            // Quota commands
            commands::fetch_account_quota,
            commands::refresh_all_quotas,
            // Config commands
            commands::load_config,
            commands::save_config,
            commands::get_training_api_status,
            commands::set_training_api_status,
            commands::get_training_telemetry,
            // Additional commands
            commands::prepare_oauth_url,
            commands::start_oauth_login,
            commands::complete_oauth_login,
            commands::cancel_oauth_login,
            commands::submit_oauth_code,
            commands::list_oauth_clients,
            commands::get_active_oauth_client,
            commands::set_active_oauth_client,
            commands::import_v1_accounts,
            commands::import_from_db,
            commands::import_custom_db,
            commands::sync_account_from_db,
            commands::save_text_file,
            commands::read_text_file,
            commands::clear_log_cache,
            commands::clear_antigravity_cache,
            commands::get_antigravity_cache_paths,
            commands::preflight_antigravity_clean,
            commands::prune_antigravity_conversations,
            commands::prune_antigravity_conversations_only,
            commands::undo_antigravity_prune,
            commands::open_data_folder,
            commands::get_data_dir_path,
            commands::set_data_dir,
            commands::migrate_data_dir,
            commands::show_main_window,
            commands::set_window_theme,
            commands::get_antigravity_path,
            commands::get_antigravity_cli_path,
            commands::get_antigravity_args,
            commands::check_for_updates,
            commands::check_update_via_script,
            commands::run_installer_update,
            commands::check_native_update,
            commands::check_homebrew_installation,
            commands::check_appimage_installation,
            commands::brew_upgrade_cask,
            commands::get_update_settings,
            commands::save_update_settings,
            commands::should_check_updates,
            commands::should_check_updates_on_startup,
            commands::update_last_check_time,
            commands::toggle_proxy_status,
            // Proxy service commands
            commands::proxy::start_proxy_service,
            commands::proxy::stop_proxy_service,
            commands::proxy::get_proxy_status,
            commands::proxy::get_proxy_stats,
            commands::proxy::get_proxy_logs,
            commands::proxy::get_proxy_logs_paginated,
            commands::proxy::get_proxy_log_detail,
            commands::proxy::get_proxy_logs_count,
            commands::proxy::export_proxy_logs,
            commands::proxy::export_proxy_logs_json,
            commands::proxy::get_proxy_logs_count_filtered,
            commands::proxy::get_proxy_logs_filtered,
            commands::proxy::set_proxy_monitor_enabled,
            commands::proxy::set_proxy_capture_health_logs,
            commands::proxy::clear_proxy_logs,
            commands::proxy::clear_thinking_store,
            commands::proxy::get_thinking_store_count,
            commands::proxy::get_proxy_db_disk_size,
            commands::proxy::generate_api_key,
            commands::proxy::reload_proxy_accounts,
            commands::proxy::update_model_mapping,
            commands::proxy::check_proxy_health,
            commands::proxy::get_proxy_pool_config,
            commands::proxy::fetch_zai_models,
            commands::proxy::get_proxy_scheduling_config,
            commands::proxy::update_proxy_scheduling_config,
            commands::proxy::clear_proxy_session_bindings,
            commands::proxy::set_preferred_account,
            commands::proxy::get_preferred_account,
            commands::proxy::clear_proxy_rate_limit,
            commands::proxy::clear_all_proxy_rate_limits,
            // Proxy Pool Binding commands
            commands::proxy_pool::bind_account_proxy,
            commands::proxy_pool::unbind_account_proxy,
            commands::proxy_pool::get_account_proxy_binding,
            commands::proxy_pool::get_all_account_bindings,
            // Autostart commands
            commands::autostart::toggle_auto_launch,
            commands::autostart::is_auto_launch_enabled,
            // Warmup commands
            commands::warm_up_all_accounts,
            commands::warm_up_account,
            commands::update_account_label,
            commands::update_account_priority,
            // HTTP API settings commands
            commands::get_http_api_settings,
            commands::save_http_api_settings,
            // Token statistics commands
            commands::get_token_stats_hourly,
            commands::get_token_stats_daily,
            commands::get_token_stats_weekly,
            commands::get_token_stats_by_account,
            commands::get_token_stats_summary,
            commands::get_token_stats_by_model,
            commands::get_token_stats_model_trend_hourly,
            commands::get_token_stats_model_trend_daily,
            commands::get_token_stats_account_trend_hourly,
            commands::get_token_stats_account_trend_daily,
            proxy::cli_sync::get_cli_sync_status,
            proxy::cli_sync::execute_cli_sync,
            proxy::cli_sync::execute_cli_restore,
            proxy::cli_sync::get_cli_config_content,
            proxy::opencode_sync::get_opencode_sync_status,
            proxy::opencode_sync::get_opencode_providers,
            proxy::opencode_sync::get_canonical_families,
            proxy::opencode_sync::execute_opencode_sync,
            proxy::opencode_sync::execute_opencode_openai_sync,
            proxy::opencode_sync::execute_opencode_remove_provider,
            proxy::opencode_sync::execute_opencode_restore,
            proxy::opencode_sync::get_opencode_config_content,
            proxy::opencode_sync::execute_opencode_clear,
            proxy::hermes_sync::get_hermes_sync_status,
            proxy::hermes_sync::execute_hermes_sync,
            proxy::hermes_sync::execute_hermes_restore,
            proxy::hermes_sync::execute_hermes_clear,
            proxy::hermes_sync::get_hermes_config_content,
            proxy::openclaw_sync::get_openclaw_sync_status,
            proxy::openclaw_sync::execute_openclaw_sync,
            proxy::openclaw_sync::execute_openclaw_restore,
            proxy::openclaw_sync::execute_openclaw_clear,
            proxy::openclaw_sync::get_openclaw_config_content,
            proxy::droid_sync::get_droid_sync_status,
            proxy::droid_sync::execute_droid_sync,
            proxy::droid_sync::execute_droid_restore,
            proxy::droid_sync::get_droid_config_content,
            // Security/IP monitoring commands
            commands::security::get_ip_access_logs,
            commands::security::get_ip_stats,
            commands::security::get_ip_token_stats,
            commands::security::clear_ip_access_logs,
            commands::security::get_ip_blacklist,
            commands::security::add_ip_to_blacklist,
            commands::security::remove_ip_from_blacklist,
            commands::security::clear_ip_blacklist,
            commands::security::check_ip_in_blacklist,
            commands::security::get_ip_whitelist,
            commands::security::add_ip_to_whitelist,
            commands::security::remove_ip_from_whitelist,
            commands::security::clear_ip_whitelist,
            commands::security::check_ip_in_whitelist,
            commands::security::get_security_config,
            commands::security::update_security_config,
            // Cloudflared commands
            commands::cloudflared::cloudflared_check,
            commands::cloudflared::cloudflared_install,
            commands::cloudflared::cloudflared_start,
            commands::cloudflared::cloudflared_stop,
            commands::cloudflared::cloudflared_get_status,
            // Debug console commands
            modules::log_bridge::enable_debug_console,
            modules::log_bridge::disable_debug_console,
            modules::log_bridge::is_debug_console_enabled,
            modules::log_bridge::get_debug_console_logs,
            modules::log_bridge::clear_debug_console_logs,
            // User Token commands
            commands::user_token::list_user_tokens,
            commands::user_token::create_user_token,
            commands::user_token::update_user_token,
            commands::user_token::delete_user_token,
            commands::user_token::renew_user_token,
            commands::user_token::get_token_ip_bindings,
            commands::user_token::get_user_token_summary,
            // Patch commands
            commands::patch_agy_binary,
            // Multi-Instance Profile commands
            commands::list_instances,
            commands::create_instance,
            commands::copy_instance,
            commands::copy_instance_projects,
            commands::copy_instance_settings,
            commands::enforce_default_settings,
            commands::set_instance_turbo_mode,
            commands::set_instance_plan_review,
            commands::export_instance_settings,
            commands::import_instance_settings,
            commands::count_instances,
            commands::rename_instance,
            commands::delete_instance,
            commands::wipe_instance_session,
            commands::launch_instance,
            commands::focus_or_launch_instance,
            commands::focus_instance_workspace,
            commands::clone_instance_executable,
            commands::set_instance_executable,
            commands::close_instance,
            commands::stop_instance,
            commands::restart_instance,
            commands::fast_forward_instance,
            commands::get_active_instance,
            commands::set_active_instance,
            commands::set_default_instance,
            commands::switch_account_to_instance,
            commands::export_instances_json,
            commands::import_instances_json,
            commands::get_auto_switcher_status,
            commands::get_auto_switcher_daemon_status,
            commands::get_auto_switcher_config,
            commands::update_auto_switcher_config,
            commands::toggle_auto_switcher,
            commands::trigger_manual_profile_rotation,
            commands::list_running_projects,
            commands::list_backed_up_prompts,
            commands::restore_prompts_backup,
            commands::clean_and_restart_workspace,
            commands::resume_recent_project_prompts,
            commands::send_prompt_now,
            commands::enqueue_prompt,
            commands::get_running_instances_process_count,
            commands::assign_project_to_instance,
            commands::get_instance_workspace_folders,
            commands::get_project_conversation_tree,
            commands::get_instance_switch_history,
            commands::get_instance_audit_trail,
            commands::sync_instance_pid_and_quota,
            commands::sync_all_instances_and_quotas,
            // Email and Mailbox Management commands
            commands::get_email_settings,
            commands::save_email_settings,
            commands::list_email_accounts,
            commands::add_email_account,
            commands::update_email_account,
            commands::delete_email_account,
            commands::set_default_email_account,
            commands::list_notify_recipients,
            commands::add_notify_recipient,
            commands::delete_notify_recipient,
            commands::test_smtp_connection,
            commands::test_imap_connection,
            commands::test_direct_email_connection,
            commands::export_email_data,
            commands::import_email_data,
            commands::backup_email_db,
            commands::restore_email_db,
            commands::get_email_watcher_status,
            commands::trigger_manual_email_check,
            commands::dispatch_email_test_ping,
            commands::dispatch_custom_email_task,
            commands::test_execute_cli_command,
            // Supabase Cross-Node Synchronization commands
            commands::get_supabase_config,
            commands::save_supabase_config,
            commands::test_supabase_endpoint,
            commands::check_supabase_endpoint_tables,
            commands::migrate_supabase_data,
            commands::get_supabase_schema_sql,
            commands::export_supabase_config,
            commands::import_supabase_config,
            commands::acquire_account_lease,
            commands::release_account_lease,
            commands::list_active_account_leases,
            commands::get_local_node_info,
            commands::auto_discover_supabase_credentials,
            commands::get_fleet_machines,
            commands::supabase::get_supabase_fleet_machines,
            commands::supabase::sync_supabase_now,
            // Telegram Inbound Watcher commands
            commands::get_telegram_config,
            commands::save_telegram_config,
            commands::test_telegram_bot,
            commands::detect_telegram_chat_id,
            commands::get_telegram_status,
            commands::send_telegram_test_message,
            // Window Management commands
            commands::minimize_window,
            commands::maximize_window,
            commands::toggle_maximize_window,
            commands::close_window,
            commands::is_window_maximized,
            commands::set_window_theme,
            // GitMap Fleet Deployment commands
            commands::check_gitmap_available,
            commands::check_gitmap_installed,
            commands::deploy_accounts_to_fleet,
            // Toolchain Installer commands
            commands::toolchain_list_items,
            commands::toolchain_check,
            commands::toolchain_install,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                // Prevent app from exiting when window is destroyed in lightweight mode
                tauri::RunEvent::ExitRequested { api, .. } => {
                    let tray_enabled = app_handle
                        .try_state::<AppRuntimeFlags>()
                        .map(|flags| flags.tray_enabled)
                        .unwrap_or(true);

                    if tray_enabled {
                        api.prevent_exit();
                    }
                }
                // Handle app exit - cleanup background tasks and release ports
                tauri::RunEvent::Exit => {
                    tracing::info!(
                        "Application exiting, cleaning up background tasks and releasing ports..."
                    );
                    if let Some(state) =
                        app_handle.try_state::<crate::commands::proxy::ProxyServiceState>()
                    {
                        let cf_state = app_handle
                            .try_state::<crate::commands::cloudflared::CloudflaredState>();
                        tauri::async_runtime::block_on(async {
                            // 1. Stop cloudflared tunnel
                            if let Some(cf) = cf_state {
                                // Justification: best-effort guarded wait; a timeout or inner failure is logged
                                crate::error::record_ignored(
                                    tokio::time::timeout(
                                        std::time::Duration::from_millis(500),
                                        cf.stop(),
                                    )
                                    .await,
                                    "timeout wait",
                                );
                            }

                            // 2. Stop Admin Server (release TCP listener and socket)
                            if let Ok(mut lock) = tokio::time::timeout(
                                std::time::Duration::from_millis(1000),
                                state.admin_server.write(),
                            )
                            .await
                            {
                                if let Some(admin) = lock.take() {
                                    admin.stop().await;
                                }
                            }

                            // 3. Stop proxy instances and background tasks
                            if let Ok(mut lock) = tokio::time::timeout(
                                std::time::Duration::from_millis(1000),
                                state.instance.write(),
                            )
                            .await
                            {
                                if let Some(instance) = lock.take() {
                                    // Justification: best-effort guarded wait; a timeout or inner failure is logged
                                    crate::error::record_ignored(
                                        tokio::time::timeout(
                                            std::time::Duration::from_millis(500),
                                            instance.token_manager.graceful_shutdown(
                                                std::time::Duration::from_millis(400),
                                            ),
                                        )
                                        .await,
                                        "timeout wait",
                                    );
                                    instance.axum_server.set_running(false).await;
                                    instance.axum_server.stop();
                                }
                            }
                        });
                    }
                }
                // Handle macOS dock icon click to reopen window
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { .. } => {
                    // Justification: best-effort call; failure logged without changing control flow
                    crate::error::record_ignored(
                        modules::lightweight::exit_lightweight_mode(app_handle),
                        "exit_lightweight_mode",
                    );
                    if let Some(window) = app_handle.get_webview_window("main") {
                        // Justification: best-effort window operation; the UI continues without it
                        crate::error::record_ignored(window.unminimize(), "window.unminimize");
                        // Justification: best-effort window operation; the UI continues without it
                        crate::error::record_ignored(window.show(), "window.show");
                        // Justification: best-effort window operation; the UI continues without it
                        crate::error::record_ignored(window.set_focus(), "window.set_focus");
                        app_handle
                            .set_activation_policy(tauri::ActivationPolicy::Regular)
                            .unwrap_or(());
                    }
                }
                _ => {}
            }
        });
}
