use super::*;

#[tauri::command]
pub async fn open_data_folder() -> Result<(), String> {
    let path = modules::account::get_data_dir()?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        use crate::utils::command::CommandExtWrapper;
        std::process::Command::new("explorer")
            .creation_flags_windows()
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    Ok(())
}

/// 获取数据目录绝对路径
#[tauri::command]
pub async fn get_data_dir_path() -> Result<String, String> {
    let path = modules::account::get_data_dir()?;
    Ok(modules::account::format_data_dir_path(&path))
}

/// Select and migrate data directory (pointer stored in home dir for restart discovery)
#[tauri::command]
pub async fn set_data_dir(
    path: String,
    proxy_state: tauri::State<'_, crate::commands::proxy::ProxyServiceState>,
    cf_state: tauri::State<'_, crate::commands::cloudflared::CloudflaredState>,
) -> Result<String, String> {
    {
        let instance = proxy_state.instance.read().await;
        if instance.is_some() {
            return Err(
                "Please stop the API proxy service before migrating the data directory".to_string(),
            );
        }
    }
    {
        let lock = cf_state.manager.read().await;
        if let Some(manager) = lock.as_ref() {
            let status = manager.get_status().await;
            if status.running {
                return Err(
                    "Please stop the Cloudflared tunnel before migrating the data directory"
                        .to_string(),
                );
            }
        }
    }

    let new_path = tokio::task::spawn_blocking(move || {
        modules::account::migrate_data_dir(PathBuf::from(path))
    })
    .await
    .map_err(|e| format!("Migration task failed: {}", e))??;

    {
        let mut lock = cf_state.manager.write().await;
        *lock = None;
    }

    Ok(modules::account::format_data_dir_path(&new_path))
}

/// Recursively copy directory contents
fn copy_dir_all_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

/// Migrate full data directory to new path
#[tauri::command]
pub async fn migrate_data_dir(new_path: String, clean_source: bool) -> Result<(), String> {
    let source_dir = modules::account::get_data_dir()?;
    let target_dir = std::path::PathBuf::from(new_path.trim());

    if target_dir.as_os_str().is_empty() {
        return Err("Target directory path cannot be empty".to_string());
    }

    // Canonicalize path to prevent comparison mistakes
    let canonical_source =
        std::fs::canonicalize(&source_dir).unwrap_or_else(|_| source_dir.clone());
    let canonical_target = if target_dir.exists() {
        std::fs::canonicalize(&target_dir).unwrap_or_else(|_| target_dir.clone())
    } else {
        target_dir.clone()
    };

    if canonical_source == canonical_target {
        return Err("Target directory cannot be identical to current data directory".to_string());
    }

    // Check if target directory is nested inside source directory
    if canonical_target.starts_with(&canonical_source) {
        return Err(
            "Target directory cannot be located inside the current data directory".to_string(),
        );
    }

    // Ensure target directory exists
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    // Execute recursive directory copy
    copy_dir_all_recursive(&source_dir, &target_dir)
        .map_err(|e| format!("Failed to copy data to new directory: {}", e))?;

    // Write persistent bootstrap pointer file
    if let Some(pointer_file) = modules::account::get_data_dir_pointer_file() {
        if let Some(parent) = pointer_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create pointer file directory: {}", e))?;
        }
        std::fs::write(&pointer_file, target_dir.to_string_lossy().trim())
            .map_err(|e| format!("Failed to save data directory configuration: {}", e))?;
    } else {
        return Err("Failed to resolve system config directory for data pointer".to_string());
    }

    // If user requested clean old dir and old dir is not a root/system directory
    if clean_source && source_dir.exists() {
        // Safety check: ensure source_dir is .antigravity_tools or contains accounts.json
        let has_accounts = source_dir.join("accounts.json").exists();
        let is_default_name =
            source_dir.file_name().and_then(|n| n.to_str()) == Some(".antigravity_tools");
        if has_accounts || is_default_name {
            if let Err(e) = std::fs::remove_dir_all(&source_dir) {
                tracing::warn!("Failed to clean source data directory after migration (files may be locked): {}", e);
            }
        }
    }

    Ok(())
}
