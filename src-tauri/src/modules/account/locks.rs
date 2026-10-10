use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock, RwLock};

use super::*;

pub(crate) static ACCOUNT_FILE_LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub(crate) fn get_account_lock(account_id: &str) -> Arc<Mutex<()>> {
    let mut locks = ACCOUNT_FILE_LOCKS.lock().unwrap();
    locks
        .entry(account_id.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

#[cfg(test)]
pub static TEST_DATA_DIR_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Global account write lock to prevent corruption during concurrent operations
pub(crate) static ACCOUNT_INDEX_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(crate) fn lock_account_file_updates() -> Result<std::sync::MutexGuard<'static, ()>, String> {
    ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))
}
