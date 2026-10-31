use super::*;

// ============================================================================
// Test category 4: Expiration time processing
// ============================================================================

#[test]
fn test_blacklist_expiration() {
    let _lock = setup_test();

    // Add an expired entry
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "expired.test.ip",
            Some("Already expired"),
            Some(now_timestamp() - 60), // Expired 1 minute ago
            "test",
        ),
        "add_to_blacklist",
    );

    // Expired entries should be automatically purged
    let is_blocked = is_ip_in_blacklist("expired.test.ip");
    // NOTE: Depending on the implementation, expired entries may be cleaned up at query time
    // According to the implementation of security_db.rs, get_blacklist_entry_for_ip will first clean up expired entries
    assert!(!is_blocked.unwrap(), "Expired entry should be cleaned up");

    cleanup_test_data();
}

#[test]
fn test_blacklist_not_yet_expired() {
    let _lock = setup_test();

    // Add an unexpired entry
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "not.expired.ip",
            Some("Will expire later"),
            Some(now_timestamp() + 3600), // Expires in 1 hour
            "test",
        ),
        "add_to_blacklist",
    );

    // Unexpired entries should still be valid
    assert!(is_ip_in_blacklist("not.expired.ip").unwrap());

    cleanup_test_data();
}

#[test]
fn test_permanent_blacklist() {
    let _lock = setup_test();

    // Add permanent ban (no expiry date)
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "permanent.block.ip",
            Some("Permanent ban"),
            None, // No expiration date
            "test",
        ),
        "add_to_blacklist",
    );

    // Permanent bans should always be in effect
    assert!(is_ip_in_blacklist("permanent.block.ip").unwrap());

    cleanup_test_data();
}
