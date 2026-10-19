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

/// Copy text with the OS clipboard helper. Works when the webview document is not focused.
#[tauri::command]
pub fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    crate::modules::repo_db::copy_text_to_clipboard_sync(&text)
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

/// Read an image file directly from disk and return as a Base64 data URI (data:image/<ext>;base64,...)
/// for safe, taint-free clipboard copy and embedding in WebView2 without CORS or canvas security errors.
#[tauri::command]
pub async fn read_image_as_base64(path: String) -> Result<String, String> {
    let clean_path = crate::modules::repo_db::failed_commands::decode_uri_to_path(&path);
    let trimmed = clean_path.trim().replace(r"\\?\", "").replace(r"\?\", "");
    let requested = PathBuf::from(&trimmed);
    if requested.as_os_str().is_empty() {
        return Err("invalid_path: empty path".to_string());
    }
    let resolved = resolve_existing_or_parent(&requested)?;
    if is_sensitive_path(&resolved) || is_os_protected_path(&resolved) {
        return Err("security_denied: sensitive path".to_string());
    }
    let ext = resolved
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => return Err(format!("unsupported_image_extension: .{}", ext)),
    };
    let metadata =
        std::fs::metadata(&resolved).map_err(|e| format!("failed_to_read_metadata: {}", e))?;
    if !metadata.is_file() {
        return Err("invalid_path: expected a regular file".to_string());
    }
    if metadata.len() > 100 * 1024 * 1024 {
        return Err("file_too_large: image exceeds 100MB limit".to_string());
    }
    let bytes = std::fs::read(&resolved).map_err(|e| format!("failed_to_read_file: {}", e))?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

fn is_os_protected_path(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    let protected = [
        "/etc/",
        "/var/spool/cron",
        "/root/",
        "/proc/",
        "/sys/",
        "/dev/",
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\pagefile.sys",
    ];
    protected
        .iter()
        .any(|prefix| lower == *prefix || lower.starts_with(prefix))
}

/// Prompt-tree backups are JSON the user picks in a save/open dialog, including
/// under their own profile. OS directories stay blocked.
fn validate_prompt_backup_path(path: &str, must_exist: bool) -> Result<PathBuf, String> {
    let requested = PathBuf::from(path);
    if requested.as_os_str().is_empty() {
        return Err("invalid_path: empty path".to_string());
    }
    if !requested.is_absolute() {
        return Err("invalid_path: absolute path is required".to_string());
    }
    if is_os_protected_path(&requested) {
        return Err("security_denied: system path is not allowed".to_string());
    }
    let resolved = resolve_existing_or_parent(&requested)?;
    if is_os_protected_path(&resolved) {
        return Err("security_denied: system path is not allowed".to_string());
    }
    let is_json = resolved
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    if !is_json {
        return Err("invalid_path: prompt backup must be a .json file".to_string());
    }
    if must_exist {
        let metadata = std::fs::metadata(&resolved)
            .map_err(|e| format!("failed_to_read_file_metadata: {e}"))?;
        if !metadata.is_file() {
            return Err("invalid_path: expected a regular file".to_string());
        }
        if metadata.len() > 32 * 1024 * 1024 {
            return Err("invalid_path: backup file is larger than 32MB".to_string());
        }
    }
    Ok(resolved)
}

#[tauri::command]
pub async fn write_prompt_backup_file(path: String, content: String) -> Result<(), String> {
    let path = validate_prompt_backup_path(&path, false)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create backup folder: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("failed to write prompt backup: {e}"))
}

#[tauri::command]
pub async fn read_prompt_backup_file(path: String) -> Result<String, String> {
    let path = validate_prompt_backup_path(&path, true)?;
    std::fs::read_to_string(&path).map_err(|e| format!("failed to read prompt backup: {e}"))
}

/// Prompt-tree SQLite exports are user-picked `.db`/`.sqlite` files, including
/// under their own profile. OS directories stay blocked.
fn validate_prompt_tree_db_path(path: &str, must_exist: bool) -> Result<PathBuf, String> {
    let requested = PathBuf::from(path);
    if requested.as_os_str().is_empty() {
        return Err("invalid_path: empty path".to_string());
    }
    if !requested.is_absolute() {
        return Err("invalid_path: absolute path is required".to_string());
    }
    let resolved = resolve_existing_or_parent(&requested)?;
    if is_os_protected_path(&resolved) {
        return Err("security_denied: system path is not allowed".to_string());
    }
    let is_sqlite = resolved
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("db") || ext.eq_ignore_ascii_case("sqlite"))
        .unwrap_or(false);
    if !is_sqlite {
        return Err("invalid_path: prompt tree export must be a .db or .sqlite file".to_string());
    }
    if must_exist {
        let metadata = std::fs::metadata(&resolved)
            .map_err(|e| format!("failed_to_read_file_metadata: {e}"))?;
        if !metadata.is_file() {
            return Err("invalid_path: expected a regular file".to_string());
        }
        if metadata.len() > 512 * 1024 * 1024 {
            return Err("invalid_path: tree export file is larger than 512MB".to_string());
        }
    }
    Ok(resolved)
}

