// Autostart command
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub async fn toggle_auto_launch(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
    let manager = app.autolaunch();

    if enable {
        manager
            .enable()
            .map_err(|e| format!("Failed to enable autostart: {}", e))?;
        crate::modules::logger::log_info("Automatic startup enabled");
    } else {
        match manager.disable() {
            Ok(_) => {
                crate::modules::logger::log_info("Automatic startup disabled");
            }
            Err(e) => {
                let err_msg = e.to_string();
                // On Windows, if registry entry does not exist, disable() returns "The system cannot find the file specified" (os error 2)
                // This situation should be considered a success because the goal (disabled) has been achieved
                if err_msg.contains("os error 2")
                    || err_msg.contains("找不到指定的文件")
                    || err_msg.contains("The system cannot find the file specified")
                {
                    crate::modules::logger::log_info("The auto-start item no longer exists and is deemed to be successfully disabled.");
                } else {
                    return Err(format!("Failed to disable autostart: {}", e));
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn is_auto_launch_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let manager = app.autolaunch();
    manager.is_enabled().map_err(|e| e.to_string())
}
