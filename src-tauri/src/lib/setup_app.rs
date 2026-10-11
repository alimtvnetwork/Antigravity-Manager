use super::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Run in headless mode (no GUI).
/// Extracted from `run()` to keep file sizes manageable.
/// Tauri setup handler: initialize app state, windows, trays, and background services.
/// Extracted from `run()` to keep file sizes manageable.
pub(crate) fn setup_app(
    app: &mut tauri::App,
    tray_enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setup starting...");

    // Initialize log bridge with app handle for debug console
    modules::log_bridge::init_log_bridge(app.handle().clone());

    // Linux: Workaround for transparent window crash/freeze
    // The transparent window feature is unstable on Linux with WebKitGTK
    // We disable the visual alpha channel to prevent softbuffer-related crashes
    #[cfg(target_os = "linux")]
    {
        use tauri::Manager;
        if is_wayland_session() {
            info!("Linux Wayland session detected; skipping transparent window workaround");
        } else if let Some(window) = app.get_webview_window("main") {
            // Access GTK window and disable transparency at the GTK level
            if let Ok(gtk_window) = window.gtk_window() {
                use gtk::prelude::WidgetExt;
                // Remove the visual's alpha channel to disable transparency
                if let Some(screen) = gtk_window.screen() {
                    // Use non-composited visual if available
                    if let Some(visual) = screen.system_visual() {
                        gtk_window.set_visual(Some(&visual));
                    }
                    info!("Linux: Applied transparent window workaround");
                }
            }
        }
    }

    let runtime_flags = app.state::<AppRuntimeFlags>();
    if runtime_flags.tray_enabled {
        modules::tray::create_tray(app.handle())?;
        info!("Tray created");
    } else {
        info!("Tray disabled for this session");
    }

    // Explicitly set window icon and version title for main window on Windows/Linux and heal restored offscreen coordinates
    if let Some(window) = app.get_webview_window("main") {
        let window_title = format!("Antigravity Manager Tools v{}", env!("CARGO_PKG_VERSION"));
        // Justification: best-effort window operation; the UI continues without it
        crate::error::record_ignored(window.set_title(&window_title), "window.set_title");
        #[cfg(target_os = "windows")]
        {
            if let Ok(hwnd) = window.hwnd() {
                use std::os::windows::ffi::OsStrExt;
                let wide: Vec<u16> = std::ffi::OsStr::new(&window_title)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                unsafe {
                    extern "system" {
                        fn SetWindowTextW(hwnd: *mut std::ffi::c_void, lpString: *const u16)
                            -> i32;
                    }
                    SetWindowTextW(hwnd.0, wide.as_ptr());
                }
            }
        }

        let icon_bytes: &[u8] = include_bytes!("../../icons/icon.png");
        if let Ok(img) = image::load_from_memory(icon_bytes) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);
            // Justification: best-effort window operation; the UI continues without it
            crate::error::record_ignored(window.set_icon(icon), "window.set_icon");
        }
        let is_minimized_arg = std::env::args().any(|arg| arg == "--minimized");
        if !is_minimized_arg {
            restore_and_focus_window(&window);
        }
    }

    // Windows: asynchronously heal desktop and start menu shortcut icons and refresh shell
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(|| {
            crate::utils::win_shortcut::heal_shortcuts_native();
        });
    }

    // macOS: asynchronously heal LaunchServices registrations, purge stale trash associations, and run garbage collection
    #[cfg(target_os = "macos")]
    {
        std::thread::spawn(|| {
            // Justification: best-effort process spawn; failure logged
            crate::error::record_ignored(std::process::Command::new("bash")
                .arg("-c")
                .arg(r#"
                    rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true
                    osascript -e '
                    tell application "Finder"
                        try
                            set destFolder to (POSIX file "/private/tmp") as alias
                            repeat with anItem in (every item of trash)
                                try
                                    set n to name of anItem as text
                                    if n contains "Antigravity" or n contains "agm" then
                                        move anItem to destFolder with replacing
                                    end if
                                end try
                            end repeat
                        end try
                    end tell' 2>/dev/null || true
                    rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true

                    lsregister=$(find /System/Library/Frameworks/CoreServices.framework -name "lsregister" -type f 2>/dev/null | head -n 1)
                    if [ -n "$lsregister" ]; then
                        find "$HOME/.Trash" -maxdepth 1 \( -iname "*antigravity*" -o -iname "*agm*" \) 2>/dev/null | while read -r ta; do
                            if [ -n "$ta" ]; then
                                chflags -R nouchg,noschg "$ta" 2>/dev/null || true
                                "$lsregister" -u "$ta" 2>/dev/null || true
                                rm -rf "$ta" 2>/dev/null || true
                            fi
                        done
                        "$lsregister" -gc -R -v -apps u,s,l 2>/dev/null || "$lsregister" -gc 2>/dev/null || true
                    fi
                "#)
                .status(), "spawn bash");
        });
    }

    // Discover and persist initial IDE information on first run in background
    std::thread::spawn(|| {
        crate::modules::process::discover_and_persist_initial_ide_info();
    });

    // Warm up in-memory smart process cache and cross-check running Antigravity IDE instances
    std::thread::spawn(|| {
        crate::modules::instance::warm_up_smart_process_cache();
    });

    // Immediately start management server (8045) for Web access
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        // Load config
        if let Ok(config) = modules::config::load_app_config() {
            let state = handle.state::<commands::proxy::ProxyServiceState>();
            let cf_state = handle.state::<commands::cloudflared::CloudflaredState>();
            let integration = crate::modules::integration::SystemManager::Desktop(handle.clone());

            // 1. Ensure admin dashboard is running
            if let Err(e) = commands::proxy::ensure_admin_server(
                config.proxy.clone(),
                &state,
                integration.clone(),
                Arc::new(cf_state.inner().clone()),
            )
            .await
            {
                error!("Failed to start admin server: {}", e);
            } else {
                info!(
                    "Admin server (port {}) started successfully",
                    config.proxy.port
                );
            }

            // 2. Automatically start proxy routing if configured
            if config.proxy.auto_start {
                if let Err(e) = commands::proxy::internal_start_proxy_service(
                    config.proxy,
                    &state,
                    integration,
                    Arc::new(cf_state.inner().clone()),
                )
                .await
                {
                    error!("Failed to auto-start proxy service: {}", e);
                } else {
                    info!("Proxy service auto-started successfully");
                }
            }
        } else {
            // Config load failures must be logged with high visibility
            error!(
                "Failed to load app config at startup; admin server and proxy service were NOT started. \
                 Fix or reset the config file and restart the app."
            );
        }
    });

    // Start smart scheduler for 7-day weekly reset warmup
    let scheduler_state = app.handle().state::<commands::proxy::ProxyServiceState>();
    modules::scheduler::start_scheduler(
        Some(app.handle().clone()),
        scheduler_state.inner().clone(),
    );
    info!("Smart scheduler (7-Day Weekly Reset Warmup) initialized.");

    // Start prompt queue scheduler daemon (10-minute ticker)
    modules::scheduler::start_prompt_queue_scheduler();
    info!("Prompt queue scheduler daemon initialized.");

    // Start instance PID and quota scheduler daemon (10-minute ticker)
    modules::scheduler::start_instance_pid_quota_scheduler();
    info!("Instance PID and quota scheduler daemon initialized.");

    // Start auto profile switcher daemon
    modules::auto_switcher::start_auto_switcher();
    info!("Auto profile switcher daemon initialized.");

    // Start periodic conversation cleanup daemon (1-hour ticker)
    modules::agy_cleaner::start_cleanup_daemon();
    info!("Conversation cleanup daemon initialized.");

    // Start Telegram inbound remote command daemon
    modules::telegram_inbound::start_telegram_daemon();
    info!("Telegram inbound remote command daemon initialized.");

    // Check if system was updated and notify via configured channels (Email, Telegram)
    modules::notification_hub::check_and_notify_system_updated();
    info!("System update notification watcher initialized.");

    // [PHASE 1] Integrated into main Axum port (8045), port 19527 no longer started separately
    info!("Management API integrated into main proxy server (port 8045)");

    // Non-blocking background startup backfill for accounts with missing subscription tier
    tauri::async_runtime::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        let should_backfill = crate::modules::config::load_app_config()
            .map(|c| c.auto_refresh_missing_tiers)
            .unwrap_or(true);
        if should_backfill {
            crate::modules::logger::log_info(
                "Starting background startup backfill for accounts with missing subscription tier...",
            );
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::account::refresh_missing_tiers(Some(2)).await,
                "refresh_missing_tiers",
            );
        }
    });
    info!("Account subscription tier startup backfill task spawned.");

    Ok(())
}

