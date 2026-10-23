//! Unit tests for FIFO Prompt Restore & Queue Dispatch Validator (`Imp-623`).

use super::fifo_prompt_restore_validator::{
    sanitize_or_enforce_fifo_clause, validate_prompt_restore_sql, FifoOrderingViolation,
    FifoValidationReport, PromptRestoreQueryKind, MANDATORY_FIFO_ORDER_CLAUSE,
};

#[test]
fn test_valid_sql_canonical_fifo_passes() {
    let sql = "SELECT id, prompt_text FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 50;";
    let report = validate_prompt_restore_sql(sql, PromptRestoreQueryKind::ActivePromptsRestore);

    assert!(report.is_valid);
    assert_eq!(report.query_kind, PromptRestoreQueryKind::ActivePromptsRestore);
    assert_eq!(report.normalized_order_clause, MANDATORY_FIFO_ORDER_CLAUSE);
    assert!(report.violation.is_none());
}

#[test]
fn test_valid_sql_with_table_alias_and_spacing() {
    let sql = "SELECT p.id FROM prompt_backups p ORDER BY  p.created_at  ASC ,  p.id  ASC";
    let report = validate_prompt_restore_sql(sql, PromptRestoreQueryKind::PromptBackupsRestore);

    assert!(report.is_valid);
    assert_eq!(report.normalized_order_clause, MANDATORY_FIFO_ORDER_CLAUSE);
    assert!(report.violation.is_none());
}

#[test]
fn test_inverted_lifo_created_at_desc_rejected() {
    let bad_sql = "SELECT * FROM prompt_backups ORDER BY created_at DESC LIMIT 10";
    let report = validate_prompt_restore_sql(bad_sql, PromptRestoreQueryKind::PromptBackupsRestore);

    assert!(!report.is_valid);
    match report.violation {
        Some(FifoOrderingViolation::ForbiddenLifoDetected { query_snippet, forbidden_clause }) => {
            assert!(
                forbidden_clause.contains("created_at DESC")
                    || forbidden_clause.contains("ORDER BY created_at DESC")
            );
            assert!(query_snippet.contains("SELECT * FROM prompt_backups"));
        }
        other => panic!("Expected ForbiddenLifoDetected, got: {:?}", other),
    }
}

#[test]
fn test_inverted_lifo_updated_at_desc_rejected() {
    let bad_sql = "SELECT id, prompt_text FROM active_prompts ORDER BY updated_at DESC LIMIT 20";
    let report = validate_prompt_restore_sql(bad_sql, PromptRestoreQueryKind::ActivePromptsRestore);

    assert!(!report.is_valid);
    match report.violation {
        Some(FifoOrderingViolation::ForbiddenLifoDetected { forbidden_clause, .. }) => {
            assert!(
                forbidden_clause.contains("updated_at DESC")
                    || forbidden_clause.contains("ORDER BY updated_at DESC")
            );
        }
        other => panic!("Expected ForbiddenLifoDetected, got: {:?}", other),
    }
}

#[test]
fn test_inverted_lifo_rowid_desc_rejected() {
    let bad_sql = "SELECT * FROM repo_prompts ORDER BY rowid DESC";
    let report = validate_prompt_restore_sql(bad_sql, PromptRestoreQueryKind::RepoPromptsDispatch);

    assert!(!report.is_valid);
    assert!(matches!(
        report.violation,
        Some(FifoOrderingViolation::ForbiddenLifoDetected { .. })
    ));
}

#[test]
fn test_missing_id_tiebreaker_rejected() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC LIMIT 10";
    let report = validate_prompt_restore_sql(sql, PromptRestoreQueryKind::ActivePromptsRestore);

    assert!(!report.is_valid);
    match report.violation {
        Some(FifoOrderingViolation::MissingIdTieBreaker { query_snippet }) => {
            assert!(query_snippet.contains("ORDER BY created_at ASC"));
        }
        other => panic!("Expected MissingIdTieBreaker, got: {:?}", other),
    }
}

#[test]
fn test_missing_id_tiebreaker_with_other_column() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC, priority ASC";
    let report = validate_prompt_restore_sql(sql, PromptRestoreQueryKind::ActivePromptsRestore);

    assert!(!report.is_valid);
    assert!(matches!(
        report.violation,
        Some(FifoOrderingViolation::MissingIdTieBreaker { .. })
    ));
}

#[test]
fn test_missing_created_at_sort_no_order_by() {
    let sql = "SELECT * FROM active_prompts WHERE status = 'Enqueued'";
    let report = validate_prompt_restore_sql(sql, PromptRestoreQueryKind::GeneralPromptQuery);

    assert!(!report.is_valid);
    match report.violation {
        Some(FifoOrderingViolation::MissingCreatedAtSort { query_snippet }) => {
            assert!(query_snippet.contains("WHERE status = 'Enqueued'"));
        }
        other => panic!("Expected MissingCreatedAtSort, got: {:?}", other),
    }
}

