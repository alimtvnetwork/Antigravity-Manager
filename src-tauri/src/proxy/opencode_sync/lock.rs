use serde_json::Value;
use std::fs;

pub(crate) const CREATE_NO_WINDOW: u32 = 0x08000000;

pub(crate) const OPENCODE_DIR: &str = ".config/opencode";
pub(crate) const OPENCODE_CONFIG_FILE: &str = "opencode.json";
pub(crate) const OPENCODE_CONFIG_FILE_JSONC: &str = "opencode.jsonc";
pub(crate) const ANTIGRAVITY_CONFIG_FILE: &str = "antigravity.json";
pub(crate) const ANTIGRAVITY_ACCOUNTS_FILE: &str = "antigravity-accounts.json";
pub(crate) const BACKUP_SUFFIX: &str = ".antigravity-manager.bak";
pub(crate) const OLD_BACKUP_SUFFIX: &str = ".antigravity.bak";

pub(crate) const ANTIGRAVITY_PROVIDER_ID: &str = "antigravity-manager";
pub(crate) const APIKEY_FUN_PROVIDER_ID: &str = "apikey-fun";
pub(crate) const MAX_PROVIDER_ID_LEN: usize = 128;
pub(crate) const OPENAI_COMPATIBLE_NPM: &str = "@ai-sdk/openai-compatible";

pub(crate) static OPENCODE_CONFIG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn acquire_opencode_config_lock() -> std::sync::MutexGuard<'static, ()> {
    OPENCODE_CONFIG_MUTEX.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("OPENCODE_CONFIG_MUTEX was poisoned, recovering lock");
        poisoned.into_inner()
    })
}

pub(crate) fn atomically_write_config(
    config_path: &std::path::Path,
    config: &Value,
) -> Result<(), String> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
    }

    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    let file_name = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config.json");
    let tmp_path =
        config_path.with_file_name(format!("{}.tmp.{}", file_name, uuid::Uuid::new_v4()));

    // Set permissions at creation, before any credentials reach the file.
    // create_new also refuses to follow a pre-existing temporary-file symlink.
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&tmp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let write_result = (|| -> std::io::Result<()> {
        use std::io::Write;
        #[cfg(unix)]
        if let Ok(metadata) = fs::metadata(config_path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.write_all(json_str.as_bytes())?;
        file.sync_all()
    })();
    drop(file);
    if let Err(e) = write_result {
        // Justification: best-effort cleanup; a leftover file is harmless
        crate::error::record_ignored(fs::remove_file(&tmp_path), "remove_file");
        return Err(format!("Failed to write temp file: {}", e));
    }

    fs::rename(&tmp_path, config_path).map_err(|e| {
        // Justification: best-effort cleanup; a leftover file is harmless
        crate::error::record_ignored(fs::remove_file(&tmp_path), "remove_file");
        format!("Failed to rename config file: {}", e)
    })?;
    Ok(())
}
