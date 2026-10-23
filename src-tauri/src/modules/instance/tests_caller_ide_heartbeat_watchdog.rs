//! Unit tests for Caller IDE Vitality Heartbeat & Eviction Watchdog (`Imp-661`).

#[cfg(test)]
mod tests {
    use crate::modules::instance::caller_ide_heartbeat_watchdog::{
        CallerEvictionDecisionType, CallerIdeHeartbeatReport, CallerIdeHeartbeatWatchdog,
        EvictionProtectionReasonType, WatchdogVitalityStateType, DEFAULT_HEARTBEAT_TTL_MS,
        MAX_HEARTBEAT_TTL_MS, MIN_HEARTBEAT_TTL_MS,
    };

    #[test]
    fn test_default_watchdog_uninitialized() {
        let watchdog = CallerIdeHeartbeatWatchdog::new();
        assert_eq!(watchdog.caller_pid(), None);
        assert!(!watchdog.is_caller_pid_alive());
        assert!(!watchdog.should_protect_from_eviction(1234, false));
        let report = watchdog.generate_report_at(100_000);
        assert_eq!(report.caller_pid, None);
        assert!(!report.is_alive);
        assert!(!report.is_eviction_immune);
        assert_eq!(
            report.vitality_state,
            WatchdogVitalityStateType::Uninitialized
        );
        assert_eq!(
            report.decision_type,
            CallerEvictionDecisionType::EvictionPermittedNoCallerTracked
        );
    }

