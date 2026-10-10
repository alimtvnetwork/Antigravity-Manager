use std::fs;
use std::path::Path;

use super::*;

/// Backup email_vault.db and split email_passwords.db to target path
pub fn backup_vault_db(target_path: &Path) -> Result<(), String> {
    let source_path = email_vault_db::get_email_vault_db_path()?;
    let is_source_exists = source_path.exists();
    if !is_source_exists {
        return Err("Email vault database does not exist yet".to_string());
    }

    fs::copy(&source_path, target_path)
        .map_err(|e| format!("Failed to backup email vault database: {}", e))?;

    // Also backup split passwords database alongside target if present
    if let Ok(pass_src) = email_vault_db::get_email_passwords_db_path() {
        let is_pass_exists = pass_src.exists();
        if is_pass_exists {
            let pass_target = target_path.with_extension("passwords.db");
            // Justification: best-effort file copy; logged for diagnosis
            crate::error::record_ignored(fs::copy(&pass_src, &pass_target), "fs::copy");
        }
    }

    Ok(())
}

/// Restore email_vault.db and split email_passwords.db from source backup file
pub fn restore_vault_db(source_path: &Path) -> Result<(), String> {
    let is_source_exists = source_path.exists();
    if !is_source_exists {
        return Err("Source database file does not exist".to_string());
    }

    let target_path = email_vault_db::get_email_vault_db_path()?;
    fs::copy(source_path, &target_path)
        .map_err(|e| format!("Failed to restore email vault database: {}", e))?;

    // Restore companion split passwords database if backup file exists
    let pass_src = source_path.with_extension("passwords.db");
    let is_pass_src_exists = pass_src.exists();
    if is_pass_src_exists {
        if let Ok(pass_target) = email_vault_db::get_email_passwords_db_path() {
            // Justification: best-effort file copy; logged for diagnosis
            crate::error::record_ignored(fs::copy(&pass_src, &pass_target), "fs::copy");
        }
    }

    // Re-verify connectivity and run pragmas
    let _ = email_vault_db::connect_vault_db()?;
    let _ = email_vault_db::connect_passwords_db()?;
    Ok(())
}
