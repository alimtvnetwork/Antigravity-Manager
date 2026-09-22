mod commands;
pub mod constants;
pub mod error;
#[cfg(target_os = "linux")]
mod linux_graphics;
mod models;
mod modules;
mod proxy; // Proxy service module
mod utils;

use modules::logger;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tracing::{error, info, warn};

#[derive(Clone, Copy)]
struct AppRuntimeFlags {
    tray_enabled: bool,
}

fn env_flag_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn is_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}

fn should_enable_tray() -> bool {
    if env_flag_enabled("ANTIGRAVITY_DISABLE_TRAY") {
        info!("Tray disabled by ANTIGRAVITY_DISABLE_TRAY");
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        if is_wayland_session() && !env_flag_enabled("ANTIGRAVITY_FORCE_TRAY") {
            // Smart adaptive detection: check if valid AppIndicator / StatusNotifier dynamic library exists
            let has_appindicator = [
                "/usr/lib/x86_64-linux-gnu/libayatana-appindicator3.so.1",
                "/usr/lib/x86_64-linux-gnu/libappindicator3.so.1",
                "/usr/lib64/libayatana-appindicator3.so.1",
                "/usr/lib64/libappindicator3.so.1",
                "/usr/lib/libayatana-appindicator3.so.1",
                "/usr/lib/libappindicator3.so.1",
            ]
            .iter()
            .any(|path| std::path::Path::new(path).exists());

            if has_appindicator {
                info!("Linux Wayland session detected with valid AppIndicator libraries. Enabling tray automatically.");
                return true;
            }

            warn!(
                "Linux Wayland session detected without AppIndicator libraries; disabling tray by default to avoid GTK crashes. Install libayatana-appindicator3 or set ANTIGRAVITY_FORCE_TRAY=1 to force-enable."
            );
            return false;
        }
    }

    true
}

fn credential_state(value: &str) -> &'static str {
    if value.trim().is_empty() {
        "not set"
    } else {
        "set"
    }
}

#[cfg(target_os = "windows")]
pub fn force_restore_and_focus_win32(window: &tauri::WebviewWindow) {
    use std::ffi::c_void;

    #[link(name = "user32")]
    extern "system" {
        fn ShowWindow(hwnd: *mut c_void, n_cmd_show: i32) -> i32;
        fn SetForegroundWindow(hwnd: *mut c_void) -> i32;
        fn BringWindowToTop(hwnd: *mut c_void) -> i32;
        fn SwitchToThisWindow(hwnd: *mut c_void, alt_tab: i32);
        fn IsIconic(hwnd: *mut c_void) -> i32;
        fn OpenIcon(hwnd: *mut c_void) -> i32;
        fn GetCurrentThreadId() -> u32;
        fn GetWindowThreadProcessId(hwnd: *mut c_void, lpdw_process_id: *mut u32) -> u32;
        fn AttachThreadInput(id_attach: u32, id_attach_to: u32, f_attach: i32) -> i32;
        fn GetForegroundWindow() -> *mut c_void;
    }

    if let Ok(hwnd) = window.hwnd() {
        let hwnd_ptr = hwnd.0 as *mut c_void;
        unsafe {
            let is_min = IsIconic(hwnd_ptr) != 0;
            if is_min {
                OpenIcon(hwnd_ptr);
                ShowWindow(hwnd_ptr, 9); // SW_RESTORE
            } else {
                ShowWindow(hwnd_ptr, 5); // SW_SHOW
            }

            let fg_hwnd = GetForegroundWindow();
            let fg_thread = GetWindowThreadProcessId(fg_hwnd, std::ptr::null_mut());
            let current_thread = GetCurrentThreadId();

            let is_different_thread = fg_thread != current_thread;
            let has_fg_thread = fg_thread != 0;
            if is_different_thread {
                if has_fg_thread {
                    AttachThreadInput(current_thread, fg_thread, 1);
                    BringWindowToTop(hwnd_ptr);
                    SetForegroundWindow(hwnd_ptr);
                    AttachThreadInput(current_thread, fg_thread, 0);
                }
            } else {
                BringWindowToTop(hwnd_ptr);
                SetForegroundWindow(hwnd_ptr);
            }

            SwitchToThisWindow(hwnd_ptr, 1);
        }
    }
}

