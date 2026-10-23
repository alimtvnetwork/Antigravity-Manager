//! Unit tests for Split-DB Lock Contention Timeout Escalator (`Imp-629`).

use super::lock_timeout_escalator::{
    is_busy_error_code, LockEscalationDecision, LockEscalationPolicy, LockEscalationStep,
    LockTimeoutEscalator,
};

#[test]
fn test_step_progression() {
    let policy = LockEscalationPolicy {
        base_timeout_ms: 250,
        max_timeout_ms: 5000,
        max_attempts: 6,
        backoff_multiplier: 2.0,
        jitter_ratio: 0.0,
    };

    // Verify step calculation (250ms -> 500ms -> 1000ms -> 2000ms -> 4000ms clamped to 5000ms)
    let step0 = policy.calculate_step(0);
    assert_eq!(step0.calculated_timeout_ms, 250);
    assert_eq!(step0.attempt_index, 0);
    assert!(!step0.is_terminal_attempt);

    let step1 = policy.calculate_step(1);
    assert_eq!(step1.calculated_timeout_ms, 500);
    assert_eq!(step1.attempt_index, 1);
    assert!(!step1.is_terminal_attempt);

    let step2 = policy.calculate_step(2);
    assert_eq!(step2.calculated_timeout_ms, 1000);
    assert_eq!(step2.attempt_index, 2);
    assert!(!step2.is_terminal_attempt);

    let step3 = policy.calculate_step(3);
    assert_eq!(step3.calculated_timeout_ms, 2000);
    assert_eq!(step3.attempt_index, 3);
    assert!(!step3.is_terminal_attempt);

    let step4 = policy.calculate_step(4);
    assert_eq!(step4.calculated_timeout_ms, 4000);
    assert_eq!(step4.attempt_index, 4);
    assert!(!step4.is_terminal_attempt);

    // 250 * 2^5 = 8000, clamped to 5000ms
    let step5 = policy.calculate_step(5);
    assert_eq!(step5.calculated_timeout_ms, 5000);
    assert_eq!(step5.attempt_index, 5);
    assert!(step5.is_terminal_attempt);

    // Verify step escalation through LockTimeoutEscalator state machine
    let mut escalator = LockTimeoutEscalator::new(policy);
    let expected_delays = [250, 500, 1000, 2000, 4000, 5000];

    for (idx, &expected_ms) in expected_delays.iter().enumerate() {
        match escalator.next_timeout(true) {
            LockEscalationDecision::RetryWithTimeout { timeout_ms, attempt } => {
                assert_eq!(timeout_ms, expected_ms);
                assert_eq!(attempt, (idx + 1) as u32);
            }
            LockEscalationDecision::LockExhaustedDeadlock { .. } => {
                panic!("escalator unexpectedly exhausted at attempt {}", idx + 1);
            }
        }
    }
}

#[test]
fn test_max_attempts_exhaustion_deadlock() {
    let policy = LockEscalationPolicy {
        base_timeout_ms: 100,
        max_timeout_ms: 1000,
        max_attempts: 3,
        backoff_multiplier: 2.0,
        jitter_ratio: 0.0,
    };
    let mut escalator = LockTimeoutEscalator::new(policy);
    assert!(!escalator.is_exhausted);

    // Execute 3 successful retry steps
    for attempt_expected in 1..=3 {
        match escalator.next_timeout(true) {
            LockEscalationDecision::RetryWithTimeout { attempt, .. } => {
                assert_eq!(attempt, attempt_expected);
            }
            _ => panic!("Expected RetryWithTimeout for attempt {}", attempt_expected),
        }
    }

    assert_eq!(escalator.attempt_count, 3);
    assert!(!escalator.is_exhausted);

    // 4th call must trigger deadlock exhaustion
    let decision = escalator.next_timeout(true);
    match decision {
        LockEscalationDecision::LockExhaustedDeadlock {
            total_elapsed_ms,
            attempts_made,
            error_message,
        } => {
            assert_eq!(attempts_made, 3);
            assert_eq!(total_elapsed_ms, 100 + 200 + 400); // 700ms total
            assert!(error_message.contains("SQLite lock contention exhausted"));
            assert!(error_message.contains("3 attempts"));
        }
        _ => panic!("Expected LockExhaustedDeadlock"),
    }
    assert!(escalator.is_exhausted);

    // Subsequent calls stay exhausted
    let decision_again = escalator.next_timeout(true);
    assert!(decision_again.is_exhausted());
}

#[test]
fn test_sqlite_busy_error_code_detection() {
    // Primary SQLite busy and lock error codes
    assert!(is_busy_error_code(5)); // SQLITE_BUSY
    assert!(is_busy_error_code(6)); // SQLITE_LOCKED

    // Extended SQLite busy error codes
    assert!(is_busy_error_code(261)); // SQLITE_BUSY_RECOVERY
    assert!(is_busy_error_code(517)); // SQLITE_BUSY_SNAPSHOT
    assert!(is_busy_error_code(773)); // SQLITE_BUSY_TIMEOUT
    assert!(is_busy_error_code(262)); // SQLITE_LOCKED_SHAREDCACHE
    assert!(is_busy_error_code(518)); // SQLITE_LOCKED_VTAB

    // Non-busy error codes return false
    assert!(!is_busy_error_code(0)); // SQLITE_OK
    assert!(!is_busy_error_code(1)); // SQLITE_ERROR
    assert!(!is_busy_error_code(14)); // SQLITE_CANTOPEN
    assert!(!is_busy_error_code(19)); // SQLITE_CONSTRAINT
    assert!(!is_busy_error_code(101)); // SQLITE_DONE
    assert!(!is_busy_error_code(-1));
}

