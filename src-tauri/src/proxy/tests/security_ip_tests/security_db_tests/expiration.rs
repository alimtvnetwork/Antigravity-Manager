use super::*;

// ============================================================================
// 测试类别 4: 过期时间处理
// ============================================================================

#[test]
fn test_blacklist_expiration() {
    let _lock = setup_test();

    // 添加一个已过期的条目
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "expired.test.ip",
            Some("Already expired"),
            Some(now_timestamp() - 60), // 1分钟前过期
            "test",
        ),
        "add_to_blacklist",
    );

    // 过期条目应该被自动清理
    let is_blocked = is_ip_in_blacklist("expired.test.ip");
    // 注意：取决于实现，过期条目可能在查询时被清理
    // 根据 security_db.rs 的实现，get_blacklist_entry_for_ip 会先清理过期条目
    assert!(!is_blocked.unwrap(), "Expired entry should be cleaned up");

    cleanup_test_data();
}

#[test]
fn test_blacklist_not_yet_expired() {
    let _lock = setup_test();

    // 添加一个未过期的条目
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "not.expired.ip",
            Some("Will expire later"),
            Some(now_timestamp() + 3600), // 1小时后过期
            "test",
        ),
        "add_to_blacklist",
    );

    // 未过期条目应该仍然生效
    assert!(is_ip_in_blacklist("not.expired.ip").unwrap());

    cleanup_test_data();
}

#[test]
fn test_permanent_blacklist() {
    let _lock = setup_test();

    // 添加永久封禁 (无过期时间)
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "permanent.block.ip",
            Some("Permanent ban"),
            None, // 无过期时间
            "test",
        ),
        "add_to_blacklist",
    );

    // 永久封禁应该始终生效
    assert!(is_ip_in_blacklist("permanent.block.ip").unwrap());

    cleanup_test_data();
}