pub fn restore_and_focus_window(window: &tauri::WebviewWindow) {
    let _ = window.show();
    let _ = window.unminimize();
    if let Ok(is_min) = window.is_minimized() {
        if is_min {
            let _ = window.unminimize();
        }
    }
    if let Ok(pos) = window.outer_position() {
        let is_offscreen_x = pos.x < -1000;
        let is_offscreen_y = pos.y < -1000;
        if is_offscreen_x || is_offscreen_y {
            let _ = window.center();
        }
    }
    if let Ok(size) = window.outer_size() {
        let is_too_narrow = size.width < 500;
        let is_too_short = size.height < 400;
        if is_too_narrow || is_too_short {
            let _ = window.set_size(tauri::LogicalSize::new(1200, 800));
            let _ = window.center();
        }
    }
    let _ = window.set_focus();
    #[cfg(target_os = "windows")]
    {
        force_restore_and_focus_win32(window);
        let _ = window.set_always_on_top(true);
        let _ = window.set_always_on_top(false);
        let _ = window.set_focus();
    }
    #[cfg(target_os = "macos")]
    {
        use tauri::Manager;
        window
            .app_handle()
            .set_activation_policy(tauri::ActivationPolicy::Regular)
            .unwrap_or(());
    }
    let _ = window.emit("window-restored", ());
}

#[cfg(target_os = "linux")]
fn nvidia_proprietary_loaded() -> bool {
    std::path::Path::new("/dev/nvidia0").exists()
        || std::path::Path::new("/proc/driver/nvidia/version").exists()
}

#[cfg(target_os = "linux")]
fn configure_linux_graphics() {
    use linux_graphics::{
        desktop_is_wlroots_family, should_disable_webkit_dmabuf, should_force_x11_backend,
    };

    let is_wayland = is_wayland_session();
    let has_x11_display = std::env::var("DISPLAY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_else(|_| std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default());
    let force_wayland = env_flag_enabled("ANTIGRAVITY_FORCE_WAYLAND");
    let force_x11 = env_flag_enabled("ANTIGRAVITY_FORCE_X11");
    let gdk_already_set = std::env::var("GDK_BACKEND").is_ok();

    if should_force_x11_backend(
        gdk_already_set,
        force_x11,
        force_wayland,
        is_wayland,
        has_x11_display,
        &desktop,
    ) {
        // Force X11 backend under GNOME/KDE Wayland to avoid a GTK shm crash.
        std::env::set_var("GDK_BACKEND", "x11");
        warn!(
            "Forcing GDK_BACKEND=x11 for stability on Wayland. Set ANTIGRAVITY_FORCE_WAYLAND=1 to keep Wayland backend."
        );
    } else if is_wayland && !gdk_already_set && desktop_is_wlroots_family(&desktop) {
        info!(
            "Keeping native Wayland GDK backend on {} (Xwayland DISPLAY is not a reason to force X11).",
            desktop
        );
    }

    let webkit_already_set = std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_ok();
    if should_disable_webkit_dmabuf(
        webkit_already_set,
        is_wayland,
        nvidia_proprietary_loaded(),
        &desktop,
    ) {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        info!(
            "WEBKIT_DISABLE_DMABUF_RENDERER=1 (WebKit DMA-BUF workaround on this Wayland setup). Set it yourself to override."
        );
    }
}

/// Increase file descriptor limit for macOS to prevent "Too many open files" errors
#[cfg(target_os = "macos")]
fn increase_nofile_limit() {
    unsafe {
        let mut rl = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rl) == 0 {
            info!(
                "Current open file limit: soft={}, hard={}",
                rl.rlim_cur, rl.rlim_max
            );

            // Attempt to increase to 4096 or maximum hard limit
            let target = 4096.min(rl.rlim_max);
            if rl.rlim_cur < target {
                rl.rlim_cur = target;
                if libc::setrlimit(libc::RLIMIT_NOFILE, &rl) == 0 {
                    info!("Successfully increased hard file limit to {}", target);
                } else {
                    warn!("Failed to increase file descriptor limit");
                }
            }
        }
    }
}

