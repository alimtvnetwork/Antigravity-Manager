//! Unit tests for Split-DB Connection Pool Starvation & Checkout Watchdog (Imp-634).

use super::connection_starvation_watchdog::{
    ConnectionLease, ConnectionStarvationWatchdog, StarvationAlert, StarvationSeverity,
    StarvationWatchdogPolicy,
};

#[test]
fn test_default_policy_thresholds() {
    let policy = StarvationWatchdogPolicy::default();
    assert_eq!(policy.warning_threshold_ms, 3000);
    assert_eq!(policy.critical_threshold_ms, 10000);
    assert_eq!(policy.max_active_checkouts, 16);
}

#[test]
fn test_custom_policy_creation() {
    let policy = StarvationWatchdogPolicy::new(1500, 5000, 8);
    assert_eq!(policy.warning_threshold_ms, 1500);
    assert_eq!(policy.critical_threshold_ms, 5000);
    assert_eq!(policy.max_active_checkouts, 8);
}

#[test]
fn test_checkout_under_threshold_returns_empty_alerts() {
    let mut watchdog = ConnectionStarvationWatchdog::default();
    let lease = ConnectionLease::new(
        "lease-01",
        "thread-worker-1",
        1_000_000,
        "SELECT * FROM active_prompts WHERE status = 'Enqueued'",
        "active_prompts.db",
    );
    watchdog.acquire_lease(lease);

    assert_eq!(watchdog.get_active_lease_count(), 1);
    assert!(watchdog.has_active_leases());
    assert!(watchdog.is_lease_active("lease-01"));

    // Evaluated at 1_002_000 (2000ms hold, below warning threshold 3000ms)
    let alerts = watchdog.check_starvation(1_002_000);
    assert!(
        alerts.is_empty(),
        "Under threshold checkouts must not produce alerts"
    );

    // Evaluated right below threshold at 1_002_999 (2999ms hold)
    let boundary_alerts = watchdog.check_starvation(1_002_999);
    assert!(boundary_alerts.is_empty());
}

#[test]
fn test_checkout_exceeding_warning_threshold_triggers_warning_alert() {
    let mut watchdog = ConnectionStarvationWatchdog::default();
    let lease = ConnectionLease::new(
        "lease-warn",
        "thread-worker-2",
        10_000,
        "SELECT id, prompt_content FROM repo_prompts.db",
        "repo_prompts.db",
    );
    watchdog.acquire_lease(lease);

    // Evaluated at 13_500 (3500ms hold, exceeding 3000ms warning threshold)
    let alerts = watchdog.check_starvation(13_500);
    assert_eq!(alerts.len(), 1);

    let alert = &alerts[0];
    assert_eq!(alert.lease_id, "lease-warn");
    assert_eq!(alert.hold_duration_ms, 3500);
    assert_eq!(alert.severity, StarvationSeverity::Warning);
    assert!(alert.is_warning());
    assert!(!alert.is_critical());
    assert_eq!(alert.thread_id, "thread-worker-2");
    assert_eq!(alert.database_name, "repo_prompts.db");
    assert!(alert.query_preview.contains("repo_prompts.db"));
}

#[test]
fn test_checkout_exceeding_critical_threshold_triggers_critical_alert() {
    let mut watchdog = ConnectionStarvationWatchdog::default();
    let lease = ConnectionLease::new(
        "lease-crit",
        "thread-worker-3",
        10_000,
        "BEGIN IMMEDIATE; UPDATE active_prompts SET status = 'Running'",
        "active_prompts.db",
    );
    watchdog.acquire_lease(lease);

    // Evaluated at 21_500 (11500ms hold, exceeding 10000ms critical threshold)
    let alerts = watchdog.check_starvation(21_500);
    assert_eq!(alerts.len(), 1);

    let alert = &alerts[0];
    assert_eq!(alert.lease_id, "lease-crit");
    assert_eq!(alert.hold_duration_ms, 11500);
    assert_eq!(alert.severity, StarvationSeverity::CriticalStarvation);
    assert!(alert.is_critical());
    assert!(!alert.is_warning());
    assert_eq!(alert.thread_id, "thread-worker-3");
    assert_eq!(alert.database_name, "active_prompts.db");
}

