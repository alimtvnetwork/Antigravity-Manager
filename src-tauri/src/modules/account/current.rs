pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};

use super::*;

/// Get current account ID
pub fn get_current_account_id() -> Result<Option<String>, String> {
    let index = load_account_index()?;
    Ok(index.current_account_id)
}

/// Get currently active account details
pub fn get_current_account() -> Result<Option<Account>, String> {
    if let Some(id) = get_current_account_id()? {
        Ok(Some(load_account(&id)?))
    } else {
        Ok(None)
    }
}

/// Set current active account ID
pub fn set_current_account_id(account_id: &str) -> Result<(), String> {
    set_current_account_id_with_target(account_id, None)
}

/// Set current active account ID and target IDE
pub fn set_current_account_id_with_target(
    account_id: &str,
    target_ide: Option<&str>,
) -> Result<(), String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;
    let mut index = load_account_index()?;
    index.current_account_id = Some(account_id.to_string());
    index.current_target_ide = target_ide.map(|s| s.to_string());
    save_account_index(&index)
}