#[test]
fn test_missing_created_at_sort_wrong_primary_sort() {
    let sql = "SELECT * FROM active_prompts ORDER BY id ASC, created_at ASC";
    let report = validate_prompt_restore_sql(sql, PromptRestoreQueryKind::ActivePromptsRestore);

    assert!(!report.is_valid);
    assert!(matches!(
        report.violation,
        Some(FifoOrderingViolation::MissingCreatedAtSort { .. })
    ));
}

#[test]
fn test_sanitize_or_enforce_no_order_by() {
    let query_simple = "SELECT * FROM active_prompts";
    let sanitized_simple = sanitize_or_enforce_fifo_clause(query_simple);
    assert_eq!(
        sanitized_simple,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC"
    );

    let query_where = "SELECT * FROM active_prompts WHERE status = 'Enqueued'";
    let sanitized_where = sanitize_or_enforce_fifo_clause(query_where);
    assert_eq!(
        sanitized_where,
        "SELECT * FROM active_prompts WHERE status = 'Enqueued' ORDER BY created_at ASC, id ASC"
    );

    let query_limit = "SELECT * FROM active_prompts LIMIT 25";
    let sanitized_limit = sanitize_or_enforce_fifo_clause(query_limit);
    assert_eq!(
        sanitized_limit,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 25"
    );

    let query_semi = "SELECT * FROM prompt_backups;";
    let sanitized_semi = sanitize_or_enforce_fifo_clause(query_semi);
    assert_eq!(
        sanitized_semi,
        "SELECT * FROM prompt_backups ORDER BY created_at ASC, id ASC;"
    );

    let query_limit_semi = "SELECT * FROM active_prompts LIMIT 10;";
    let sanitized_limit_semi = sanitize_or_enforce_fifo_clause(query_limit_semi);
    assert_eq!(
        sanitized_limit_semi,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 10;"
    );
}

#[test]
fn test_sanitize_or_enforce_replace_invalid_order_by() {
    let query_lifo = "SELECT * FROM active_prompts ORDER BY updated_at DESC LIMIT 10";
    let sanitized_lifo = sanitize_or_enforce_fifo_clause(query_lifo);
    assert_eq!(
        sanitized_lifo,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 10"
    );

    let query_created_desc = "SELECT * FROM prompt_backups ORDER BY created_at DESC";
    let sanitized_created_desc = sanitize_or_enforce_fifo_clause(query_created_desc);
    assert_eq!(
        sanitized_created_desc,
        "SELECT * FROM prompt_backups ORDER BY created_at ASC, id ASC"
    );

    let query_missing_id = "SELECT * FROM active_prompts ORDER BY created_at ASC LIMIT 5";
    let sanitized_missing_id = sanitize_or_enforce_fifo_clause(query_missing_id);
    assert_eq!(
        sanitized_missing_id,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 5"
    );
}

#[test]
fn test_sanitize_already_valid_query() {
    let query_valid = "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 10";
    let sanitized_valid = sanitize_or_enforce_fifo_clause(query_valid);
    assert_eq!(
        sanitized_valid,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 10"
    );
}

#[test]
fn test_sanitize_empty_query() {
    let sanitized = sanitize_or_enforce_fifo_clause("");
    assert_eq!(sanitized, MANDATORY_FIFO_ORDER_CLAUSE);
}

#[test]
fn test_violation_display_and_error_formatting() {
    let v1 = FifoOrderingViolation::ForbiddenLifoDetected {
        query_snippet: "SELECT * FROM active_prompts...".to_string(),
        forbidden_clause: "ORDER BY updated_at DESC".to_string(),
    };
    let s1 = format!("{}", v1);
    assert!(s1.contains("Forbidden LIFO ordering detected"));
    assert!(s1.contains("ORDER BY updated_at DESC"));

    let v2 = FifoOrderingViolation::MissingIdTieBreaker {
        query_snippet: "SELECT * FROM prompt_backups...".to_string(),
    };
    let s2 = format!("{}", v2);
    assert!(s2.contains("Missing mandatory 'id ASC' tie-breaker"));

    let v3 = FifoOrderingViolation::MissingCreatedAtSort {
        query_snippet: "SELECT * FROM repo_prompts...".to_string(),
    };
    let s3 = format!("{}", v3);
    assert!(s3.contains("Missing mandatory 'created_at ASC' primary sort"));

    // Verify std::error::Error implementation
    let err: &dyn std::error::Error = &v1;
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_prompt_restore_query_kind_and_report_constructors() {
    assert_eq!(
        PromptRestoreQueryKind::default(),
        PromptRestoreQueryKind::GeneralPromptQuery
    );

    let rep_valid = FifoValidationReport::valid(
        PromptRestoreQueryKind::ActivePromptsRestore,
        MANDATORY_FIFO_ORDER_CLAUSE.to_string(),
    );
    assert!(rep_valid.is_valid);
    assert!(rep_valid.violation.is_none());

    let rep_invalid = FifoValidationReport::invalid(
        PromptRestoreQueryKind::PromptBackupsRestore,
        "ORDER BY created_at DESC".to_string(),
        FifoOrderingViolation::MissingIdTieBreaker {
            query_snippet: "test".to_string(),
        },
    );
    assert!(!rep_invalid.is_valid);
    assert!(rep_invalid.violation.is_some());
}
