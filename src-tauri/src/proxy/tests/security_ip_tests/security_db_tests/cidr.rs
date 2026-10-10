use super::*;

// ============================================================================
// 测试类别 3: CIDR 匹配
// ============================================================================

#[test]
fn test_cidr_matching_basic() {
    let _lock = setup_test();

    // 添加 CIDR 范围到黑名单
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("192.168.1.0/24", Some("Block subnet"), None, "test"),
        "add_to_blacklist",
    );

    // 验证该子网内的 IP 都被阻止
    assert!(
        is_ip_in_blacklist("192.168.1.1").unwrap(),
        "192.168.1.1 should match /24"
    );
    assert!(
        is_ip_in_blacklist("192.168.1.100").unwrap(),
        "192.168.1.100 should match /24"
    );
    assert!(
        is_ip_in_blacklist("192.168.1.254").unwrap(),
        "192.168.1.254 should match /24"
    );

    // 验证子网外的 IP 不被阻止
    assert!(
        !is_ip_in_blacklist("192.168.2.1").unwrap(),
        "192.168.2.1 should not match"
    );
    assert!(
        !is_ip_in_blacklist("10.0.0.1").unwrap(),
        "10.0.0.1 should not match"
    );

    cleanup_test_data();
}

#[test]
fn test_cidr_matching_various_masks() {
    let _lock = setup_test();

    // 测试 /16 掩码
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("10.10.0.0/16", Some("Block /16"), None, "test"),
        "add_to_blacklist",
    );

    assert!(is_ip_in_blacklist("10.10.0.1").unwrap(), "Should match /16");
    assert!(
        is_ip_in_blacklist("10.10.255.255").unwrap(),
        "Should match /16"
    );
    assert!(
        !is_ip_in_blacklist("10.11.0.1").unwrap(),
        "Should not match /16"
    );

    cleanup_test_data();

    // 测试 /32 掩码 (单个 IP)
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("8.8.8.8/32", Some("Block single"), None, "test"),
        "add_to_blacklist",
    );

    assert!(is_ip_in_blacklist("8.8.8.8").unwrap(), "Should match /32");
    assert!(
        !is_ip_in_blacklist("8.8.8.9").unwrap(),
        "Should not match /32"
    );

    cleanup_test_data();
}

#[test]
fn test_cidr_edge_cases() {
    let _lock = setup_test();

    // 测试 /0 (所有 IP) - 边界情况
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("0.0.0.0/0", Some("Block all"), None, "test"),
        "add_to_blacklist",
    );

    assert!(
        is_ip_in_blacklist("1.2.3.4").unwrap(),
        "Everything should match /0"
    );
    assert!(
        is_ip_in_blacklist("255.255.255.255").unwrap(),
        "Everything should match /0"
    );

    cleanup_test_data();

    // 测试 /8 掩码
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("10.0.0.0/8", Some("Block /8"), None, "test"),
        "add_to_blacklist",
    );

    assert!(
        is_ip_in_blacklist("10.255.255.255").unwrap(),
        "Should match /8"
    );
    assert!(
        !is_ip_in_blacklist("11.0.0.0").unwrap(),
        "Should not match /8"
    );

    cleanup_test_data();
}
