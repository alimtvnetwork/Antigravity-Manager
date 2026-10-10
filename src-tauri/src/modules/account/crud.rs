pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::*;

/// Load account data
pub fn load_account(account_id: &str) -> Result<Account, String> {
    let accounts_dir = get_accounts_dir()?;
    let account_path = accounts_dir.join(format!("{}.json", account_id));
    load_account_at_path(&account_path)
}

/// Find an account by its email address
pub fn get_account_by_email(email: &str) -> Result<Option<Account>, String> {
    let index = load_account_index()?;
    let target = email.trim();
    if let Some(summary) = index
        .accounts
        .iter()
        .find(|a| a.email.trim().eq_ignore_ascii_case(target))
    {
        load_account(&summary.id).map(Some)
    } else {
        Ok(None)
    }
}

/// Save account data at specific file path (thread-safe and atomic)
pub(crate) fn save_account_at_path(
    account_path: &PathBuf,
    account: &Account,
) -> Result<(), String> {
    let _lock = get_account_lock(&account.id);
    let _guard = _lock.lock().unwrap();

    let content = serde_json::to_string_pretty(account)
        .map_err(|e| format!("failed_to_serialize_account_data: {}", e))?;

    crate::utils::fs::write_atomic(account_path, content.as_bytes())
        .map_err(|e| format!("failed_to_save_account_file: {}", e))
}

/// Save account data (thread-safe and atomic)
pub fn save_account(account: &Account) -> Result<(), String> {
    let accounts_dir = get_accounts_dir()?;
    let account_path = accounts_dir.join(format!("{}.json", account.id));
    save_account_at_path(&account_path, account)
}

/// Save proxy selection priority using the shared account write lock.
pub fn update_account_priority(account_id: &str, priority: u8) -> Result<(), String> {
    crate::models::account::validate_priority(priority)?;
    let _account_write = lock_account_file_updates()?;
    let mut account = load_account(account_id)?;
    account.priority = priority;
    save_account(&account)
}

/// List all accounts
pub fn list_accounts() -> Result<Vec<Account>, String> {
    crate::modules::logger::log_info("Listing accounts...");
    let index = load_account_index()?;
    let mut accounts = Vec::new();

    for summary in &index.accounts {
        match load_account(&summary.id) {
            Ok(account) => accounts.push(account),
            Err(e) => {
                crate::modules::logger::log_error(&format!(
                    "Failed to load account {}: {}",
                    summary.id, e
                ));
                // [FIX #929] Removed auto-repair logic.
                // We no longer silently delete account IDs from the index if the file is missing.
                // This prevents account loss during version upgrades or temporary FS issues.
            }
        }
    }

    if let Err(error) = crate::modules::token_stats::populate_weekly_usage(&mut accounts) {
        tracing::warn!("Weekly token usage unavailable: {}", error);
    }
    Ok(accounts)
}

/// Add account
pub fn add_account(
    email: String,
    name: Option<String>,
    token: TokenData,
) -> Result<Account, String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;
    let mut index = load_account_index()?;

    // Check if account already exists
    if index.accounts.iter().any(|s| s.email == email) {
        return Err(format!("Account already exists: {}", email));
    }

    // Create new account
    let account_id = Uuid::new_v4().to_string();
    let mut account = Account::new(account_id.clone(), email.clone(), token);
    account.name = name.clone();

    // Save account data
    save_account(&account)?;

    // Update index
    index.accounts.push(AccountSummary {
        id: account.id.clone(),
        email: account.email.clone(),
        name: account.name.clone(),
        disabled: account.disabled,
        proxy_disabled: account.proxy_disabled,
        protected_models: account.protected_models.clone(),
        created_at: account.created_at,
        last_used: account.last_used,
    });

    // If first account, set as current
    if index.current_account_id.is_none() {
        index.current_account_id = Some(account_id);
    }

    save_account_index(&index)?;

    crate::modules::task_history_db::record(
        crate::modules::audit_action::AuditAction::AddAccount,
        &account.email,
        "ok",
        "account saved",
        None,
    );

    Ok(account)
}