/// Handle window events (close requested -> hide to tray).
/// Extracted from `run()` to keep file sizes manageable.
pub(crate) fn handle_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    match event {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            let tray_enabled = window
                .app_handle()
                .try_state::<AppRuntimeFlags>()
                .map(|flags| flags.tray_enabled)
                .unwrap_or(true);

            if tray_enabled {
                api.prevent_close();

                let is_lightweight = modules::load_app_config()
                    .map(|c| c.lightweight_mode)
                    .unwrap_or(false);

                if is_lightweight {
                    // Justification: best-effort call; failure logged without changing control flow
                    crate::error::record_ignored(
                        modules::lightweight::enter_lightweight_mode(window.app_handle()),
                        "enter_lightweight_mode",
                    );
                } else {
                    // Justification: best-effort window operation; the UI continues without it
                    crate::error::record_ignored(window.hide(), "window.hide");
                    #[cfg(target_os = "macos")]
                    {
                        use tauri::Manager;
                        window
                            .app_handle()
                            .set_activation_policy(tauri::ActivationPolicy::Accessory)
                            .unwrap_or(());
                    }
                }
            }
        }
        tauri::WindowEvent::Focused(focused) => {
            if *focused {
                // Justification: best-effort frontend event; a dropped event only skips a UI refresh
                crate::error::record_ignored(
                    window.emit("window-restored", ()),
                    "emit window-restored",
                );
            }
        }
        _ => {}
    }
}

