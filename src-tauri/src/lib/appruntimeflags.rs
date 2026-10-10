use super::*;

#[derive(Clone, Copy)]
pub(crate) struct AppRuntimeFlags {
    tray_enabled: bool,
}

pub(crate) fn env_flag_enabled(name: &str) -> bool {
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
pub(crate) fn is_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}

pub(crate) fn should_enable_tray() -> bool {
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

pub(crate) fn credential_state(value: &str) -> &'static str {
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
        fn InvalidateRect(hwnd: *mut c_void, lp_rect: *const c_void, b_erase: i32) -> i32;
        fn RedrawWindow(
            hwnd: *mut c_void,
            lprc_update: *const c_void,
            hrgn_update: *mut c_void,
            flags: u32,
        ) -> i32;
        fn SetWindowPos(
            hwnd: *mut c_void,
            hwnd_insert_after: *mut c_void,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
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

            // Force DWM frame recalculation and repaint to eliminate blank gray preview / occluded window freeze
            const SWP_NOSIZE: u32 = 0x0001;
            const SWP_NOMOVE: u32 = 0x0002;
            const SWP_NOZORDER: u32 = 0x0004;
            const SWP_FRAMECHANGED: u32 = 0x0020;
            SetWindowPos(
                hwnd_ptr,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );

            const RDW_INVALIDATE: u32 = 0x0001;
            const RDW_INTERNALPAINT: u32 = 0x0002;
            const RDW_ALLCHILDREN: u32 = 0x0080;
            const RDW_UPDATENOW: u32 = 0x0100;
            InvalidateRect(hwnd_ptr, std::ptr::null(), 1);
            RedrawWindow(
                hwnd_ptr,
                std::ptr::null(),
                std::ptr::null_mut(),
                RDW_INVALIDATE | RDW_INTERNALPAINT | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );

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
    // Justification: best-effort window operation; the UI continues without it
    crate::error::record_ignored(window.show(), "window.show");
    // Justification: best-effort window operation; the UI continues without it
    crate::error::record_ignored(window.unminimize(), "window.unminimize");
    if let Ok(is_min) = window.is_minimized() {
        if is_min {
            // Justification: best-effort window operation; the UI continues without it
            crate::error::record_ignored(window.unminimize(), "window.unminimize");
        }
    }
    if let Ok(pos) = window.outer_position() {
        let is_offscreen_x = pos.x < -1000;
        let is_offscreen_y = pos.y < -1000;
        if is_offscreen_x || is_offscreen_y {
            // Justification: best-effort window operation; the UI continues without it
            crate::error::record_ignored(window.center(), "window.center");
        }
    }
    if let Ok(size) = window.outer_size() {
        let is_too_narrow = size.width < 500;
        let is_too_short = size.height < 400;
        if is_too_narrow || is_too_short {
            // Justification: best-effort window operation; the UI continues without it
            crate::error::record_ignored(
                window.set_size(tauri::LogicalSize::new(1200, 800)),
                "window.set_size",
            );
            // Justification: best-effort window operation; the UI continues without it
            crate::error::record_ignored(window.center(), "window.center");
        }
    }
    // Justification: best-effort window operation; the UI continues without it
    crate::error::record_ignored(window.set_focus(), "window.set_focus");
    #[cfg(target_os = "windows")]
    {
        force_restore_and_focus_win32(window);
        // Justification: best-effort window operation; the UI continues without it
        crate::error::record_ignored(window.set_always_on_top(true), "window.set_always_on_top");
        // Justification: best-effort window operation; the UI continues without it
        crate::error::record_ignored(window.set_always_on_top(false), "window.set_always_on_top");
        // Justification: best-effort window operation; the UI continues without it
        crate::error::record_ignored(window.set_focus(), "window.set_focus");
    }
    #[cfg(target_os = "macos")]
    {
        use tauri::Manager;
        window
            .app_handle()
            .set_activation_policy(tauri::ActivationPolicy::Regular)
            .unwrap_or(());
    }
    // Justification: best-effort window operation; the UI continues without it
    crate::error::record_ignored(window.eval(
        "window.dispatchEvent(new Event('resize')); if (document.body) { document.body.style.transform = 'translateZ(0)'; }",
    ), "window.eval");
    // Justification: best-effort frontend event; a dropped event only skips a UI refresh
    crate::error::record_ignored(window.emit("window-restored", ()), "emit window-restored");
}

#[cfg(target_os = "linux")]
pub(crate) fn nvidia_proprietary_loaded() -> bool {
    std::path::Path::new("/dev/nvidia0").exists()
        || std::path::Path::new("/proc/driver/nvidia/version").exists()
}

#[cfg(target_os = "linux")]
pub(crate) fn configure_linux_graphics() {
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
pub(crate) fn increase_nofile_limit() {
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
pub(crate) mod windows_api {
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
pub(crate) fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