/// Windows FFI calls to disable Efficiency Mode (EcoQoS / Power Throttling)
/// to prevent background freezes when minimized/hidden.
#[cfg(target_os = "windows")]
mod windows_api {
    type Bool = i32;
    type Handle = *mut std::ffi::c_void;

    #[repr(C)]
    struct ProcessPowerThrottlingState {
        version: u32,
        control_mask: u32,
        state_mask: u32,
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> Handle;
        fn SetProcessInformation(
            h_process: Handle,
            process_information_class: u32,
            process_information: *mut std::ffi::c_void,
            process_information_size: u32,
        ) -> Bool;
    }

    pub fn disable_efficiency_mode() {
        unsafe {
            let mut state = ProcessPowerThrottlingState {
                version: 1,        // PROCESS_POWER_THROTTLING_STATE::VERSION
                control_mask: 0x1, // PROCESS_POWER_THROTTLING_CURRENT_EXECUTION_SPEED
                state_mask: 0,
            };
            let process_handle = GetCurrentProcess();
            // ProcessPowerThrottling = 4
            let res = SetProcessInformation(
                process_handle,
                4,
                &mut state as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<ProcessPowerThrottlingState>() as u32,
            );
            if res == 0 {
                let err = std::io::Error::last_os_error();
                tracing::warn!(
                    "Failed to disable Windows Power Throttling / EcoQoS: {}",
                    err
                );
            } else {
                tracing::info!(
                    "Successfully disabled Windows Power Throttling / EcoQoS for the process."
                );
            }
        }
    }
}

