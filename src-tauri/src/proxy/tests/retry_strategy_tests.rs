//! Test all branches of determine_retry_strategy and should_rotate_account,
//! Focus on covering 404 retry and account rotation logic.

use crate::proxy::handlers::common::{
    determine_retry_strategy, should_rotate_account, RetryStrategy,
};
use std::time::Duration;

// ===== determine_retry_strategy =====

#[test]
fn test_retry_strategy_404() {
    let strategy = determine_retry_strategy(404, "", false);
    assert!(
        matches!(strategy, RetryStrategy::NoRetry),
        "Expected NoRetry for 404 model not found, got {:?}",
        strategy
    );
}

#[test]
fn test_retry_strategy_429_no_delay() {
    let strategy = determine_retry_strategy(429, "rate limited", false);
    assert!(
        matches!(strategy, RetryStrategy::GraceRetry(d) if d == Duration::from_millis(3000)),
        "Expected GraceRetry(3s), got {:?}",
        strategy
    );
}

#[test]
fn test_retry_strategy_503() {
    let strategy = determine_retry_strategy(503, "", false);
    assert!(
        matches!(
            strategy,
            RetryStrategy::ExponentialBackoff {
                base_ms: 5000,
                max_ms: 30000
            }
        ),
        "Expected ExponentialBackoff {{ base_ms: 5000, max_ms: 30000 }}, got {:?}",
        strategy
    );
}

#[test]
fn test_retry_strategy_529() {
    let strategy = determine_retry_strategy(529, "", false);
    assert!(
        matches!(
            strategy,
            RetryStrategy::ExponentialBackoff {
                base_ms: 5000,
                max_ms: 30000
            }
        ),
        "Expected ExponentialBackoff {{ base_ms: 5000, max_ms: 30000 }}, got {:?}",
        strategy
    );
}

#[test]
fn test_retry_strategy_500() {
    let strategy = determine_retry_strategy(500, "", false);
    assert!(
        matches!(strategy, RetryStrategy::LinearBackoff { base_ms: 3000 }),
        "Expected LinearBackoff {{ base_ms: 3000 }}, got {:?}",
        strategy
    );
}

#[test]
fn test_retry_strategy_401_403() {
    for status in [401, 403] {
        let strategy = determine_retry_strategy(status, "", false);
        match strategy {
            RetryStrategy::FixedDelay(d) => assert_eq!(d, Duration::from_millis(200)),
            other => panic!("Expected FixedDelay(200ms) for {}, got {:?}", status, other),
        }
    }
}

#[test]
fn test_retry_strategy_other() {
    for status in [200, 201, 301, 418, 502] {
        let strategy = determine_retry_strategy(status, "", false);
        assert!(
            matches!(strategy, RetryStrategy::NoRetry),
            "Expected NoRetry for {}, got {:?}",
            status,
            strategy
        );
    }
}

#[test]
fn test_retry_strategy_400_thinking_signature() {
    let signatures = [
        "Invalid `signature` for thinking",
        "Error with thinking.signature",
        "thinking.thinking block failed",
        "Corrupted thought signature detected",
    ];
    for sig in signatures {
        let strategy = determine_retry_strategy(400, sig, false);
        match strategy {
            RetryStrategy::FixedDelay(d) => assert_eq!(d, Duration::from_millis(200)),
            other => panic!(
                "Expected FixedDelay(200ms) for 400 + '{}', got {:?}",
                sig, other
            ),
        }
    }
}

#[test]
fn test_retry_strategy_400_no_signature() {
    let strategy = determine_retry_strategy(400, "bad request", false);
    assert!(
        matches!(strategy, RetryStrategy::NoRetry),
        "Expected NoRetry for 400 without signature, got {:?}",
        strategy
    );
}

// ===== should_rotate_account =====

#[test]
fn test_rotate_account_true_cases() {
    for status in [429, 401, 403, 404, 500, 503, 529] {
        assert!(
            should_rotate_account(status, None),
            "Expected should_rotate_account({}) == true",
            status
        );
    }
}

