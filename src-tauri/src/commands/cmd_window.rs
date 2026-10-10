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

/// 设置窗口主题（用于同步 Windows 标题栏按钮颜色）
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

/// 最小化窗口
#[tauri::command]
pub async fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// 最大化窗口
#[tauri::command]
pub async fn maximize_window(window: tauri::Window) -> Result<(), String> {
    window.maximize().map_err(|e| e.to_string())
}

/// 切换最大化/还原窗口
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

/// 关闭窗口
#[tauri::command]
pub async fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

/// 查询窗口是否处于最大化状态
#[tauri::command]
pub async fn is_window_maximized(window: tauri::Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}
