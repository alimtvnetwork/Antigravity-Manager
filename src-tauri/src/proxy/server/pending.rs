//! Pending account reload/delete queues shared with TokenManager.
use std::collections::HashSet;
use std::sync::OnceLock;

// TokenManager checks and processes these accounts during get_token
static PENDING_RELOAD_ACCOUNTS: OnceLock<std::sync::RwLock<HashSet<String>>> = OnceLock::new();

// [NEW] Global accounts queue pending deletion (Issue #1477)
// When account is deleted, account ID is enqueued here to purge memory cache in TokenManager
static PENDING_DELETE_ACCOUNTS: OnceLock<std::sync::RwLock<HashSet<String>>> = OnceLock::new();

fn get_pending_reload_accounts() -> &'static std::sync::RwLock<HashSet<String>> {
    PENDING_RELOAD_ACCOUNTS.get_or_init(|| std::sync::RwLock::new(HashSet::new()))
}

fn get_pending_delete_accounts() -> &'static std::sync::RwLock<HashSet<String>> {
    PENDING_DELETE_ACCOUNTS.get_or_init(|| std::sync::RwLock::new(HashSet::new()))
}

/// Trigger account reload signal (called by update_account_quota)
pub fn trigger_account_reload(account_id: &str) {
    if let Ok(mut pending) = get_pending_reload_accounts().write() {
        pending.insert(account_id.to_string());
        tracing::debug!(
            "[Quota] Queued account {} for TokenManager reload",
            account_id
        );
    }
}

/// Trigger account deletion signal (Issue #1477)
pub fn trigger_account_delete(account_id: &str) {
    if let Ok(mut pending) = get_pending_delete_accounts().write() {
        pending.insert(account_id.to_string());
        tracing::debug!("[Proxy] Queued account {} for cache removal", account_id);
    }
}

/// Retrieve and drain accounts pending reload (called by TokenManager)
pub fn take_pending_reload_accounts() -> Vec<String> {
    if let Ok(mut pending) = get_pending_reload_accounts().write() {
        let accounts: Vec<String> = pending.drain().collect();
        if !accounts.is_empty() {
            tracing::debug!(
                "[Quota] Taking {} pending accounts for reload",
                accounts.len()
            );
        }
        accounts
    } else {
        Vec::new()
    }
}

/// Retrieve and drain accounts pending deletion (Issue #1477)
pub fn take_pending_delete_accounts() -> Vec<String> {
    if let Ok(mut pending) = get_pending_delete_accounts().write() {
        let accounts: Vec<String> = pending.drain().collect();
        if !accounts.is_empty() {
            tracing::debug!(
                "[Proxy] Taking {} pending accounts for cache removal",
                accounts.len()
            );
        }
        accounts
    } else {
        Vec::new()
    }
}