    #[test]
    fn test_watchdog_registration_and_unregistration() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(4321);
        assert_eq!(watchdog.caller_pid(), Some(4321));
        assert!(watchdog.is_heartbeat_fresh(watchdog.last_heartbeat_age_ms(0) + 1));
        watchdog.unregister_caller();
        assert_eq!(watchdog.caller_pid(), None);
        assert!(!watchdog.is_heartbeat_fresh(100_000));
        assert_eq!(watchdog.last_heartbeat_age_ms(100_000), u64::MAX);
    }

    #[test]
    fn test_active_caller_heartbeat_grants_immunity() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        let now_ms = 15_000;
        let is_protected =
            watchdog.should_protect_from_eviction_with_probe(1001, false, now_ms, |_pid| true);
        assert!(is_protected);
        assert!(watchdog.is_heartbeat_fresh(now_ms));
        assert_eq!(watchdog.last_heartbeat_age_ms(now_ms), 5_000);
    }

    #[test]
    fn test_decision_type_is_protected_active_caller() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        let decision =
            watchdog.evaluate_eviction_decision_with_probe(1001, false, 12_000, |_pid| true);
        assert_eq!(decision, CallerEvictionDecisionType::ProtectedActiveCaller);
        assert!(decision.is_protected());
        assert_eq!(decision.as_str(), "protected_active_caller");
    }

    #[test]
    fn test_stale_heartbeat_permits_eviction() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        let now_ms = 10_000 + DEFAULT_HEARTBEAT_TTL_MS + 1_000;
        let is_protected =
            watchdog.should_protect_from_eviction_with_probe(1001, false, now_ms, |_pid| true);
        assert!(!is_protected);
        assert!(!watchdog.is_heartbeat_fresh(now_ms));
        let decision =
            watchdog.evaluate_eviction_decision_with_probe(1001, false, now_ms, |_pid| true);
        assert_eq!(
            decision,
            CallerEvictionDecisionType::EvictionPermittedHeartbeatStale
        );
    }

    #[test]
    fn test_heartbeat_refresh_restores_immunity() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        let stale_time = 50_000;
        assert!(
            !watchdog.should_protect_from_eviction_with_probe(1001, false, stale_time, |_| true)
        );
        watchdog.record_heartbeat_at(51_000);
        let fresh_time = 52_000;
        assert!(watchdog.should_protect_from_eviction_with_probe(1001, false, fresh_time, |_| true));
        assert_eq!(watchdog.last_heartbeat_age_ms(fresh_time), 1_000);
    }

    #[test]
    fn test_forced_eviction_overrides_immunity() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        let is_protected =
            watchdog.should_protect_from_eviction_with_probe(1001, true, 12_000, |_pid| true);
        assert!(!is_protected);
        let decision =
            watchdog.evaluate_eviction_decision_with_probe(1001, true, 12_000, |_pid| true);
        assert_eq!(
            decision,
            CallerEvictionDecisionType::EvictionPermittedForcedOverride
        );
    }

    #[test]
    fn test_different_target_pid_permits_eviction() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        let is_protected =
            watchdog.should_protect_from_eviction_with_probe(2002, false, 12_000, |_pid| true);
        assert!(!is_protected);
        let decision =
            watchdog.evaluate_eviction_decision_with_probe(2002, false, 12_000, |_pid| true);
        assert_eq!(
            decision,
            CallerEvictionDecisionType::EvictionPermittedDifferentTarget
        );
    }

    #[test]
    fn test_dead_caller_pid_permits_eviction() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        let is_protected =
            watchdog.should_protect_from_eviction_with_probe(1001, false, 12_000, |_pid| false);
        assert!(!is_protected);
        let decision =
            watchdog.evaluate_eviction_decision_with_probe(1001, false, 12_000, |_pid| false);
        assert_eq!(
            decision,
            CallerEvictionDecisionType::EvictionPermittedCallerDead
        );
    }

    #[test]
    fn test_disabled_protection_permits_eviction() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(10_000);
        watchdog.set_protection_enabled(false);
        assert!(!watchdog.is_protection_enabled());
        let is_protected =
            watchdog.should_protect_from_eviction_with_probe(1001, false, 12_000, |_pid| true);
        assert!(!is_protected);
    }

    #[test]
    fn test_ttl_clamping_lower_and_upper_bounds() {
        let clamped_low = CallerIdeHeartbeatWatchdog::with_caller_and_ttl(1001, 500);
        assert_eq!(clamped_low.heartbeat_ttl_ms(), MIN_HEARTBEAT_TTL_MS);
        let clamped_high = CallerIdeHeartbeatWatchdog::with_caller_and_ttl(1001, 1_000_000);
        assert_eq!(clamped_high.heartbeat_ttl_ms(), MAX_HEARTBEAT_TTL_MS);
        let normal_ttl = CallerIdeHeartbeatWatchdog::with_caller_and_ttl(1001, 45_000);
        assert_eq!(normal_ttl.heartbeat_ttl_ms(), 45_000);
    }

    #[test]
    fn test_report_generation_for_active_caller() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(20_000);
        let report = watchdog.generate_report_with_probe(25_000, |_pid| true);
        assert_eq!(report.caller_pid, Some(1001));
        assert!(report.is_alive);
        assert!(report.is_eviction_immune);
        assert_eq!(report.last_heartbeat_age_ms, 5_000);
        assert_eq!(
            report.vitality_state,
            WatchdogVitalityStateType::HealthyActive
        );
        assert_eq!(
            report.decision_type,
            CallerEvictionDecisionType::ProtectedActiveCaller
        );
    }

    #[test]
    fn test_report_generation_for_stale_caller() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(20_000);
        let stale_time = 20_000 + DEFAULT_HEARTBEAT_TTL_MS + 5_000;
        let report = watchdog.generate_report_with_probe(stale_time, |_pid| true);
        assert_eq!(report.caller_pid, Some(1001));
        assert!(report.is_alive);
        assert!(!report.is_eviction_immune);
        assert_eq!(
            report.vitality_state,
            WatchdogVitalityStateType::StaleHeartbeat
        );
        assert_eq!(
            report.decision_type,
            CallerEvictionDecisionType::EvictionPermittedHeartbeatStale
        );
    }

    #[test]
    fn test_report_generation_for_dead_caller() {
        let mut watchdog = CallerIdeHeartbeatWatchdog::new();
        watchdog.register_caller(1001);
        watchdog.record_heartbeat_at(20_000);
        let report = watchdog.generate_report_with_probe(25_000, |_pid| false);
        assert_eq!(report.caller_pid, Some(1001));
        assert!(!report.is_alive);
        assert!(!report.is_eviction_immune);
        assert_eq!(
            report.vitality_state,
            WatchdogVitalityStateType::DeadProcess
        );
        assert_eq!(
            report.decision_type,
            CallerEvictionDecisionType::EvictionPermittedCallerDead
        );
    }

    #[test]
    fn test_report_json_serialization_roundtrip() {
        let report = CallerIdeHeartbeatReport::new(
            Some(7777),
            true,
            true,
            1_234,
            EvictionProtectionReasonType::ActiveCallerWithLivePid
                .as_str()
                .to_string(),
            WatchdogVitalityStateType::HealthyActive,
            CallerEvictionDecisionType::ProtectedActiveCaller,
        );
        let json_str = serde_json::to_string(&report).expect("Serialize report");
        let parsed: CallerIdeHeartbeatReport =
            serde_json::from_str(&json_str).expect("Deserialize report");
        assert_eq!(report, parsed);
    }

    #[test]
    fn test_live_current_process_vitality_probe_integration() {
        let current_pid = std::process::id();
        let watchdog = CallerIdeHeartbeatWatchdog::with_caller(current_pid);
        assert!(watchdog.is_caller_pid_alive());
        assert!(watchdog.should_protect_from_eviction(current_pid, false));
        assert!(!watchdog.should_protect_from_eviction(current_pid, true));
    }

    #[test]
    fn test_live_dead_pid_vitality_probe_integration() {
        let current_pid = std::process::id();
        let watchdog = CallerIdeHeartbeatWatchdog::with_caller(current_pid);
        let dead_target_pid = 999_999_999;
        assert!(!watchdog.should_protect_from_eviction(dead_target_pid, false));
    }
}