#[test]
fn test_active_checkout_release_clears_lease_and_eliminates_alerts() {
    let mut watchdog = ConnectionStarvationWatchdog::default();
    let lease = ConnectionLease::new(
        "lease-transient",
        "thread-worker-4",
        1000,
        "SELECT 1",
        "active_prompts.db",
    );
    watchdog.acquire_lease(lease);

    // Trigger critical alert prior to release
    let pre_release_alerts = watchdog.check_starvation(15_000);
    assert_eq!(pre_release_alerts.len(), 1);

    // Release the lease
    let released = watchdog.release_lease("lease-transient");
    assert!(released, "Active lease should be successfully released");
    assert_eq!(watchdog.get_active_lease_count(), 0);
    assert!(!watchdog.has_active_leases());
    assert!(!watchdog.is_lease_active("lease-transient"));

    // Re-releasing a removed lease should return false
    let re_released = watchdog.release_lease("lease-transient");
    assert!(
        !re_released,
        "Releasing non-existent lease should return false"
    );

    // Post-release check must yield empty alerts
    let post_release_alerts = watchdog.check_starvation(15_000);
    assert!(
        post_release_alerts.is_empty(),
        "Released checkout must eliminate alerts"
    );
}

#[test]
fn test_pool_capacity_saturation_detects_exhaustion() {
    let policy = StarvationWatchdogPolicy::new(3000, 10000, 3);
    let mut watchdog = ConnectionStarvationWatchdog::new(policy);

    assert!(!watchdog.is_pool_exhausted());
    assert_eq!(watchdog.get_active_lease_count(), 0);

    // Add 1st lease
    watchdog.acquire_lease(ConnectionLease::new("l1", "t1", 100, "q1", "db1"));
    assert!(!watchdog.is_pool_exhausted());

    // Add 2nd lease
    watchdog.acquire_lease(ConnectionLease::new("l2", "t2", 100, "q2", "db2"));
    assert!(!watchdog.is_pool_exhausted());

    // Add 3rd lease - reaches capacity threshold (3 of 3)
    watchdog.acquire_lease(ConnectionLease::new("l3", "t3", 100, "q3", "db3"));
    assert!(
        watchdog.is_pool_exhausted(),
        "Pool must report exhausted at capacity limit"
    );
    assert_eq!(watchdog.get_active_lease_count(), 3);

    // Add 4th lease - over capacity
    watchdog.acquire_lease(ConnectionLease::new("l4", "t4", 100, "q4", "db4"));
    assert!(watchdog.is_pool_exhausted());
    assert_eq!(watchdog.get_active_lease_count(), 4);

    // Release leases back below capacity
    watchdog.release_lease("l1");
    watchdog.release_lease("l2");
    assert_eq!(watchdog.get_active_lease_count(), 2);
    assert!(
        !watchdog.is_pool_exhausted(),
        "Pool must recover from exhaustion once below limit"
    );
}

#[test]
fn test_multi_database_concurrent_leases_ordered_by_duration() {
    let mut watchdog = ConnectionStarvationWatchdog::default();

    // Lease 1: 15_000ms hold -> CriticalStarvation
    watchdog.acquire_lease(ConnectionLease::new(
        "lease-db-active",
        "thread-A",
        5_000,
        "UPDATE active_prompts SET status = 'Processing'",
        "active_prompts.db",
    ));

    // Lease 2: 4_000ms hold -> Warning
    watchdog.acquire_lease(ConnectionLease::new(
        "lease-db-repo",
        "thread-B",
        16_000,
        "SELECT * FROM repo_prompts.db WHERE indexed = 1",
        "repo_prompts.db",
    ));

    // Lease 3: 500ms hold -> Nominal (no alert)
    watchdog.acquire_lease(ConnectionLease::new(
        "lease-db-nominal",
        "thread-C",
        19_500,
        "SELECT 1",
        "active_prompts.db",
    ));

    assert_eq!(watchdog.get_active_lease_count(), 3);
    assert_eq!(watchdog.total_leases_tracked, 3);

    // Evaluated at now_ms = 20_000
    let alerts = watchdog.check_starvation(20_000);
    assert_eq!(
        alerts.len(),
        2,
        "Only Warning and Critical leases emit alerts"
    );

    // Worst offender should be first
    assert_eq!(alerts[0].lease_id, "lease-db-active");
    assert_eq!(alerts[0].severity, StarvationSeverity::CriticalStarvation);
    assert_eq!(alerts[0].hold_duration_ms, 15_000);
    assert_eq!(alerts[0].database_name, "active_prompts.db");

    // Second offender should follow
    assert_eq!(alerts[1].lease_id, "lease-db-repo");
    assert_eq!(alerts[1].severity, StarvationSeverity::Warning);
    assert_eq!(alerts[1].hold_duration_ms, 4_000);
    assert_eq!(alerts[1].database_name, "repo_prompts.db");
}

