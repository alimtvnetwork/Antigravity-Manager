use crate::modules::security_db::{
    add_to_blacklist, add_to_whitelist, cleanup_old_ip_logs, clear_ip_access_logs, get_blacklist,
    get_blacklist_entry_for_ip, get_ip_access_logs, get_ip_stats, get_whitelist, init_db,
    is_ip_in_blacklist, is_ip_in_whitelist, remove_from_blacklist, remove_from_whitelist,
    save_ip_access_log, IpAccessLog,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// 辅助函数：获取当前时间戳
fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// 辅助函数：初始化测试并加锁隔离
fn setup_test() -> (
    std::sync::MutexGuard<'static, ()>,
    std::sync::MutexGuard<'static, ()>,
) {
    let env_lock = crate::modules::account::TEST_DATA_DIR_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(init_db(), "init_db");
    cleanup_test_data();
    (env_lock, lock)
}

/// 辅助函数：清理测试环境
fn cleanup_test_data() {
    // 清理黑名单
    if let Ok(entries) = get_blacklist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(remove_from_blacklist(&entry.id), "remove_from_blacklist");
        }
    }
    // 清理白名单
    if let Ok(entries) = get_whitelist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(remove_from_whitelist(&entry.id), "remove_from_whitelist");
        }
    }
    // 清理访问日志
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(clear_ip_access_logs(), "clear_ip_access_logs");
}

mod access_log;
mod blacklist;
mod cidr;
mod concurrency_edge;
mod db_init;
mod expiration;
mod whitelist;