pub(crate) fn run_headless() {
    info!("Starting in HEADLESS mode...");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        // Initialize states manually
        // [FIX] Initialize log bridge for headless mode
        // Pass a dummy app handle or None since we don't have a Tauri app handle in headless mode
        // Actually log_bridge relies on AppHandle to emit events.
        // In headless mode, we don't emit events, but we still need the buffer.
        // We need to modify log_bridge to handle missing AppHandle gracefully, which it already does (Option).
        // But init_log_bridge requires AppHandle.
        // We'll skip passing AppHandle for now and just leverage the global buffer capability.
        // Since init_log_bridge takes AppHandle, we might need a separate init for headless or just not call init and rely on lazy init of buffer?
        // Checking log_bridge code again...
        // "static LOG_BUFFER: OnceLock<...> = OnceLock::new();" -> lazy init.
        // So we just need to ensure the tracing layer is added.
        // And `logger::init_logger()` adds the layer?
        // Let's check `modules::logger`.

        let proxy_state = commands::proxy::ProxyServiceState::new();
        let cf_state = Arc::new(commands::cloudflared::CloudflaredState::new());

        // Discover and persist initial IDE information on first run in background
        std::thread::spawn(|| {
            crate::modules::process::discover_and_persist_initial_ide_info();
        });

        // Load config
        match modules::config::load_app_config() {
            Ok(mut config) => {
                let mut modified = false;
                // Headless/docker defaults to LAN access (binds 0.0.0.0)
                // If ABV_BIND_LOCAL_ONLY is set, binds only 127.0.0.1
                let bind_local_only = std::env::var("ABV_BIND_LOCAL_ONLY")
                    .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
                    .unwrap_or(false);
                if bind_local_only {
                    config.proxy.allow_lan_access = false;
                    modified = true;
                } else {
                    config.proxy.allow_lan_access = true;
                }

                // [FIX] Force auth mode to AllExceptHealth in headless mode if it's Off or Auto
                // This ensures Web UI login validation works properly
                if matches!(
                    config.proxy.auth_mode,
                    crate::proxy::ProxyAuthMode::Off | crate::proxy::ProxyAuthMode::Auto
                ) {
                    info!(
                        "Headless mode: Forcing auth_mode to AllExceptHealth for Web UI security"
                    );
                    config.proxy.auth_mode = crate::proxy::ProxyAuthMode::AllExceptHealth;
                    modified = true;
                }

                // [NEW] Support API Key injection via environment variable
                // Priority: ABV_API_KEY > API_KEY > configuration file
                let env_key = std::env::var("ABV_API_KEY")
                    .or_else(|_| std::env::var("API_KEY"))
                    .ok();

                if let Some(key) = env_key {
                    if !key.trim().is_empty() {
                        info!("Using API Key from environment variable");
                        config.proxy.api_key = key;
                        modified = true;
                    }
                }

                // [NEW] Support Web UI password injection via environment variable
                // Priority: ABV_WEB_PASSWORD > WEB_PASSWORD > configuration file
                let env_web_password = std::env::var("ABV_WEB_PASSWORD")
                    .or_else(|_| std::env::var("WEB_PASSWORD"))
                    .ok();

                if let Some(pwd) = env_web_password {
                    if !pwd.trim().is_empty() {
                        info!("Using Web UI Password from environment variable");
                        config.proxy.admin_password = Some(pwd);
                        modified = true;
                    }
                }

                // [NEW] Support auth mode injection via environment variable
                // Priority: ABV_AUTH_MODE > AUTH_MODE > configuration file
                let env_auth_mode = std::env::var("ABV_AUTH_MODE")
                    .or_else(|_| std::env::var("AUTH_MODE"))
                    .ok();

                if let Some(mode_str) = env_auth_mode {
                    let mode = match mode_str.to_lowercase().as_str() {
                        "off" => Some(crate::proxy::ProxyAuthMode::Off),
                        "strict" => Some(crate::proxy::ProxyAuthMode::Strict),
                        "all_except_health" => Some(crate::proxy::ProxyAuthMode::AllExceptHealth),
                        "auto" => Some(crate::proxy::ProxyAuthMode::Auto),
                        _ => {
                            warn!("Invalid AUTH_MODE: {}, ignoring", mode_str);
                            None
                        }
                    };
                    if let Some(m) = mode {
                        info!("Using Auth Mode from environment variable: {:?}", m);
                        config.proxy.auth_mode = m;
                        modified = true;
                    }
                }

                info!("--------------------------------------------------");
                info!("🚀 Headless mode proxy service starting...");
                info!("📍 Port: {}", config.proxy.port);
                info!(
                    "🔑 Current API Key: {}",
                    credential_state(&config.proxy.api_key)
                );
                if let Some(ref pwd) = config.proxy.admin_password {
                    info!("🔐 Web UI Password: {}", credential_state(pwd));
                } else {
                    info!("🔐 Web UI Password: (Same as API Key)");
                }
                info!("💡 Tips: You can use these keys to login to Web UI and access AI APIs.");
                info!("💡 Search docker logs or grep gui_config.json to find them.");
                info!("--------------------------------------------------");

                // [FIX #1460] Persist environment overrides to ensure they are visible in Web UI/load_config
                if modified {
                    if let Err(e) = modules::config::save_app_config(&config) {
                        error!("Failed to persist environment overrides: {}", e);
                    } else {
                        info!("Environment overrides persisted to gui_config.json");
                    }
                }

                // Start proxy service
                if let Err(e) = commands::proxy::internal_start_proxy_service(
                    config.proxy,
                    &proxy_state,
                    crate::modules::integration::SystemManager::Headless,
                    cf_state.clone(),
                )
                .await
                {
                    error!("Failed to start proxy service in headless mode: {}", e);
                    std::process::exit(1);
                }

                info!("Headless proxy service is running.");

                // Warm up in-memory smart process cache and cross-check running Antigravity IDE instances
                modules::instance::warm_up_smart_process_cache();
                info!("Startup smart process cache warmed up in headless mode.");

                // Start smart scheduler for 7-day weekly reset warmup
                modules::scheduler::start_scheduler(None, proxy_state.clone());
                info!("Smart scheduler (7-Day Weekly Reset Warmup) started in headless mode.");

                // Start prompt queue scheduler daemon (10-minute ticker)
                modules::scheduler::start_prompt_queue_scheduler();
                info!("Prompt queue scheduler daemon started in headless mode.");

                // Start instance PID and quota scheduler daemon (10-minute ticker)
                modules::scheduler::start_instance_pid_quota_scheduler();
                info!("Instance PID and quota scheduler daemon started in headless mode.");
            }
            Err(e) => {
                error!("Failed to load config for headless mode: {}", e);
                std::process::exit(1);
            }
        }

        // Wait for Ctrl-C
        tokio::signal::ctrl_c().await.ok();
        info!("Headless mode shutting down");
    });
    return;
}

pub(crate) fn init_databases() {
    // Initialize token stats database
    if let Err(e) = modules::token_stats::init_db() {
        error!("Failed to initialize token stats database: {}", e);
    }

    // Initialize security database
    if let Err(e) = modules::security_db::init_db() {
        error!("Failed to initialize security database: {}", e);
    }

    // Initialize user token database
    if let Err(e) = modules::user_token_db::init_db() {
        error!("Failed to initialize user token database: {}", e);
    }
}
