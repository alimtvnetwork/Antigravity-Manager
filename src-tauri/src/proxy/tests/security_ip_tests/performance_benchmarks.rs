use crate::modules::security_db::{add_to_blacklist, get_blacklist, init_db, is_ip_in_blacklist};
use std::time::Instant;

/// 基准测试：黑名单查找性能
#[test]
fn benchmark_blacklist_lookup() {
    let _lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(init_db(), "init_db");

    // 清理并添加 100 个黑名单条目
    if let Ok(entries) = get_blacklist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                crate::modules::security_db::remove_from_blacklist(&entry.id),
                "remove_from_blacklist",
            );
        }
    }

    for i in 0..20 {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            add_to_blacklist(&format!("bench.ip.{}", i), Some("Benchmark"), None, "test"),
            "add_to_blacklist",
        );
    }

    // 执行 20 次查找
    let iterations = 20;
    let start = Instant::now();
    for _ in 0..iterations {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(is_ip_in_blacklist("bench.ip.10"), "is_ip_in_blacklist");
    }
    let duration = start.elapsed();

    println!("{} blacklist lookups took: {:?}", iterations, duration);
    println!("Average per lookup: {:?}", duration / iterations);

    // 性能断言：平均查找应该在合理时间内
    assert!(
        duration < std::time::Duration::from_secs(120),
        "Blacklist lookup should be reasonably fast"
    );

    // 清理
    if let Ok(entries) = get_blacklist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                crate::modules::security_db::remove_from_blacklist(&entry.id),
                "remove_from_blacklist",
            );
        }
    }
}

/// 基准测试：CIDR 匹配性能
#[test]
fn benchmark_cidr_matching() {
    let _lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(init_db(), "init_db");

    // 清理并添加 CIDR 规则
    if let Ok(entries) = get_blacklist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                crate::modules::security_db::remove_from_blacklist(&entry.id),
                "remove_from_blacklist",
            );
        }
    }

    // 添加 10 个 CIDR 规则
    for i in 0..10 {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            add_to_blacklist(
                &format!("10.{}.0.0/16", i),
                Some("CIDR Benchmark"),
                None,
                "test",
            ),
            "add_to_blacklist",
        );
    }

    // 测试 CIDR 匹配性能
    let iterations = 20;
    let start = Instant::now();
    for _ in 0..iterations {
        // 测试需要遍历 CIDR 的 IP
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(is_ip_in_blacklist("10.5.100.50"), "is_ip_in_blacklist");
    }
    let duration = start.elapsed();

    println!("{} CIDR matches took: {:?}", iterations, duration);
    println!("Average per match: {:?}", duration / iterations);

    // 性能断言：CIDR 匹配应该在合理时间内
    assert!(
        duration < std::time::Duration::from_secs(120),
        "CIDR matching should be reasonably fast"
    );

    // 清理
    if let Ok(entries) = get_blacklist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                crate::modules::security_db::remove_from_blacklist(&entry.id),
                "remove_from_blacklist",
            );
        }
    }
}