/// Export the full prompt tree (prompts, structure, metadata) to a portable
/// SQLite `.db` file at the dialog-chosen path.
#[tauri::command]
pub async fn export_prompt_tree_sqlite(
    path: String,
) -> Result<crate::modules::repo_db::TreeExportStats, String> {
    let path = validate_prompt_tree_db_path(&path, false)?;
    crate::modules::repo_db::write_prompt_tree_db(&path).map_err(|e| e.to_string())
}

/// Web (non-Tauri) export path: returns base64-encoded `.db` bytes for a
/// browser blob download.
#[tauri::command]
pub async fn export_prompt_tree_sqlite_bytes(
) -> Result<crate::modules::repo_db::TreeExportPayload, String> {
    crate::modules::repo_db::export_prompt_tree_db_bytes().map_err(|e| e.to_string())
}

/// Import a portable prompt-tree `.db` export into the live database.
/// The file is validated (SQLite magic header, required tables, export kind)
/// and real validation errors are reported — never silently swallowed.
#[tauri::command]
pub async fn import_prompt_tree_sqlite(path: String) -> Result<usize, String> {
    let path = validate_prompt_tree_db_path(&path, true)?;
    crate::modules::repo_db::import_prompt_tree_db(&path).map_err(|e| e.to_string())
}

/// Web (non-Tauri) import path: accepts base64-encoded `.db` bytes picked via
/// the file input and runs the same validated import pipeline.
#[tauri::command]
pub async fn import_prompt_tree_sqlite_bytes(data_b64: String) -> Result<usize, String> {
    crate::modules::repo_db::import_prompt_tree_db_bytes(&data_b64).map_err(|e| e.to_string())
}

#[cfg(test)]
mod prompt_backup_path_tests {
    use super::validate_prompt_backup_path;

    #[test]
    fn user_profile_json_is_allowed_and_windows_is_blocked() {
        let path = std::env::temp_dir().join("agm-prompt-backup-probe.json");
        let path_str = path.to_str().expect("temp path is utf-8");
        let resolved = validate_prompt_backup_path(path_str, false).expect("temp json allowed");
        assert!(resolved
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json")));
        let denied = validate_prompt_backup_path(r"C:\Windows\Temp\x.json", false);
        assert!(denied.is_err());
    }

    #[tokio::test]
    async fn read_image_as_base64_rejects_empty_and_system_path() {
        let empty_err = super::read_image_as_base64("".to_string()).await;
        assert!(empty_err.is_err());

        let win_err = super::read_image_as_base64(r"C:\Windows\System32\cmd.exe".to_string()).await;
        assert!(win_err.is_err());
    }

    #[tokio::test]
    async fn read_image_as_base64_encodes_png_file() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target_dir = manifest_dir.join("target");
        let _ = std::fs::create_dir_all(&target_dir); // Justification: best-effort test setup
        let test_png = target_dir.join("test_clipboard_image.png");
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG magic header
        std::fs::write(&test_png, &png_bytes).expect("write test png");
        let res = super::read_image_as_base64(test_png.to_string_lossy().to_string()).await;
        let _ = std::fs::remove_file(&test_png); // Justification: best-effort test cleanup
        assert!(res.is_ok());
        let uri = res.unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
    }

    #[tokio::test]
    async fn read_image_as_base64_resolves_file_protocol_uri() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target_dir = manifest_dir.join("target");
        let _ = std::fs::create_dir_all(&target_dir); // Justification: best-effort test setup
        let test_png = target_dir.join("test_file_uri_image.png");
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        std::fs::write(&test_png, &png_bytes).expect("write test png");
        let file_uri = format!("file:///{}", test_png.to_string_lossy().replace('\\', "/"));
        let res = super::read_image_as_base64(file_uri).await;
        let _ = std::fs::remove_file(&test_png); // Justification: best-effort test cleanup
        assert!(res.is_ok());
        let uri = res.unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_is_sensitive_path_permits_administrator_user_profile() {
        use super::is_sensitive_path;
        use std::path::Path;

        // Administrator user home directory must NOT be blocked
        assert!(!is_sensitive_path(Path::new(
            r"C:\Users\Administrator\Pictures\screenshot.png"
        )));
        assert!(!is_sensitive_path(Path::new(
            r"C:\Users\Administrator\.gemini\antigravity\test.json"
        )));
        assert!(!is_sensitive_path(Path::new(
            r"c:\users\administrator\workspace\repo\image.webp"
        )));

        // Windows and OS critical system directories MUST still be blocked
        assert!(is_sensitive_path(Path::new(r"C:\Windows\System32\cmd.exe")));
        assert!(is_sensitive_path(Path::new(
            r"C:\Program Files\Antigravity\app.exe"
        )));
        assert!(is_sensitive_path(Path::new(
            r"C:\Program Files (x86)\App\binary.dll"
        )));
        assert!(is_sensitive_path(Path::new(r"/etc/shadow")));
        assert!(is_sensitive_path(Path::new(r"/root/.ssh/id_rsa")));
        assert!(is_sensitive_path(Path::new(r"C:\pagefile.sys")));
    }
}
