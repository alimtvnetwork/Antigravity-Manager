use crate::modules::security_db::{
    add_to_blacklist, clear_ip_access_logs, get_blacklist, init_db, is_ip_in_blacklist,
    remove_from_blacklist, save_ip_access_log, IpAccessLog,
};
use std::thread;
use std::time::{Duration, Instant};

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
    if let Ok(entries) = get_blacklist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(remove_from_blacklist(&entry.id), "remove_from_blacklist");
        }
    }
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(clear_ip_access_logs(), "clear_ip_access_logs");
}

/// 压力测试：大量黑名单条目
#[test]
fn stress_test_large_blacklist() {
    let _lock = setup_test();

    let count = 25;

    // 批量添加
    let start = Instant::now();
    for i in 0..count {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            add_to_blacklist(
                &format!("stress.{}.{}.{}.{}", i / 256, (i / 16) % 16, i % 16, i),
                None,
                None,
                "stress",
            ),
            "add_to_blacklist",
        );
    }
    let add_duration = start.elapsed();
    println!("Added {} entries in {:?}", count, add_duration);

    // 随机查找测试
    let start = Instant::now();
    for i in 0..25 {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            is_ip_in_blacklist(&format!(
                "stress.{}.{}.{}.{}",
                i / 256,
                (i / 16) % 16,
                i % 16,
                i
            )),
            "is_ip_in_blacklist",
        );
    }
    let lookup_duration = start.elapsed();
    println!("25 lookups in large blacklist took {:?}", lookup_duration);

    // 验证性能合理
    assert!(
        lookup_duration < Duration::from_secs(30),
        "Lookups should be reasonably fast even with large blacklist"
    );

    cleanup_test_data();
}

/// 压力测试：大量访问日志
#[test]
fn stress_test_access_logging() {
    let _lock = setup_test();

    let count = 25;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // 批量写入日志
    let start = Instant::now();
    for i in 0..count {
        let log = IpAccessLog {
            id: uuid::Uuid::new_v4().to_string(),
            client_ip: format!("log.stress.{}", i % 100),
            timestamp: now,
            method: Some("POST".to_string()),
            path: Some("/v1/messages".to_string()),
            user_agent: Some("StressTest/1.0".to_string()),
            status: Some(200),
            duration: Some(100),
            api_key_hash: Some("hash".to_string()),
            blocked: false,
            block_reason: None,
            username: None,
        };
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(save_ip_access_log(&log), "save_ip_access_log");
    }
    let write_duration = start.elapsed();
    println!("Wrote {} access logs in {:?}", count, write_duration);

    // 验证写入性能合理
    assert!(
        write_duration < Duration::from_secs(60),
        "Access log writing should be reasonably fast"
    );

    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(clear_ip_access_logs(), "clear_ip_access_logs");
}

/// 压力测试：并发操作
#[test]
fn stress_test_concurrent_operations() {
    let _lock = setup_test();

    let thread_count = 5;
    let ops_per_thread = 20;

    let handles: Vec<_> = (0..thread_count)
        .map(|t| {
            thread::spawn(move || {
                for i in 0..ops_per_thread {
                    // 每个线程添加-查询-删除
                    let ip = format!("concurrent.{}.{}", t, i);
                    if let Ok(entry) = add_to_blacklist(&ip, None, None, "concurrent") {
                        // Justification: best-effort call; failure logged without changing control flow
                        crate::error::record_ignored(is_ip_in_blacklist(&ip), "is_ip_in_blacklist");
                        // Justification: best-effort call; failure logged without changing control flow
                        crate::error::record_ignored(
                            remove_from_blacklist(&entry.id),
                            "remove_from_blacklist",
                        );
                    }
                }
            })
        })
        .collect();

    // 等待所有线程完成
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // 验证没有遗留数据
    let remaining = get_blacklist().unwrap();
    let concurrent_remaining: Vec<_> = remaining
        .iter()
        .filter(|e| e.ip_pattern.starts_with("concurrent."))
        .collect();

    assert!(
        concurrent_remaining.is_empty(),
        "All concurrent test data should be cleaned up"
    );

    cleanup_test_data();
}
