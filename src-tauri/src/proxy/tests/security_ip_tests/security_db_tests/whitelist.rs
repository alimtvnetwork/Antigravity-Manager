use super::*;

// ============================================================================
// 测试类别 5: IP 白名单
// ============================================================================

#[test]
fn test_whitelist_add_and_check() {
    let _lock = setup_test();

    // 添加 IP 到白名单
    let result = add_to_whitelist("10.0.0.1", Some("Trusted server"));
    assert!(result.is_ok());

    // 验证 IP 在白名单中
    assert!(is_ip_in_whitelist("10.0.0.1").unwrap());
    assert!(!is_ip_in_whitelist("10.0.0.2").unwrap());

    cleanup_test_data();
}

#[test]
fn test_whitelist_cidr() {
    let _lock = setup_test();

    // 添加 CIDR 范围到白名单
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_whitelist("192.168.0.0/16", Some("Internal network")),
        "add_to_whitelist",
    );

    // 验证子网内的 IP 都被允许
    assert!(is_ip_in_whitelist("192.168.1.1").unwrap());
    assert!(is_ip_in_whitelist("192.168.255.255").unwrap());

    // 验证子网外的 IP 不在白名单
    assert!(!is_ip_in_whitelist("10.0.0.1").unwrap());

    cleanup_test_data();
}
