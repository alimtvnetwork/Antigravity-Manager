pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::*;

/// Get accounts directory path
pub fn get_accounts_dir() -> Result<PathBuf, String> {
    let data_dir = get_data_dir()?;
    let accounts_dir = data_dir.join(ACCOUNTS_DIR);

    if !accounts_dir.exists() {
        fs::create_dir_all(&accounts_dir)
            .map_err(|e| format!("failed_to_create_accounts_dir: {}", e))?;
    }

    Ok(accounts_dir)
}

/// Load account index from a specific directory (internal helper)
pub(crate) fn load_account_index_in_dir(data_dir: &PathBuf) -> Result<AccountIndex, String> {
    let index_path = data_dir.join(ACCOUNTS_INDEX);

    if !index_path.exists() {
        crate::modules::logger::log_warn(
            "Account index file not found, attempting recovery from accounts directory",
        );
        let recovered = rebuild_index_from_accounts_in_dir(data_dir)?;
        try_save_recovered_index(data_dir, &index_path, &recovered, None)?;
        return Ok(recovered);
    }

    let raw_content =
        fs::read(&index_path).map_err(|e| format!("failed_to_read_account_index: {}", e))?;

    // If file is empty, attempt recovery
    if raw_content.is_empty() {
        crate::modules::logger::log_warn(
            "Account index is empty, attempting recovery from accounts directory",
        );
        let recovered = rebuild_index_from_accounts_in_dir(data_dir)?;
        try_save_recovered_index(data_dir, &index_path, &recovered, None)?;
        return Ok(recovered);
    }

    // Sanitize content: strip BOM and leading NUL bytes
    let sanitized = sanitize_index_content(&raw_content);

    // If sanitized content is empty/whitespace, attempt recovery
    if sanitized.trim().is_empty() {
        crate::modules::logger::log_warn(
            "Account index is empty after sanitization, attempting recovery from accounts directory",
        );
        let recovered = rebuild_index_from_accounts_in_dir(data_dir)?;
        try_save_recovered_index(data_dir, &index_path, &recovered, None)?;
        return Ok(recovered);
    }

    // Try to parse sanitized content (flat or enveloped with variable expansion)
    let parsed_result = crate::modules::json_envelope::extract_payload::<AccountIndex>(&sanitized)
        .map(|(data, _)| data)
        .or_else(|_| serde_json::from_str::<AccountIndex>(&sanitized));
    match parsed_result {
        Ok(index) => {
            crate::modules::logger::log_info(&format!(
                "Successfully loaded index with {} accounts",
                index.accounts.len()
            ));
            Ok(index)
        }
        Err(parse_err) => {
            crate::modules::logger::log_error(&format!(
                "Failed to parse account index: {}. Attempting recovery from accounts directory",
                parse_err
            ));
            let recovered = rebuild_index_from_accounts_in_dir(data_dir)?;
            try_save_recovered_index(data_dir, &index_path, &recovered, Some(&raw_content))?;
            Ok(recovered)
        }
    }
}

/// Save account index to a specific directory (internal helper)
pub(crate) fn save_account_index_in_dir(
    data_dir: &PathBuf,
    index: &AccountIndex,
) -> Result<(), String> {
    let index_path = data_dir.join(ACCOUNTS_INDEX);

    let content = serde_json::to_string_pretty(index)
        .map_err(|e| format!("failed_to_serialize_account_index: {}", e))?;

    crate::utils::fs::write_atomic(&index_path, content.as_bytes())
        .map_err(|e| format!("failed_to_save_account_index: {}", e))
}

/// Export account index wrapped in standard JSON envelope with variable section
pub fn export_accounts_envelope() -> Result<String, String> {
    let index = load_account_index()?;
    let envelope = crate::modules::json_envelope::JsonEnvelope::new("agm/accounts-export", index);
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("failed_to_serialize_accounts_envelope: {}", e))
}