/// Add or update account
pub fn upsert_account(
    email: String,
    name: Option<String>,
    token: TokenData,
) -> Result<Account, String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;
    let mut index = load_account_index()?;

    // Find account ID if exists
    let existing_account_id = index
        .accounts
        .iter()
        .find(|s| s.email == email)
        .map(|s| s.id.clone());

    if let Some(account_id) = existing_account_id {
        // Update existing account
        match load_account(&account_id) {
            Ok(mut account) => {
                let old_access_token = account.token.access_token.clone();
                let old_refresh_token = account.token.refresh_token.clone();
                account.token = token;
                account.name = name.clone();
                // If an account was previously disabled (e.g. invalid_grant), any explicit token upsert
                // should re-enable it (user manually updated credentials in the UI).
                if account.disabled
                    && (account.token.refresh_token != old_refresh_token
                        || account.token.access_token != old_access_token)
                {
                    account.disabled = false;
                    account.disabled_reason = None;
                    account.disabled_at = None;
                }
                account.update_last_used();
                save_account(&account)?;

                // Sync name in index
                if let Some(idx_summary) = index.accounts.iter_mut().find(|s| s.id == account_id) {
                    idx_summary.name = name;
                    save_account_index(&index)?;
                }

                crate::modules::task_history_db::record(
                    crate::modules::audit_action::AuditAction::UpdateAccount,
                    &account.email,
                    "ok",
                    "account credentials updated",
                    None,
                );
                return Ok(account);
            }
            Err(e) => {
                crate::modules::logger::log_warn(&format!(
                    "Account {} file missing ({}), recreating...",
                    account_id, e
                ));
                // Index exists but file is missing, recreating
                let mut account = Account::new(account_id.clone(), email.clone(), token);
                account.name = name.clone();
                save_account(&account)?;

                // Sync name in index
                if let Some(idx_summary) = index.accounts.iter_mut().find(|s| s.id == account_id) {
                    idx_summary.name = name;
                    save_account_index(&index)?;
                }

                crate::modules::task_history_db::record(
                    crate::modules::audit_action::AuditAction::UpdateAccount,
                    &account.email,
                    "ok",
                    "account file recreated",
                    None,
                );
                return Ok(account);
            }
        }
    }

    // Add if not exists
    // Note: add_account will attempt to acquire lock, which would deadlock here.
    // Use an internal version or release lock.

    // Release lock, let add_account handle it
    drop(_lock);
    add_account(email, name, token)
}

/// Delete account
pub fn delete_account(account_id: &str) -> Result<(), String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;
    let mut index = load_account_index()?;

    // Remove from index
    let original_len = index.accounts.len();
    index.accounts.retain(|s| s.id != account_id);

    // If account was present in index, update current account and persist index
    if index.accounts.len() != original_len {
        if index.current_account_id.as_deref() == Some(account_id) {
            index.current_account_id = index.accounts.first().map(|s| s.id.clone());
        }
        save_account_index(&index)?;
    } else {
        tracing::info!(
            "[delete_account] Account ID {} already absent from index; proceeding with idempotent cleanup",
            account_id
        );
    }

    // Delete account file
    let accounts_dir = get_accounts_dir()?;
    let account_path = accounts_dir.join(format!("{}.json", account_id));

    if account_path.exists() {
        fs::remove_file(&account_path)
            .map_err(|e| format!("failed_to_delete_account_file: {}", e))?;
    }

    // [FIX #1477] Trigger TokenManager cache cleanup signal
    crate::proxy::server::trigger_account_delete(account_id);

    Ok(())
}

/// Batch delete accounts (atomic index operation)
pub fn delete_accounts(account_ids: &[String]) -> Result<(), String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;
    let mut index = load_account_index()?;

    let accounts_dir = get_accounts_dir()?;

    for account_id in account_ids {
        // Remove from index
        index.accounts.retain(|s| &s.id != account_id);

        // Clear current account if it's being deleted
        if index.current_account_id.as_deref() == Some(account_id) {
            index.current_account_id = None;
        }

        // Delete account file
        let account_path = accounts_dir.join(format!("{}.json", account_id));
        if account_path.exists() {
            // Justification: best-effort cleanup; a leftover file is harmless
            crate::error::record_ignored(fs::remove_file(&account_path), "remove_file");
        }

        // [FIX #1477] Trigger TokenManager cache cleanup signal
        crate::proxy::server::trigger_account_delete(account_id);
    }

    // If current account is empty, use first one as default
    if index.current_account_id.is_none() {
        index.current_account_id = index.accounts.first().map(|s| s.id.clone());
    }

    save_account_index(&index)
}

/// Reorder account list
/// Update account order in index file based on provided IDs
pub fn reorder_accounts(account_ids: &[String]) -> Result<(), String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;
    let mut index = load_account_index()?;

    // Create a map of account ID to summary
    let id_to_summary: std::collections::HashMap<_, _> = index
        .accounts
        .iter()
        .map(|s| (s.id.clone(), s.clone()))
        .collect();

    // Rebuild account list with new order
    let mut new_accounts = Vec::new();
    for id in account_ids {
        if let Some(summary) = id_to_summary.get(id) {
            new_accounts.push(summary.clone());
        }
    }

    // Add accounts missing from new order to the end
    for summary in &index.accounts {
        if !account_ids.contains(&summary.id) {
            new_accounts.push(summary.clone());
        }
    }

    index.accounts = new_accounts;

    crate::modules::logger::log_info(&format!(
        "Account order updated, {} accounts total",
        index.accounts.len()
    ));

    save_account_index(&index)
}
