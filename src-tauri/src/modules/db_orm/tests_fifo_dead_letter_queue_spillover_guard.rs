//! Unit tests for Strict FIFO Dead-Letter Queue Spillover Guard (`Imp-687`).

use super::fifo_dead_letter_queue_spillover_guard::*;

#[test]
fn test_validate_spillover_query_valid_canonical() {
    let queries = [
        "SELECT * FROM active_prompts WHERE retry_count >= 5 ORDER BY created_at ASC, id ASC",
        "SELECT id, prompt_text FROM prompt_backups ORDER BY created_at ASC, id ASC LIMIT 50",
        "SELECT * FROM repo_prompts ORDER BY `created_at` ASC, `id` ASC",
        "SELECT * FROM active_prompts ORDER BY \"created_at\" ASC, \"id\" ASC",
        "SELECT * FROM active_prompts ORDER BY t.created_at ASC, t.id ASC",
        "SELECT * FROM active_prompts WHERE attempts > 3 ORDER BY created_at ASC, id ASC; ",
    ];

    for query in &queries {
        let result = validate_spillover_query(query);
        assert!(
            result.is_ok(),
            "Expected query to be valid FIFO: '{query}', got: {result:?}"
        );
    }
}

#[test]
fn test_validate_spillover_query_inverted_lifo() {
    let lifo_queries = [
        "SELECT * FROM active_prompts ORDER BY updated_at DESC",
        "SELECT * FROM active_prompts ORDER BY created_at DESC, id ASC",
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id DESC",
        "SELECT * FROM prompt_backups ORDER BY id DESC",
        "SELECT * FROM repo_prompts ORDER BY id ASC, created_at ASC",
    ];

    for query in &lifo_queries {
        let result = validate_spillover_query(query);
        assert_eq!(
            result,
            Err(SpilloverGuardViolation::InvertedSpilloverOrdering),
            "Expected InvertedSpilloverOrdering for query: '{query}'"
        );
    }
}

#[test]
fn test_validate_spillover_query_missing_original_timestamp() {
    let missing_timestamp_queries = [
        "SELECT * FROM active_prompts WHERE retry_count >= 5",
        "SELECT * FROM prompt_backups ORDER BY status ASC",
        "SELECT * FROM repo_prompts ORDER BY updated_at ASC",
        "",
    ];

    for query in &missing_timestamp_queries {
        let result = validate_spillover_query(query);
        assert_eq!(
            result,
            Err(SpilloverGuardViolation::MissingOriginalTimestamp),
            "Expected MissingOriginalTimestamp for query: '{query}'"
        );
    }
}

#[test]
fn test_validate_spillover_query_missing_id_tie_breaker() {
    let missing_tiebreaker_queries = [
        "SELECT * FROM active_prompts ORDER BY created_at ASC",
        "SELECT * FROM prompt_backups ORDER BY created_at ASC, status ASC",
        "SELECT * FROM repo_prompts ORDER BY created_at ASC, id", // Missing explicit ASC
    ];

    for query in &missing_tiebreaker_queries {
        let result = validate_spillover_query(query);
        assert_eq!(
            result,
            Err(SpilloverGuardViolation::MissingIdTieBreaker),
            "Expected MissingIdTieBreaker for query: '{query}'"
        );
    }
}

#[test]
fn test_build_spillover_drain_query() {
    let query_with_limit = build_spillover_drain_query("active_prompts", 3, Some(100));
    assert!(query_with_limit.contains("WHERE retry_count >= 3"));
    assert!(query_with_limit.contains("ORDER BY created_at ASC, id ASC"));
    assert!(query_with_limit.contains("LIMIT 100"));
    assert!(validate_spillover_query(&query_with_limit).is_ok());

    let query_without_limit = build_spillover_drain_query("prompt_backups", 5, None);
    assert!(query_without_limit.contains("WHERE retry_count >= 5"));
    assert!(query_without_limit.contains("ORDER BY created_at ASC, id ASC"));
    assert!(!query_without_limit.contains("LIMIT"));
    assert!(validate_spillover_query(&query_without_limit).is_ok());
}

