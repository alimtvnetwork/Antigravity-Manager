use super::*;

fn resolve_existing_or_parent(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return path
            .canonicalize()
            .map_err(|e| format!("failed_to_resolve_path: {}", e));
    }

    let parent = path
        .parent()
        .ok_or_else(|| "invalid_path: missing parent directory".to_string())?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| format!("failed_to_resolve_parent: {}", e))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "invalid_path: missing file name".to_string())?;
    Ok(canonical_parent.join(file_name))
}

fn is_sensitive_path(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    let sensitive_prefixes = [
        "/etc/",
        "/var/spool/cron",
        "/root/",
        "/proc/",
        "/sys/",
        "/dev/",
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\users\\administrator",
        "c:\\pagefile.sys",
    ];

    sensitive_prefixes
        .iter()
        .any(|prefix| lower == *prefix || lower.starts_with(prefix))
}

fn validate_user_json_path(path: &str, must_exist: bool) -> Result<PathBuf, String> {
    let requested = PathBuf::from(path);
    if requested.as_os_str().is_empty() {
        return Err("invalid_path: empty path".to_string());
    }
    if !requested.is_absolute() {
        return Err("invalid_path: absolute path is required".to_string());
    }

    let resolved = resolve_existing_or_parent(&requested)?;
    if is_sensitive_path(&resolved) {
        return Err("security_denied: sensitive system path is not allowed".to_string());
    }

    let is_allowed_ext = resolved
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            matches!(
                lower.as_str(),
                "json" | "csv" | "yaml" | "yml" | "txt" | "xlsx" | "xls"
            )
        })
        .unwrap_or(false);
    if !is_allowed_ext {
        return Err("invalid_path: only text configuration files (.json, .csv, .yaml, .yml, .txt, .xlsx) are allowed".to_string());
    }

    if must_exist {
        let metadata = std::fs::metadata(&resolved)
            .map_err(|e| format!("failed_to_read_file_metadata: {}", e))?;
        if !metadata.is_file() {
            return Err("invalid_path: expected a regular file".to_string());
        }
    }

    Ok(resolved)
}

/// 保存文本文件 (绕过前端 Scope 限制)
#[tauri::command]
pub async fn save_text_file(path: String, content: String) -> Result<(), String> {
    let path = validate_user_json_path(&path, false)?;
    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 读取文本文件 (绕过前端 Scope 限制)
#[tauri::command]
pub async fn read_text_file(path: String) -> Result<String, String> {
    let path = validate_user_json_path(&path, true)?;
    std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))
}