/// Rebuild AccountIndex by scanning accounts/*.json files in specific directory
pub(crate) fn rebuild_index_from_accounts_in_dir(
    data_dir: &PathBuf,
) -> Result<AccountIndex, String> {
    let accounts_dir = data_dir.join(ACCOUNTS_DIR);
    let mut summaries = Vec::new();

    if accounts_dir.exists() {
        if let Ok(entries) = fs::read_dir(&accounts_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Some(account_id) = path.file_stem().and_then(|s| s.to_str()) {
                        match load_account_at_path(&path) {
                            Ok(account) => {
                                summaries.push(AccountSummary {
                                    id: account.id,
                                    email: account.email,
                                    name: account.name,
                                    disabled: account.disabled,
                                    proxy_disabled: account.proxy_disabled,
                                    protected_models: account.protected_models,
                                    created_at: account.created_at,
                                    last_used: account.last_used,
                                });
                            }
                            Err(e) => {
                                crate::modules::logger::log_warn(&format!(
                                    "Failed to load account {} during recovery: {}",
                                    account_id, e
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by last_used desc, then by email for deterministic order
    summaries.sort_by(|a, b| {
        b.last_used
            .cmp(&a.last_used)
            .then_with(|| a.email.cmp(&b.email))
    });

    let current_account_id = summaries.first().map(|s| s.id.clone());

    crate::modules::logger::log_info(&format!(
        "Rebuilt index from accounts directory: {} accounts recovered",
        summaries.len()
    ));

    Ok(AccountIndex {
        version: "2.0".to_string(),
        accounts: summaries,
        current_account_id,
        current_target_ide: None,
    })
}

/// Load account from a specific path with self-healing support for trailing characters/corrupted suffixes
pub(crate) fn load_account_at_path(account_path: &PathBuf) -> Result<Account, String> {
    let content = fs::read_to_string(account_path)
        .map_err(|e| format!("failed_to_read_account_data: {}", e))?;

    match serde_json::from_str::<Account>(&content) {
        Ok(account) => Ok(account),
        Err(e) => {
            let err_msg = e.to_string();
            // Self-healing attempt: handle trailing characters / extra closing brackets
            if err_msg.contains("trailing characters")
                || err_msg.contains("trailing comma")
                || err_msg.contains("trailing")
            {
                let mut de = serde_json::Deserializer::from_str(&content);
                if let Ok(account) = serde::Deserialize::deserialize(&mut de) {
                    crate::modules::logger::log_warn(&format!(
                        "Self-healing account JSON at {:?}: recovered valid account data from trailing characters, saving clean file",
                        account_path
                    ));
                    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                    crate::error::record_ignored(
                        save_account_at_path(account_path, &account),
                        "save_account_at_path",
                    );
                    return Ok(account);
                }
            }
            Err(format!("failed_to_parse_account_data: {}", err_msg))
        }
    }
}

/// Load account index with recovery support
pub fn load_account_index() -> Result<AccountIndex, String> {
    let data_dir = get_data_dir()?;
    load_account_index_in_dir(&data_dir)
}

/// Sanitize index file content by stripping BOM and leading NUL bytes
pub(crate) fn sanitize_index_content(raw: &[u8]) -> String {
    // Skip UTF-8 BOM if present
    let without_bom = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        raw
    };

    // Skip leading NUL bytes
    let without_nul = without_bom
        .iter()
        .skip_while(|&&b| b == 0x00)
        .copied()
        .collect::<Vec<u8>>();

    // Convert to string (lossy - invalid UTF-8 sequences become replacement chars)
    String::from_utf8_lossy(&without_nul).into_owned()
}

/// Best-effort save of recovered index without deadlocking
pub(crate) fn try_save_recovered_index(
    data_dir: &PathBuf,
    _index_path: &PathBuf,
    index: &AccountIndex,
    corrupt_content: Option<&[u8]>,
) -> Result<(), String> {
    // Backup corrupt file if content provided
    if let Some(content) = corrupt_content {
        let timestamp = chrono::Utc::now().timestamp();
        let backup_name = format!("accounts.json.corrupt-{}-{}", timestamp, Uuid::new_v4());
        let backup_path = data_dir.join(&backup_name);
        if let Err(e) = crate::utils::fs::write_atomic(&backup_path, content) {
            crate::modules::logger::log_warn(&format!(
                "Failed to backup corrupt index to {}: {}",
                backup_name, e
            ));
        } else {
            crate::modules::logger::log_info(&format!(
                "Backed up corrupt index to {}",
                backup_name
            ));
        }
    }

    // Try to acquire lock without blocking - if we can't get it, skip saving
    match ACCOUNT_INDEX_LOCK.try_lock() {
        Ok(_guard) => {
            if let Err(e) = save_account_index_in_dir(data_dir, index) {
                crate::modules::logger::log_warn(&format!(
                    "Failed to save recovered index: {}. Will retry on next load.",
                    e
                ));
            } else {
                crate::modules::logger::log_info("Successfully saved recovered index");
            }
        }
        Err(_) => {
            crate::modules::logger::log_warn(
                "Could not acquire lock to save recovered index. Will retry on next load.",
            );
        }
    }

    Ok(())
}

/// Save account index (atomic write)
pub fn save_account_index(index: &AccountIndex) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    save_account_index_in_dir(&data_dir, index)
}
