use super::*;

// ============================================================================
// Test Category 9: Concurrency Safety
// ============================================================================

#[test]
fn test_concurrent_access() {
    use std::thread;

    let _lock = setup_test();

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                // Add different IP per thread
                let ip = format!("concurrent.test.{}", i);
                // Justification: best-effort call; failure logged without changing control flow
                crate::error::record_ignored(
                    add_to_blacklist(&ip, Some("Concurrent test"), None, "test"),
                    "add_to_blacklist",
                );

                // Verify the IP you added
                is_ip_in_blacklist(&ip).unwrap_or(false)
            })
        })
        .collect();

    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All threads should succeed
    assert!(
        results.iter().all(|&r| r),
        "All concurrent adds should succeed"
    );

    cleanup_test_data();
}

// ============================================================================
// Test Category 10: Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_duplicate_blacklist_entry() {
    let _lock = setup_test();

    // The first addition should be successful
    let result1 = add_to_blacklist("duplicate.test.ip", Some("First"), None, "test");
    assert!(result1.is_ok());

    // Adding the same IP for the second time should fail (UNIQUE constraint)
    let result2 = add_to_blacklist("duplicate.test.ip", Some("Second"), None, "test");
    assert!(result2.is_err(), "Duplicate IP should fail");

    cleanup_test_data();
}

#[test]
fn test_empty_ip_pattern() {
    let _lock = setup_test();

    // Empty IP patterns should still be added (depending on business requirements)
    // Here we only test without panic
    let result = add_to_blacklist("", Some("Empty IP"), None, "test");
    // The result may succeed or fail, but there should be no panic
    // Justification: test asserts no-panic only; either outcome is acceptable, failure is logged.
    crate::error::record_ignored(result, "add_to_blacklist empty ip");

    cleanup_test_data();
}

#[test]
fn test_special_characters_in_reason() {
    let _lock = setup_test();

    // Reason why test contains special characters
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

    // Add a blacklist entry
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("hit.count.test", Some("Count test"), None, "test"),
        "add_to_blacklist",
    );

    // Multiple queries should increase hit_count
    for _ in 0..5 {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            get_blacklist_entry_for_ip("hit.count.test"),
            "get_blacklist_entry_for_ip",
        );
    }

    // Check hit_count
    let blacklist = get_blacklist().unwrap();
    let entry = blacklist.iter().find(|e| e.ip_pattern == "hit.count.test");
    assert!(entry.is_some());
    assert!(
        entry.unwrap().hit_count >= 5,
        "Hit count should be at least 5"
    );

    cleanup_test_data();
}
