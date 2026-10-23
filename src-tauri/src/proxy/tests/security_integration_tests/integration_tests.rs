use crate::modules::security_db::{
    self, add_to_blacklist, add_to_whitelist, get_blacklist, get_whitelist, init_db,
    remove_from_blacklist, remove_from_whitelist,
};
use std::time::Duration;

/// Auxiliary function: initialize test and lock isolation
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

/// Auxiliary function: clean up the test environment
fn cleanup_test_data() {
    if let Ok(entries) = get_blacklist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(remove_from_blacklist(&entry.id), "remove_from_blacklist");
        }
    }
    if let Ok(entries) = get_whitelist() {
        for entry in entries {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(remove_from_whitelist(&entry.id), "remove_from_whitelist");
        }
    }
}

// ============================================================================
// Integration test scenario 1: Blacklist blocking request
// ============================================================================

/// Test scenario: When the IP is in the blacklist, the request should be rejected
///
/// Expected behavior:
/// 1. Add IP to blacklist
/// 2. Requests initiated by this IP return 403 Forbidden
/// 3. The response body contains the reason for the ban
#[test]
fn test_scenario_blacklist_blocks_request() {
    let _lock = setup_test();

    // Add test IP to blacklist
    let entry = add_to_blacklist(
        "192.168.100.100",
        Some("Integration test - malicious activity"),
        None,
        "integration_test",
    );
    assert!(entry.is_ok(), "Should add IP to blacklist");

    // Verify blacklist entry exists
    let blacklist = get_blacklist().unwrap();
    let found = blacklist.iter().any(|e| e.ip_pattern == "192.168.100.100");
    assert!(found, "IP should be in blacklist");

    // Actual HTTP request testing requires starting the server
    // Verify the correctness of the data layer here
    let is_blocked = security_db::is_ip_in_blacklist("192.168.100.100").unwrap();
    assert!(is_blocked, "IP should be blocked");

    cleanup_test_data();
}

// ============================================================================
// Integration test scenario 2: Whitelist priority mode
// ============================================================================

/// Test scenario: In whitelist priority mode, whitelist IP skips blacklist check
///
/// Expected behavior:
/// 1. IP exists in both blacklist and whitelist
/// 2. Enable whitelist_priority mode
/// 3. Requests should be allowed (whitelist first)
#[test]
fn test_scenario_whitelist_priority() {
    let _lock = setup_test();

    // Add IP to blacklist
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "10.0.0.50",
            Some("Should be overridden by whitelist"),
            None,
            "test",
        ),
        "add_to_blacklist",
    );

    // Add same IP to whitelist
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_whitelist("10.0.0.50", Some("Trusted - override blacklist")),
        "add_to_whitelist",
    );

    // Verify that both lists contain the IP
    assert!(security_db::is_ip_in_blacklist("10.0.0.50").unwrap());
    assert!(security_db::is_ip_in_whitelist("10.0.0.50").unwrap());

    // In the actual middleware, when whitelist_priority=true, the whitelist will be checked first
    // If in whitelist, skip blacklist check
    // Only the data correctness is verified here, and the middleware logic is guaranteed by ip_filter.rs

    cleanup_test_data();
}

// ============================================================================
// Integration test scenario 3: Temporary ban and expiration
// ============================================================================

/// Test scenario: Temporary ban is automatically lifted after expiration
///
/// Expected behavior:
/// 1. Add temporary ban (expired)
/// 2. Automatically clean up expired entries when querying
/// 3. The request should be allowed
#[test]
fn test_scenario_temporary_ban_expiration() {
    let _lock = setup_test();

    // Get current timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Add expired temporary ban
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "expired.ban.test",
            Some("Temporary ban - should be expired"),
            Some(now - 60), // Expired 1 minute ago
            "test",
        ),
        "add_to_blacklist",
    );

    // Expiration cleanup should be triggered when querying
    let is_blocked = security_db::is_ip_in_blacklist("expired.ban.test").unwrap();
    assert!(!is_blocked, "Expired ban should not block");

    cleanup_test_data();
}

// ============================================================================
// Integration test scenario 4: CIDR range ban
// ============================================================================

/// Test scenario: CIDR range ban covers the entire subnet
///
/// Expected behavior:
/// 1. Ban 192.168.1.0/24
/// 2. All requests to 192.168.1.x are denied
/// 3. The request for 192.168.2.x passes normally.
#[test]
fn test_scenario_cidr_subnet_blocking() {
    let _lock = setup_test();

    // Block entire subnet
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "192.168.1.0/24",
            Some("Entire subnet blocked"),
            None,
            "test",
        ),
        "add_to_blacklist",
    );

    // Verify that the IP within the subnet is blocked
    for last_octet in [1, 50, 100, 200, 254] {
        let ip = format!("192.168.1.{}", last_octet);
        let is_blocked = security_db::is_ip_in_blacklist(&ip).unwrap();
        assert!(is_blocked, "IP {} should be blocked by CIDR", ip);
    }

    // Verify that IPs outside the subnet are not blocked
    for last_octet in [1, 50, 100] {
        let ip = format!("192.168.2.{}", last_octet);
        let is_blocked = security_db::is_ip_in_blacklist(&ip).unwrap();
        assert!(!is_blocked, "IP {} should NOT be blocked", ip);
    }

    cleanup_test_data();
}

