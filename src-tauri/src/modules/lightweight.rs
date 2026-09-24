use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

/// Attempt to acquire or rebuild the main window.
/// If the main window has been destroyed in lightweight mode, dynamically rebuild it from tauri.conf.json configuration,
/// restoring window position, dimensions, and icons.
pub fn ensure_main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("main") {
        return Ok(window);
    }

    tracing::info!("[Lightweight] Main window is currently inactive, rebuilding WebviewWindow('main') from config...");

    // 1. Locate window configuration labeled "main"
    let window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .cloned()
        .ok_or_else(|| {
            "Window configuration with label 'main' not found in tauri.conf.json".to_string()
        })?;

    // 2. Dynamically construct window
    let window = WebviewWindowBuilder::from_config(app, &window_config)
        .map_err(|e| format!("Failed to build main window: {}", e))?
        .build()
        .map_err(|e| format!("Failed to launch main window: {}", e))?;

    // 3. Explicitly set application icon on non-macOS platforms (ensure Win32 taskbar icon restoration)
    #[cfg(not(target_os = "macos"))]
    {
        let icon_bytes: &[u8] = include_bytes!("../../icons/icon.png");
        if let Ok(img) = image::load_from_memory(icon_bytes) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let _ = window.set_icon(tauri::image::Image::new_owned(
                rgba.into_raw(),
                width,
                height,
            ));
        }
    }

    // 4. Linux transparent window compatibility handling
    #[cfg(target_os = "linux")]
    {
        if !crate::is_wayland_session() {
            if let Ok(gtk_window) = window.gtk_window() {
                use gtk::prelude::WidgetExt;
                if let Some(screen) = gtk_window.screen() {
                    if let Some(visual) = screen.system_visual() {
                        gtk_window.set_visual(Some(&visual));
                    }
                }
            }
        }
    }

    // 5. Restore window state (remembered positions and dimensions)
    let _ = window.restore_state(StateFlags::all().difference(StateFlags::VISIBLE));

    tracing::info!("[Lightweight] Main window rebuilt successfully");
    Ok(window)
}

/// Exit lightweight mode: ensure main window exists, then show, unminimize, and focus it.
pub fn exit_lightweight_mode(app: &AppHandle) -> Result<WebviewWindow, String> {
    let window = ensure_main_window(app)?;

    crate::restore_and_focus_window(&window);

    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Regular)
            .unwrap_or(());
    }

    Ok(window)
}

/// Enter lightweight mode: save window state and destroy WebView to release 100MB+ resident memory.
pub fn enter_lightweight_mode(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        tracing::info!("[Lightweight] Entering lightweight mode, saving window state and destroying WebView...");

        // 1. Save window state
        let _ = app.save_window_state(StateFlags::all().difference(StateFlags::VISIBLE));

        // 2. Destroy window and webview renderer
        let _ = window.destroy();

        // 3. Switch macOS activation policy to Accessory to remove from Dock
        #[cfg(target_os = "macos")]
        {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory)
                .unwrap_or(());
        }

        tracing::info!("[Lightweight] WebView released, resident memory minimized");
    }

    Ok(())
}