#[test]
fn test_reset_restores_initial_state() {
    let mut escalator = LockTimeoutEscalator::new(LockEscalationPolicy::default());

    escalator.next_timeout(true);
    escalator.next_timeout(true);
    assert_eq!(escalator.attempt_count, 2);
    assert!(escalator.total_elapsed_ms > 0);

    // Manual reset
    escalator.reset();
    assert_eq!(escalator.attempt_count, 0);
    assert_eq!(escalator.total_elapsed_ms, 0);
    assert!(!escalator.is_exhausted);

    // next_timeout(false) resets state and returns base timeout at attempt 0
    escalator.next_timeout(true);
    assert_eq!(escalator.attempt_count, 1);
    let decision = escalator.next_timeout(false);
    match decision {
        LockEscalationDecision::RetryWithTimeout { timeout_ms, attempt } => {
            assert_eq!(timeout_ms, escalator.policy.base_timeout_ms);
            assert_eq!(attempt, 0);
        }
        _ => panic!("Expected RetryWithTimeout when not busy"),
    }
    assert_eq!(escalator.attempt_count, 0);
    assert_eq!(escalator.total_elapsed_ms, 0);
    assert!(!escalator.is_exhausted);
}

#[test]
fn test_jitter_bounds_validation() {
    let policy = LockEscalationPolicy {
        base_timeout_ms: 500,
        max_timeout_ms: 1200,
        max_attempts: 50,
        backoff_multiplier: 1.5,
        jitter_ratio: 0.5,
    };
    let mut escalator = LockTimeoutEscalator::new(policy.clone());

    for _ in 0..50 {
        match escalator.next_timeout(true) {
            LockEscalationDecision::RetryWithTimeout { timeout_ms, .. } => {
                // Must never exceed max_timeout_ms
                assert!(
                    timeout_ms <= policy.max_timeout_ms,
                    "Timeout {} exceeded max cap {}",
                    timeout_ms,
                    policy.max_timeout_ms
                );
                // Must never underflow or be negative
                assert!(timeout_ms > 0, "Timeout must be greater than zero");
            }
            LockEscalationDecision::LockExhaustedDeadlock { .. } => break,
        }
    }

    // Extreme jitter test (jitter_ratio: 1.0)
    let extreme_policy = LockEscalationPolicy {
        base_timeout_ms: 200,
        max_timeout_ms: 800,
        max_attempts: 30,
        backoff_multiplier: 2.0,
        jitter_ratio: 1.0,
    };
    let mut extreme_escalator = LockTimeoutEscalator::new(extreme_policy.clone());
    for _ in 0..30 {
        if let LockEscalationDecision::RetryWithTimeout { timeout_ms, .. } =
            extreme_escalator.next_timeout(true)
        {
            assert!(timeout_ms <= extreme_policy.max_timeout_ms);
            assert!(timeout_ms <= 800);
        }
    }
}

#[test]
fn test_serde_roundtrip() {
    let policy = LockEscalationPolicy::default();
    let serialized_policy = serde_json::to_string(&policy).expect("serialize policy");
    let deserialized_policy: LockEscalationPolicy =
        serde_json::from_str(&serialized_policy).expect("deserialize policy");
    assert_eq!(policy, deserialized_policy);

    let step = LockEscalationStep {
        attempt_index: 3,
        calculated_timeout_ms: 2000,
        is_terminal_attempt: false,
    };
    let serialized_step = serde_json::to_string(&step).expect("serialize step");
    let deserialized_step: LockEscalationStep =
        serde_json::from_str(&serialized_step).expect("deserialize step");
    assert_eq!(step, deserialized_step);

    let retry_dec = LockEscalationDecision::RetryWithTimeout {
        timeout_ms: 500,
        attempt: 2,
    };
    let ser_retry = serde_json::to_string(&retry_dec).expect("serialize retry");
    let de_retry: LockEscalationDecision =
        serde_json::from_str(&ser_retry).expect("deserialize retry");
    assert_eq!(retry_dec, de_retry);

    let deadlock_dec = LockEscalationDecision::LockExhaustedDeadlock {
        total_elapsed_ms: 7750,
        attempts_made: 5,
        error_message: "Deadlock detected".to_string(),
    };
    let ser_deadlock = serde_json::to_string(&deadlock_dec).expect("serialize deadlock");
    let de_deadlock: LockEscalationDecision =
        serde_json::from_str(&ser_deadlock).expect("deserialize deadlock");
    assert_eq!(deadlock_dec, de_deadlock);

    let escalator = LockTimeoutEscalator::new(policy);
    let ser_esc = serde_json::to_string(&escalator).expect("serialize escalator");
    let de_esc: LockTimeoutEscalator =
        serde_json::from_str(&ser_esc).expect("deserialize escalator");
    assert_eq!(escalator, de_esc);
}

#[test]
fn test_to_app_error_conversion() {
    let retry_dec = LockEscalationDecision::RetryWithTimeout {
        timeout_ms: 250,
        attempt: 1,
    };
    assert!(retry_dec.to_app_error().is_none());

    let deadlock_dec = LockEscalationDecision::LockExhaustedDeadlock {
        total_elapsed_ms: 5000,
        attempts_made: 5,
        error_message: "Contention exhausted".to_string(),
    };
    let app_err = deadlock_dec.to_app_error();
    assert!(app_err.is_some());
    let err_str = app_err.unwrap().to_string();
    assert!(err_str.contains("Contention exhausted"));
}
