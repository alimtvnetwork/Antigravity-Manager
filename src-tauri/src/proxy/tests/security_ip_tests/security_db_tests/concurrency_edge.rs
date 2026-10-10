use super::*;

// ============================================================================
// 测试类别 9: 并发安全性
// ============================================================================

#[test]
fn test_concurrent_access() {
    use std::thread;

    let _lock = setup_test();

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                // 每个线程添加不同的 IP
                let ip = format!("concurrent.test.{}", i);
                // Justification: best-effort call; failure logged without changing control flow
                crate::error::record_ignored(
                    add_to_blacklist(&ip, Some("Concurrent test"), None, "test"),
                    "add_to_blacklist",
                );

                // 验证自己添加的 IP
                is_ip_in_blacklist(&ip).unwrap_or(false)
            })
        })
        .collect();

    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // 所有线程都应该成功
    assert!(
        results.iter().all(|&r| r),
        "All concurrent adds should succeed"
    );

    cleanup_test_data();
}

// ============================================================================
// 测试类别 10: 边界情况和错误处理
// ============================================================================

#[test]
fn test_duplicate_blacklist_entry() {
    let _lock = setup_test();

    // 第一次添加应该成功
    let result1 = add_to_blacklist("duplicate.test.ip", Some("First"), None, "test");
    assert!(result1.is_ok());

    // 第二次添加相同 IP 应该失败 (UNIQUE constraint)
    let result2 = add_to_blacklist("duplicate.test.ip", Some("Second"), None, "test");
    assert!(result2.is_err(), "Duplicate IP should fail");

    cleanup_test_data();
}

#[test]
fn test_empty_ip_pattern() {
    let _lock = setup_test();

    // 空 IP 模式应该仍然可以添加 (取决于业务需求)
    // 这里只测试不会 panic
    let result = add_to_blacklist("", Some("Empty IP"), None, "test");
    // 结果可能成功或失败，但不应该 panic
    // Justification: test asserts no-panic only; either outcome is acceptable, failure is logged.
    crate::error::record_ignored(result, "add_to_blacklist empty ip");

    cleanup_test_data();
}

#[test]
fn test_special_characters_in_reason() {
    let _lock = setup_test();

    // 测试包含特殊字符的原因
    let reason = "Test with 'quotes' and \"double quotes\" and emoji 🚫";
    let result = add_to_blacklist("special.char.test", Some(reason), None, "test");
    assert!(result.is_ok());

    let entry = get_blacklist_entry_for_ip("special.char.test")
        .unwrap()
        .unwrap();
    assert_eq!(entry.reason.as_deref(), Some(reason));

    cleanup_test_data();
}

#[test]
fn test_hit_count_increment() {
    let _lock = setup_test();

    // 添加一个黑名单条目
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("hit.count.test", Some("Count test"), None, "test"),
        "add_to_blacklist",
    );

    // 多次查询应该增加 hit_count
    for _ in 0..5 {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            get_blacklist_entry_for_ip("hit.count.test"),
            "get_blacklist_entry_for_ip",
        );
    }

    // 检查 hit_count
    let blacklist = get_blacklist().unwrap();
    let entry = blacklist.iter().find(|e| e.ip_pattern == "hit.count.test");
    assert!(entry.is_some());
    assert!(
        entry.unwrap().hit_count >= 5,
        "Hit count should be at least 5"
    );

    cleanup_test_data();
}