#[test]
fn test_monotonic_total_leases_tracked() {
    let mut watchdog = ConnectionStarvationWatchdog::default();
    assert_eq!(watchdog.total_leases_tracked, 0);

    for i in 1..=5 {
        watchdog.acquire_lease(ConnectionLease::new(
            format!("lease-{i}"),
            format!("thread-{i}"),
            1000 * i,
            "SELECT 1",
            "active_prompts.db",
        ));
        assert_eq!(watchdog.total_leases_tracked, i);
    }

    // Releasing active leases must NOT decrement total_leases_tracked
    watchdog.release_lease("lease-1");
    watchdog.release_lease("lease-2");
    assert_eq!(watchdog.total_leases_tracked, 5);
    assert_eq!(watchdog.get_active_lease_count(), 3);
}

#[test]
fn test_saturating_time_edge_case() {
    let mut watchdog = ConnectionStarvationWatchdog::default();
    let lease = ConnectionLease::new(
        "lease-future",
        "thread-worker",
        50_000,
        "SELECT 1",
        "active_prompts.db",
    );
    watchdog.acquire_lease(lease);

    // now_ms is less than acquired_at_epoch_ms (clock skew scenario)
    let alerts = watchdog.check_starvation(40_000);
    assert!(
        alerts.is_empty(),
        "Clock skew must saturate duration to 0 and emit no alerts"
    );
}

#[test]
fn test_clear_removes_all_active_leases() {
    let mut watchdog = ConnectionStarvationWatchdog::default();
    watchdog.acquire_lease(ConnectionLease::new("l1", "t1", 100, "q1", "db1"));
    watchdog.acquire_lease(ConnectionLease::new("l2", "t2", 200, "q2", "db2"));
    assert_eq!(watchdog.get_active_lease_count(), 2);

    watchdog.clear();
    assert_eq!(watchdog.get_active_lease_count(), 0);
    assert!(!watchdog.has_active_leases());
}

#[test]
fn test_serde_roundtrip_all_structures() {
    // 1. StarvationSeverity
    let severities = [
        StarvationSeverity::Nominal,
        StarvationSeverity::Warning,
        StarvationSeverity::CriticalStarvation,
    ];
    for sev in &severities {
        let serialized = serde_json::to_string(sev).expect("Serialize severity");
        let deserialized: StarvationSeverity =
            serde_json::from_str(&serialized).expect("Deserialize severity");
        assert_eq!(sev, &deserialized);
    }

    // 2. ConnectionLease
    let lease = ConnectionLease::new(
        "lease-test-serde",
        "thread-123",
        1_700_000_000_000,
        "SELECT * FROM test",
        "test.db",
    );
    let lease_json = serde_json::to_string(&lease).expect("Serialize lease");
    let deserialized_lease: ConnectionLease =
        serde_json::from_str(&lease_json).expect("Deserialize lease");
    assert_eq!(lease, deserialized_lease);

    // 3. StarvationAlert
    let alert = StarvationAlert {
        lease_id: "lease-alert-test".to_string(),
        hold_duration_ms: 12_000,
        severity: StarvationSeverity::CriticalStarvation,
        thread_id: "thread-456".to_string(),
        query_preview: "INSERT INTO active_prompts VALUES (1)".to_string(),
        database_name: "active_prompts.db".to_string(),
    };
    let alert_json = serde_json::to_string(&alert).expect("Serialize alert");
    let deserialized_alert: StarvationAlert =
        serde_json::from_str(&alert_json).expect("Deserialize alert");
    assert_eq!(alert, deserialized_alert);

    // 4. StarvationWatchdogPolicy
    let policy = StarvationWatchdogPolicy::new(2000, 8000, 10);
    let policy_json = serde_json::to_string(&policy).expect("Serialize policy");
    let deserialized_policy: StarvationWatchdogPolicy =
        serde_json::from_str(&policy_json).expect("Deserialize policy");
    assert_eq!(policy, deserialized_policy);

    // 5. ConnectionStarvationWatchdog
    let mut watchdog = ConnectionStarvationWatchdog::new(policy);
    watchdog.acquire_lease(lease);
    let watchdog_json = serde_json::to_string(&watchdog).expect("Serialize watchdog");
    let deserialized_watchdog: ConnectionStarvationWatchdog =
        serde_json::from_str(&watchdog_json).expect("Deserialize watchdog");
    assert_eq!(
        watchdog.total_leases_tracked,
        deserialized_watchdog.total_leases_tracked
    );
    assert_eq!(watchdog.policy, deserialized_watchdog.policy);
    assert_eq!(
        watchdog.active_leases.len(),
        deserialized_watchdog.active_leases.len()
    );
}