// Test command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Check for terminal CLI profile management arguments
    if modules::cli::handle_cli_arguments() {
        return;
    }

    // Disable Windows background throttling/EcoQoS
    #[cfg(target_os = "windows")]
    windows_api::disable_efficiency_mode();

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

    if is_headless {
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
                    if matches!(config.proxy.auth_mode, crate::proxy::ProxyAuthMode::Off | crate::proxy::ProxyAuthMode::Auto) {
                        info!("Headless mode: Forcing auth_mode to AllExceptHealth for Web UI security");
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
                    info!("🔑 Current API Key: {}", credential_state(&config.proxy.api_key));
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
                    ).await {
                        error!("Failed to start proxy service in headless mode: {}", e);
                        std::process::exit(1);
                    }

                    info!("Headless proxy service is running.");

                    // Start smart scheduler for 7-day weekly reset warmup
                    modules::scheduler::start_scheduler(None, proxy_state.clone());
                    info!("Smart scheduler (7-Day Weekly Reset Warmup) started in headless mode.");
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
            let _ = app.get_webview_window("main").map(|window| {
                restore_and_focus_window(&window);
            });
        }))
        .manage(commands::proxy::ProxyServiceState::new())
        .manage(commands::cloudflared::CloudflaredState::new())
        .manage(AppRuntimeFlags { tray_enabled })
        .setup(|app| {
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

            // Explicitly set window icon for main window on Windows/Linux and heal restored offscreen coordinates
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(pos) = window.outer_position() {
                    let is_offscreen_x = pos.x < -1000;
                    let is_offscreen_y = pos.y < -1000;
                    if is_offscreen_x || is_offscreen_y {
                        let _ = window.center();
                    }
                }
                let icon_bytes: &[u8] = include_bytes!("../icons/icon.png");
                if let Ok(img) = image::load_from_memory(icon_bytes) {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);
                    let _ = window.set_icon(icon);
                }
            }

            // Windows: asynchronously heal desktop and start menu shortcut icons and refresh shell
            #[cfg(target_os = "windows")]
            {
                std::thread::spawn(|| {
                    crate::utils::win_shortcut::heal_shortcuts_native();
                });
            }

            // Immediately start management server (8045) for Web access
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Load config
                if let Ok(config) = modules::config::load_app_config() {
                    let state = handle.state::<commands::proxy::ProxyServiceState>();
                    let cf_state = handle.state::<commands::cloudflared::CloudflaredState>();
                    let integration =
                        crate::modules::integration::SystemManager::Desktop(handle.clone());

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
            modules::scheduler::start_scheduler(Some(app.handle().clone()), scheduler_state.inner().clone());
            info!("Smart scheduler (7-Day Weekly Reset Warmup) initialized.");

            // Start auto profile switcher daemon
            modules::auto_switcher::start_auto_switcher();
            info!("Auto profile switcher daemon initialized.");

            // [PHASE 1] Integrated into main Axum port (8045), port 19527 no longer started separately
            info!("Management API integrated into main proxy server (port 8045)");

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let tray_enabled = window
                        .app_handle()
                        .try_state::<AppRuntimeFlags>()
                        .map(|flags| flags.tray_enabled)
                        .unwrap_or(true);

                    if tray_enabled {
                        let _ = window.hide();
                        #[cfg(target_os = "macos")]
                        {
                            use tauri::Manager;
                            window
                                .app_handle()
                                .set_activation_policy(tauri::ActivationPolicy::Accessory)
                                .unwrap_or(());
                        }
                        api.prevent_close();
                    }
                }
                tauri::WindowEvent::Focused(focused) => {
                    if *focused {
                        let _ = window.emit("window-restored", ());
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // Account management commands
            commands::list_accounts,
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
            commands::proxy::clear_proxy_logs,
            commands::proxy::clear_thinking_store,
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
            proxy::opencode_sync::get_canonical_families,
            proxy::opencode_sync::execute_opencode_sync,
            proxy::opencode_sync::execute_opencode_openai_sync,
            proxy::opencode_sync::execute_opencode_restore,
            proxy::opencode_sync::get_opencode_config_content,
            proxy::opencode_sync::execute_opencode_clear,
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
            commands::rename_instance,
            commands::delete_instance,
            commands::wipe_instance_session,
            commands::launch_instance,
            commands::clone_instance_executable,
            commands::set_instance_executable,
            commands::close_instance,
            commands::get_active_instance,
            commands::set_active_instance,
            commands::switch_account_to_instance,
            commands::export_instances_json,
            commands::import_instances_json,
            commands::get_auto_switcher_status,
            commands::update_auto_switcher_config,
            commands::trigger_manual_profile_rotation,
            commands::list_running_projects,
            commands::list_backed_up_prompts,
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
            // Telegram Inbound Watcher commands
            commands::get_telegram_config,
            commands::save_telegram_config,
            commands::test_telegram_bot,
            commands::get_telegram_status,
            commands::send_telegram_test_message,
        ])


        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                // Handle app exit - cleanup background tasks and release ports
                tauri::RunEvent::Exit => {
                    tracing::info!("Application exiting, cleaning up background tasks and releasing ports...");
                    if let Some(state) =
                        app_handle.try_state::<crate::commands::proxy::ProxyServiceState>()
                    {
                        let cf_state = app_handle.try_state::<crate::commands::cloudflared::CloudflaredState>();
                        tauri::async_runtime::block_on(async {
                            // 1. Stop cloudflared tunnel
                            if let Some(cf) = cf_state {
                                let _ = tokio::time::timeout(std::time::Duration::from_millis(500), cf.stop()).await;
                            }

                            // 2. Stop Admin Server (release TCP listener and socket)
                            if let Ok(mut lock) = tokio::time::timeout(std::time::Duration::from_millis(1000), state.admin_server.write()).await {
                                if let Some(admin) = lock.take() {
                                    admin.stop().await;
                                }
                            }

                            // 3. Stop proxy instances and background tasks
                            if let Ok(mut lock) = tokio::time::timeout(std::time::Duration::from_millis(1000), state.instance.write()).await {
                                if let Some(instance) = lock.take() {
                                    let _ = tokio::time::timeout(
                                        std::time::Duration::from_millis(500),
                                        instance.token_manager.graceful_shutdown(std::time::Duration::from_millis(400)),
                                    ).await;
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
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        app_handle
                            .set_activation_policy(tauri::ActivationPolicy::Regular)
                            .unwrap_or(());
                    }
                }
                _ => {}
            }
        });
}