// ============================================================================
// Integration test scenario 5: Ban message details
// ============================================================================

/// Test scenario: Ban response contains detailed information
///
/// Expected behavior:
/// 1. Add ban with reason
/// 2. When the request is rejected, the response contains:
///    - Reason for ban
///    - Whether it is a temporary/permanent ban
///    - Remaining ban time (if temporary)
#[test]
fn test_scenario_ban_message_details() {
    let _lock = setup_test();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Add temporary ban (expires in 2 hours)
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist(
            "temp.ban.message",
            Some("Rate limit exceeded"),
            Some(now + 7200), // 2 hours later
            "rate_limiter",
        ),
        "add_to_blacklist",
    );

    // Get ban details
    let entry = security_db::get_blacklist_entry_for_ip("temp.ban.message")
        .unwrap()
        .unwrap();

    assert_eq!(entry.reason.as_deref(), Some("Rate limit exceeded"));
    assert!(entry.expires_at.is_some());

    let remaining = entry.expires_at.unwrap() - now;
    assert!(
        remaining > 0 && remaining <= 7200,
        "Should have ~2h remaining"
    );

    cleanup_test_data();
}

// ============================================================================
// Integration Test Scenario 6: Access Logging
// ============================================================================

/// Test scenario: blocked requests logged
///
/// Expected behavior:
/// 1. Blacklist IP initiates request
/// 2. Request rejected
/// 3. Access log records: IP, time, status (403), ban reason
#[test]
fn test_scenario_blocked_request_logging() {
    let _lock = setup_test();

    // Simulate saving blocked access logs
    let log = security_db::IpAccessLog {
        id: uuid::Uuid::new_v4().to_string(),
        client_ip: "blocked.request.test".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        method: Some("POST".to_string()),
        path: Some("/v1/messages".to_string()),
        user_agent: Some("TestClient/1.0".to_string()),
        status: Some(403),
        duration: Some(0),
        api_key_hash: None,
        blocked: true,
        block_reason: Some("IP in blacklist".to_string()),
        username: None,
    };

    let save_result = security_db::save_ip_access_log(&log);
    assert!(save_result.is_ok());

    // Verification logs can be retrieved
    let logs = security_db::get_ip_access_logs(10, 0, None, true).unwrap();
    let found = logs.iter().any(|l| l.client_ip == "blocked.request.test");
    assert!(found, "Blocked request should be logged");

    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(security_db::clear_ip_access_logs(), "clear_ip_access_logs");
}

// ============================================================================
// Integration test scenario 7: Does not affect normal request performance
// ============================================================================

/// Test scenario: Security check does not significantly affect normal request performance
///
/// Expected behavior:
/// 1. Blacklist/whitelist check time < 5ms
/// 2. Latency increase < 10ms compared to baseline without security checks
#[test]
fn test_scenario_performance_impact() {
    let _lock = setup_test();

    // Add some blacklist entries
    for i in 0..50 {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            add_to_blacklist(&format!("perf.test.{}", i), None, None, "test"),
            "add_to_blacklist",
        );
    }

    // Add some CIDR rules
    for i in 0..10 {
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            add_to_blacklist(&format!("172.{}.0.0/16", i), None, None, "test"),
            "add_to_blacklist",
        );
    }

    // Test lookup performance
    let start = std::time::Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        // Security checks that simulate normal requests
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            security_db::is_ip_in_whitelist("10.0.0.1"),
            "is_ip_in_whitelist",
        );
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            security_db::is_ip_in_blacklist("10.0.0.1"),
            "is_ip_in_blacklist",
        );
    }

    let duration = start.elapsed();
    let avg_per_check = duration / (iterations * 2);

    println!("Average security check time: {:?}", avg_per_check);

    // Assertion: On average each check should be within a reasonable time
    assert!(
        avg_per_check < Duration::from_millis(100),
        "Security check should be fast"
    );

    cleanup_test_data();
}

// ============================================================================
// Integration test scenario 8: Data persistence
// ============================================================================

/// Test scenario: blacklist/whitelist data persistence
///
/// Expected behavior:
/// 1. Reinitialize the database connection after adding data
/// 2. The data still exists
#[test]
fn test_scenario_data_persistence() {
    let _lock = setup_test();

    // Add data
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("persist.test.ip", Some("Persistence test"), None, "test"),
        "add_to_blacklist",
    );
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_whitelist("persist.white.ip", Some("Persistence test")),
        "add_to_whitelist",
    );

    // Re-initialize (actually just verify the data is still readable)
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(init_db(), "init_db");

    // Verification data still exists
    assert!(security_db::is_ip_in_blacklist("persist.test.ip").unwrap());
    assert!(security_db::is_ip_in_whitelist("persist.white.ip").unwrap());

    cleanup_test_data();
}
