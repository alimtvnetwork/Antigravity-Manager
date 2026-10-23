use super::*;

// ============================================================================
// Test Category 2: Basic Operation of IP Blacklist
// ============================================================================

#[test]
fn test_blacklist_add_and_check() {
    let _lock = setup_test();

    // Add IP to blacklist
    let result = add_to_blacklist("192.168.1.100", Some("Test block"), None, "test");
    assert!(
        result.is_ok(),
        "Should add IP to blacklist: {:?}",
        result.err()
    );

    // Verify IP is in blacklist
    let is_blocked = is_ip_in_blacklist("192.168.1.100");
    assert!(is_blocked.is_ok());
    assert!(is_blocked.unwrap(), "IP should be in blacklist");

    // Verify other IPs are not in the blacklist
    let is_other_blocked = is_ip_in_blacklist("192.168.1.101");
    assert!(is_other_blocked.is_ok());
    assert!(
        !is_other_blocked.unwrap(),
        "Other IP should not be in blacklist"
    );

    cleanup_test_data();
}

#[test]
fn test_blacklist_remove() {
    let _lock = setup_test();

    // Add IP
    let entry = add_to_blacklist("10.0.0.5", Some("Temp block"), None, "test").unwrap();

    // Verify existence
    assert!(is_ip_in_blacklist("10.0.0.5").unwrap());

    // Remove
    let remove_result = remove_from_blacklist(&entry.id);
    assert!(remove_result.is_ok());

    // Verification removed
    assert!(!is_ip_in_blacklist("10.0.0.5").unwrap());

    cleanup_test_data();
}

#[test]
fn test_blacklist_get_entry_details() {
    let _lock = setup_test();

    // Add entry with details
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "172.16.0.50",
            Some("Abuse detected"),
            Some(now_timestamp() + 3600), // Expires in 1 hour
            "admin",
        ),
        "add_to_blacklist",
    );

    // Get item details
    let entry_result = get_blacklist_entry_for_ip("172.16.0.50");
    assert!(entry_result.is_ok());

    let entry = entry_result.unwrap();
    assert!(entry.is_some());

    let entry = entry.unwrap();
    assert_eq!(entry.ip_pattern, "172.16.0.50");
    assert_eq!(entry.reason.as_deref(), Some("Abuse detected"));
    assert_eq!(entry.created_by, "admin");
    assert!(entry.expires_at.is_some());

    cleanup_test_data();
}
