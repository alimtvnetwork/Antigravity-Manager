use super::*;

#[tauri::command]
pub async fn show_main_window(window: tauri::WebviewWindow) -> Result<(), String> {
    let icon_bytes: &[u8] = include_bytes!("../../icons/icon.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);
        // Justification: window icon is cosmetic; the window is shown regardless
        crate::error::record_ignored(window.set_icon(icon), "set main window icon");
    }
    crate::restore_and_focus_window(&window);
    Ok(())
}

/// Set window theme (used to synchronize Windows title bar button colors)
#[tauri::command]
pub async fn set_window_theme(window: tauri::Window, theme: String) -> Result<(), String> {
    use tauri::Theme;

    let tauri_theme = match theme.as_str() {
        "dark" => Some(Theme::Dark),
        "light" => Some(Theme::Light),
        _ => None, // system default
    };

    window.set_theme(tauri_theme).map_err(|e| e.to_string())
}

/// Minimize window
#[tauri::command]
pub async fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// maximize window
#[tauri::command]
pub async fn maximize_window(window: tauri::Window) -> Result<(), String> {
    window.maximize().map_err(|e| e.to_string())
}

/// Toggle maximize/restore window
#[tauri::command]
pub async fn toggle_maximize_window(window: tauri::Window) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

/// Close the window and exit the app completely
#[tauri::command]
pub fn close_window(window: tauri::Window) -> Result<(), String> {
    crate::modules::process::exit_app_fully(&window.app_handle());
}

/// Query whether the window is maximized
#[tauri::command]
pub async fn is_window_maximized(window: tauri::Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}