#[test]
fn test_rotate_account_false_cases() {
    for status in [400, 200, 502] {
        assert!(
            !should_rotate_account(status, None),
            "Expected should_rotate_account({}) == false",
            status
        );
    }
}

// ===== Adaptive multi-round and single account waiting for comprehensive regression single test (Refs #3485) =====

#[test]
fn test_calculate_max_retry_attempts_adaptive() {
    use crate::proxy::handlers::common::calculate_max_retry_attempts;
    assert_eq!(calculate_max_retry_attempts(0), 3);
    assert_eq!(calculate_max_retry_attempts(1), 3);
    assert_eq!(calculate_max_retry_attempts(2), 4);
    assert_eq!(calculate_max_retry_attempts(3), 6);
    assert_eq!(calculate_max_retry_attempts(5), 10);
    assert_eq!(calculate_max_retry_attempts(8), 12);
    assert_eq!(calculate_max_retry_attempts(20), 12);
}

#[test]
fn test_adaptive_retry_single_account_waits_quota_reset_delay() {
    use crate::proxy::handlers::common::determine_retry_strategy_adaptive;
    let err_json = r#"{"error":{"message":"Resource has been exhausted (e.g. check quota).","details":[{"@type":"type.googleapis.com/google.rpc.RetryInfo","quotaResetDelay":"4s"}]}}"#;

    // Single account (pool_size = 1): Never 50ms lightning flash death, must wait for quotaResetDelay in place GraceRetry (4s + 200ms = 4200ms)
    let strategy = determine_retry_strategy_adaptive(429, err_json, None, false, true, 0, 1);
    match strategy {
        RetryStrategy::GraceRetry(d) => assert_eq!(d, Duration::from_millis(4200)),
        other => panic!("Expected GraceRetry(4200ms), got {:?}", other),
    }
}

#[test]
fn test_adaptive_retry_multi_account_round_1_fast_rotates() {
    use crate::proxy::handlers::common::determine_retry_strategy_adaptive;
    let err_json = r#"{"error":{"message":"Resource has been exhausted","details":[{"quotaResetDelay":"4s"}]}}"#;

    // Multiple accounts Round 1 (attempt = 0, pool_size = 3): Lightning rotation (50ms), giving priority to other healthy accounts in the pool
    let strategy = determine_retry_strategy_adaptive(429, err_json, None, false, true, 0, 3);
    match strategy {
        RetryStrategy::FixedDelay(d) => assert_eq!(d, Duration::from_millis(50)),
        other => panic!("Expected FixedDelay(50ms), got {:?}", other),
    }
}

#[test]
fn test_adaptive_retry_multi_account_round_2_small_gap_micro_waits() {
    use crate::proxy::handlers::common::determine_retry_strategy_adaptive;
    let err_json = r#"{"error":{"message":"Resource has been exhausted","details":[{"quotaResetDelay":"3s"}]}}"#;

    // Multi-account Round 2 (attempt = 3, pool_size = 3): The whole pool has been tried once. If a small gap is encountered (3s <= 5s), wait and try again in place.
    let strategy = determine_retry_strategy_adaptive(429, err_json, None, false, true, 3, 3);
    match strategy {
        RetryStrategy::GraceRetry(d) => assert_eq!(d, Duration::from_millis(3200)),
        other => panic!("Expected GraceRetry(3200ms), got {:?}", other),
    }
}

#[test]
fn test_adaptive_retry_multi_account_round_2_large_gap_rotates_if_more_than_two_accounts() {
    use crate::proxy::handlers::common::determine_retry_strategy_adaptive;
    let err_json = r#"{"error":{"message":"Resource has been exhausted","details":[{"quotaResetDelay":"15s"}]}}"#;

    // Multiple accounts Round 2 (attempt = 3, pool_size = 3): The delay is 15s (> 5s) and there are other accounts, continue the lightning rotation to find the recovered account
    let strategy = determine_retry_strategy_adaptive(429, err_json, None, false, true, 3, 3);
    match strategy {
        RetryStrategy::FixedDelay(d) => assert_eq!(d, Duration::from_millis(50)),
        other => panic!("Expected FixedDelay(50ms), got {:?}", other),
    }
}