#[test]
fn test_record_dead_letter_spillover_compliant() {
    let items = vec![
        DeadLetterPromptItem::new(
            101,
            "2026-10-23T10:00:00Z",
            "Prompt A",
            DeadLetterReason::RetryLimitExceeded { attempts: 5, max_retries: 5 },
            1_700_000_000_000,
        ),
        DeadLetterPromptItem::new(
            102,
            "2026-10-23T10:05:00Z",
            "Prompt B",
            DeadLetterReason::UnrecoverableSyntaxError("Invalid JSON".to_string()),
            1_700_000_000_050,
        ),
        DeadLetterPromptItem::new(
            103,
            "2026-10-23T10:10:00Z",
            "Prompt C",
            DeadLetterReason::UpstreamPermanentRejection("Safety 400".to_string()),
            1_700_000_000_100,
        ),
    ];

    let report = record_dead_letter_spillover(&items);
    assert!(report.is_compliant());
    assert!(!report.has_violations());
    assert_eq!(report.total_spilled_over(), 3);
    assert_eq!(report.quarantined_ids(), vec![101, 102, 103]);
    assert_eq!(report.items().len(), 3);
}

#[test]
fn test_record_dead_letter_spillover_tie_breaker_compliant() {
    let items = vec![
        DeadLetterPromptItem::new(
            101,
            "1700000000",
            "Prompt 1",
            DeadLetterReason::RetryLimitExceeded { attempts: 3, max_retries: 3 },
            1_700_000_000,
        ),
        DeadLetterPromptItem::new(
            102,
            "1700000000", // Identical timestamp, higher ID (compliant)
            "Prompt 2",
            DeadLetterReason::RetryLimitExceeded { attempts: 3, max_retries: 3 },
            1_700_000_000,
        ),
        DeadLetterPromptItem::new(
            105,
            "1700000000", // Identical timestamp, strictly higher ID (compliant)
            "Prompt 3",
            DeadLetterReason::RetryLimitExceeded { attempts: 3, max_retries: 3 },
            1_700_000_000,
        ),
    ];

    let report = record_dead_letter_spillover(&items);
    assert!(report.is_compliant());
    assert!(report.violations().is_empty());
    assert_eq!(report.total_spilled_over(), 3);
}

#[test]
fn test_record_dead_letter_spillover_inverted_timestamps() {
    let items = vec![
        DeadLetterPromptItem::new(
            101,
            "2026-10-23T11:00:00Z",
            "Later prompt",
            DeadLetterReason::RetryLimitExceeded { attempts: 5, max_retries: 5 },
            1_000,
        ),
        DeadLetterPromptItem::new(
            102,
            "2026-10-23T10:00:00Z", // Regressed timestamp!
            "Earlier prompt",
            DeadLetterReason::RetryLimitExceeded { attempts: 5, max_retries: 5 },
            1_010,
        ),
    ];

    let report = record_dead_letter_spillover(&items);
    assert!(!report.is_compliant());
    assert!(report.has_violations());
    assert!(report
        .violations()
        .contains(&SpilloverGuardViolation::InvertedSpilloverOrdering));
}

#[test]
fn test_record_dead_letter_spillover_tie_breaker_inversion() {
    let items = vec![
        DeadLetterPromptItem::new(
            205,
            "1700000000",
            "Prompt X",
            DeadLetterReason::RetryLimitExceeded { attempts: 3, max_retries: 3 },
            1_000,
        ),
        DeadLetterPromptItem::new(
            204, // Lower ID on identical timestamp (inversion!)
            "1700000000",
            "Prompt Y",
            DeadLetterReason::RetryLimitExceeded { attempts: 3, max_retries: 3 },
            1_000,
        ),
    ];

    let report = record_dead_letter_spillover(&items);
    assert!(!report.is_compliant());
    assert!(report
        .violations()
        .contains(&SpilloverGuardViolation::MissingIdTieBreaker));
}

