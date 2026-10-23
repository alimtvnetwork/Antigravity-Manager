use super::*;

// ============================================================================
// Test Category 5: IP Whitelist
// ============================================================================

#[test]
fn test_whitelist_add_and_check() {
    let _lock = setup_test();

    // Add IP to whitelist
    let result = add_to_whitelist("10.0.0.1", Some("Trusted server"));
    assert!(result.is_ok());

    // Verify IP is in whitelist
    assert!(is_ip_in_whitelist("10.0.0.1").unwrap());
    assert!(!is_ip_in_whitelist("10.0.0.2").unwrap());

    cleanup_test_data();
}

#[test]
fn test_whitelist_cidr() {
    let _lock = setup_test();

    // Add CIDR range to whitelist
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_whitelist("192.168.0.0/16", Some("Internal network")),
        "add_to_whitelist",
    );

    // Verify that all IPs in the subnet are allowed
    assert!(is_ip_in_whitelist("192.168.1.1").unwrap());
    assert!(is_ip_in_whitelist("192.168.255.255").unwrap());

    // Verify that IPs outside the subnet are not whitelisted
    assert!(!is_ip_in_whitelist("10.0.0.1").unwrap());

    cleanup_test_data();
}