#[test]
fn test_record_dead_letter_spillover_duplicate_id_inversion() {
    let items = vec![
        DeadLetterPromptItem::new(
            300,
            "1700000000",
            "Prompt A",
            DeadLetterReason::RetryLimitExceeded { attempts: 3, max_retries: 3 },
            1_000,
        ),
        DeadLetterPromptItem::new(
            300, // Duplicate ID on same timestamp
            "1700000000",
            "Prompt B",
            DeadLetterReason::RetryLimitExceeded { attempts: 3, max_retries: 3 },
            1_000,
        ),
    ];

    let report = record_dead_letter_spillover(&items);
    assert!(!report.is_compliant());
    assert!(report
        .violations()
        .contains(&SpilloverGuardViolation::MissingIdTieBreaker));
}

#[test]
fn test_record_dead_letter_spillover_missing_timestamp() {
    let items = vec![
        DeadLetterPromptItem::new(
            401,
            "   ", // Blank timestamp
            "Prompt with missing timestamp",
            DeadLetterReason::UnrecoverableSyntaxError("Corrupted payload".to_string()),
            1_000,
        ),
    ];

    let report = record_dead_letter_spillover(&items);
    assert!(!report.is_compliant());
    assert!(report
        .violations()
        .contains(&SpilloverGuardViolation::MissingOriginalTimestamp));
}

#[test]
fn test_dead_letter_reasons_and_formatting() {
    let r1 = DeadLetterReason::RetryLimitExceeded { attempts: 4, max_retries: 3 };
    let r2 = DeadLetterReason::UnrecoverableSyntaxError("Unexpected token".to_string());
    let r3 = DeadLetterReason::UpstreamPermanentRejection("Model decommissioned".to_string());

    assert!(r1.is_retry_limit_exceeded());
    assert!(!r1.is_unrecoverable_syntax_error());

    assert!(r2.is_unrecoverable_syntax_error());
    assert!(!r2.is_upstream_permanent_rejection());

    assert!(r3.is_upstream_permanent_rejection());
    assert!(!r3.is_retry_limit_exceeded());

    assert_eq!(r1.to_string(), "Retry limit exceeded (4/3)");
    assert_eq!(r2.to_string(), "Unrecoverable syntax error: Unexpected token");
    assert_eq!(r3.to_string(), "Upstream permanent rejection: Model decommissioned");
}

#[test]
fn test_empty_batch_spillover() {
    let report = record_dead_letter_spillover(&[]);
    assert!(report.is_compliant());
    assert_eq!(report.total_spilled_over(), 0);
    assert!(report.violations().is_empty());
    assert!(report.items().is_empty());
}

#[test]
fn test_json_serialization_roundtrip() {
    let item = DeadLetterPromptItem::new(
        999,
        "2026-10-23T15:00:00Z",
        "Test Serialization",
        DeadLetterReason::RetryLimitExceeded { attempts: 5, max_retries: 5 },
        1_700_000_123_456,
    );

    let json = serde_json::to_string(&item).expect("Failed to serialize DeadLetterPromptItem");
    let deserialized: DeadLetterPromptItem =
        serde_json::from_str(&json).expect("Failed to deserialize DeadLetterPromptItem");
    assert_eq!(item, deserialized);
}

#[test]
fn test_violation_display_and_error() {
    let v1 = SpilloverGuardViolation::InvertedSpilloverOrdering;
    let v2 = SpilloverGuardViolation::MissingOriginalTimestamp;
    let v3 = SpilloverGuardViolation::MissingIdTieBreaker;

    assert!(v1.to_string().contains("Inverted spillover ordering"));
    assert!(v2.to_string().contains("Missing original timestamp"));
    assert!(v3.to_string().contains("Missing mandatory secondary tie-breaker"));

    let err: &dyn std::error::Error = &v1;
    assert_eq!(format!("{err}"), v1.to_string());
}
