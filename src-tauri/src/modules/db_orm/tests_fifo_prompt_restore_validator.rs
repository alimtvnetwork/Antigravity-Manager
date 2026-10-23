//! Unit tests for Strict FIFO Prompt Restore Query Validator (`Imp-644`).

use super::fifo_prompt_restore_validator::{
    build_canonical_fifo_query, extract_order_by_clause, extract_table_name,
    sanitize_or_enforce_fifo_clause, validate_fifo_query, validate_prompt_restore_sql,
    FifoOrderingViolation, FifoQueryValidationReport, FifoQueryViolation,
    PromptRestoreQueryKind, MANDATORY_FIFO_ORDER_CLAUSE,
};

// ============================================================================
// Imp-644 Tests
// ============================================================================

#[test]
fn test_canonical_fifo_query_passes_clean() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC";
    let report = validate_fifo_query(sql);

    assert!(report.is_valid_fifo);
    assert!(!report.has_violations());
    assert_eq!(report.violation_count(), 0);
    assert_eq!(report.table_name.as_deref(), Some("active_prompts"));
    assert_eq!(
        report.order_by_clause.as_deref(),
        Some("created_at ASC, id ASC")
    );
    assert!(report.violations.is_empty());
}

#[test]
fn test_canonical_fifo_query_with_limit_and_where_passes_clean() {
    let sql = "SELECT id, prompt_text FROM prompt_backups WHERE status = 'Archived' ORDER BY created_at ASC, id ASC LIMIT 50";
    let report = validate_fifo_query(sql);

    assert!(report.is_valid_fifo);
    assert!(!report.has_violations());
    assert_eq!(report.violation_count(), 0);
    assert_eq!(report.table_name.as_deref(), Some("prompt_backups"));
    assert_eq!(
        report.order_by_clause.as_deref(),
        Some("created_at ASC, id ASC")
    );
}

#[test]
fn test_canonical_fifo_query_with_table_prefix_passes_clean() {
    let sql = "SELECT * FROM active_prompts ORDER BY active_prompts.created_at ASC, active_prompts.id ASC";
    let report = validate_fifo_query(sql);

    assert!(report.is_valid_fifo);
    assert!(!report.has_violations());
    assert_eq!(report.violation_count(), 0);
}

#[test]
fn test_inverted_lifo_query_updated_at_desc_flagged() {
    let sql = "SELECT * FROM active_prompts ORDER BY updated_at DESC";
    let report = validate_fifo_query(sql);

    assert!(!report.is_valid_fifo);
    assert!(report.has_violations());
    assert_eq!(report.violation_count(), 1);
    assert_eq!(
        report.violations,
        vec![FifoQueryViolation::InvertedLIFOOrderDetected {
            clause: "updated_at DESC".to_string(),
        }]
    );
}

#[test]
fn test_inverted_query_created_at_desc_flagged() {
    let sql = "SELECT * FROM prompt_backups ORDER BY created_at DESC";
    let report = validate_fifo_query(sql);

    assert!(!report.is_valid_fifo);
    assert!(report.has_violations());
    assert_eq!(report.violation_count(), 1);
    assert_eq!(
        report.violations,
        vec![FifoQueryViolation::InvertedLIFOOrderDetected {
            clause: "created_at DESC".to_string(),
        }]
    );
}

#[test]
fn test_missing_id_asc_tiebreaker_flagged() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC";
    let report = validate_fifo_query(sql);

    assert!(!report.is_valid_fifo);
    assert!(report.has_violations());
    assert_eq!(report.violation_count(), 1);
    assert_eq!(
        report.violations,
        vec![FifoQueryViolation::MissingTieBreakerIdAsc {
            clause: "created_at ASC".to_string(),
        }]
    );
}

#[test]
fn test_missing_order_by_clause_flagged() {
    let sql = "SELECT * FROM repo_prompts WHERE status = 'pending'";
    let report = validate_fifo_query(sql);

    assert!(!report.is_valid_fifo);
    assert!(report.has_violations());
    assert_eq!(report.violation_count(), 1);
    assert_eq!(report.table_name.as_deref(), Some("repo_prompts"));
    assert!(report.order_by_clause.is_none());
    assert_eq!(
        report.violations,
        vec![FifoQueryViolation::MissingOrderByClause]
    );
}

#[test]
fn test_disallowed_sort_column_flagged() {
    let sql = "SELECT * FROM active_prompts ORDER BY priority ASC, id ASC";
    let report = validate_fifo_query(sql);

    assert!(!report.is_valid_fifo);
    assert!(report.has_violations());
    assert_eq!(report.violation_count(), 1);
    assert_eq!(
        report.violations,
        vec![FifoQueryViolation::DisallowedSortColumn {
            column: "priority".to_string(),
        }]
    );
}

#[test]
fn test_disallowed_sort_column_without_tiebreaker_flags_both() {
    let sql = "SELECT * FROM active_prompts ORDER BY priority ASC";
    let report = validate_fifo_query(sql);

    assert!(!report.is_valid_fifo);
    assert!(report.has_violations());
    assert_eq!(report.violation_count(), 2);
    assert_eq!(
        report.violations,
        vec![
            FifoQueryViolation::DisallowedSortColumn {
                column: "priority".to_string(),
            },
            FifoQueryViolation::MissingTieBreakerIdAsc {
                clause: "priority ASC".to_string(),
            },
        ]
    );
}

#[test]
fn test_canonical_query_generator_without_limit() {
    let sql = build_canonical_fifo_query("active_prompts", None);
    assert_eq!(
        sql,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC"
    );

    let report = validate_fifo_query(&sql);
    assert!(report.is_valid_fifo);
    assert!(!report.has_violations());
    assert_eq!(report.table_name.as_deref(), Some("active_prompts"));
}

#[test]
fn test_canonical_query_generator_with_limit() {
    let sql = build_canonical_fifo_query("prompt_backups", Some(100));
    assert_eq!(
        sql,
        "SELECT * FROM prompt_backups ORDER BY created_at ASC, id ASC LIMIT 100"
    );

    let report = validate_fifo_query(&sql);
    assert!(report.is_valid_fifo);
    assert!(!report.has_violations());
    assert_eq!(report.table_name.as_deref(), Some("prompt_backups"));
}

#[test]
fn test_serde_roundtrip_all_violation_variants() {
    let violations = vec![
        FifoQueryViolation::InvertedLIFOOrderDetected {
            clause: "updated_at DESC".to_string(),
        },
        FifoQueryViolation::MissingTieBreakerIdAsc {
            clause: "created_at ASC".to_string(),
        },
        FifoQueryViolation::MissingOrderByClause,
        FifoQueryViolation::DisallowedSortColumn {
            column: "title".to_string(),
        },
    ];

    for violation in &violations {
        let serialized = serde_json::to_string(violation).expect("serialize violation");
        let deserialized: FifoQueryViolation =
            serde_json::from_str(&serialized).expect("deserialize violation");
        assert_eq!(&deserialized, violation);
    }
}

#[test]
fn test_serde_roundtrip_validation_report() {
    let valid_report = FifoQueryValidationReport::new(
        true,
        Some("created_at ASC, id ASC".to_string()),
        Some("active_prompts".to_string()),
        Vec::new(),
    );

    let json_valid = serde_json::to_string(&valid_report).expect("serialize valid report");
    let de_valid: FifoQueryValidationReport =
        serde_json::from_str(&json_valid).expect("deserialize valid report");
    assert_eq!(de_valid, valid_report);

    let invalid_report = FifoQueryValidationReport::new(
        false,
        Some("updated_at DESC".to_string()),
        Some("prompt_backups".to_string()),
        vec![FifoQueryViolation::InvertedLIFOOrderDetected {
            clause: "updated_at DESC".to_string(),
        }],
    );

    let json_invalid = serde_json::to_string(&invalid_report).expect("serialize invalid report");
    let de_invalid: FifoQueryValidationReport =
        serde_json::from_str(&json_invalid).expect("deserialize invalid report");
    assert_eq!(de_invalid, invalid_report);
}

#[test]
fn test_display_implementation() {
    let v1 = FifoQueryViolation::InvertedLIFOOrderDetected {
        clause: "updated_at DESC".to_string(),
    };
    assert!(v1.to_string().contains("Inverted LIFO ordering detected"));

    let v2 = FifoQueryViolation::MissingTieBreakerIdAsc {
        clause: "created_at ASC".to_string(),
    };
    assert!(v2.to_string().contains("Missing mandatory secondary tie-breaker"));

    let v3 = FifoQueryViolation::MissingOrderByClause;
    assert!(v3.to_string().contains("Missing ORDER BY clause"));

    let v4 = FifoQueryViolation::DisallowedSortColumn {
        column: "ranking".to_string(),
    };
    assert!(v4.to_string().contains("Disallowed sort column 'ranking'"));
}

#[test]
fn test_table_name_and_clause_extractions() {
    let sql_repo = "SELECT * FROM repo_prompts.db ORDER BY created_at ASC, id ASC;";
    let table = extract_table_name(sql_repo);
    assert_eq!(table.as_deref(), Some("repo_prompts"));

    let clause = extract_order_by_clause(sql_repo);
    assert_eq!(clause.as_deref(), Some("created_at ASC, id ASC"));

    let sql_custom = "SELECT * FROM custom_queue ORDER BY created_at ASC, id ASC";
    assert_eq!(
        extract_table_name(sql_custom).as_deref(),
        Some("custom_queue")
    );
}

#[test]
fn test_case_insensitive_query_handling() {
    let sql_lower = "select * from active_prompts order by created_at asc, id asc";
    let report = validate_fifo_query(sql_lower);
    assert!(report.is_valid_fifo);
    assert!(!report.has_violations());
    assert_eq!(report.table_name.as_deref(), Some("active_prompts"));
}

// ============================================================================
// Legacy Tests (Imp-623 Backwards Compatibility)
// ============================================================================

#[test]
fn test_legacy_valid_sql_canonical_fifo_passes() {
    let sql = "SELECT id, prompt_text FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 50;";
    let report = validate_prompt_restore_sql(sql, PromptRestoreQueryKind::ActivePromptsRestore);

    assert!(report.is_valid);
    assert_eq!(report.query_kind, PromptRestoreQueryKind::ActivePromptsRestore);
    assert_eq!(report.normalized_order_clause, MANDATORY_FIFO_ORDER_CLAUSE);
    assert!(report.violation.is_none());
}

#[test]
fn test_legacy_inverted_lifo_created_at_desc_rejected() {
    let bad_sql = "SELECT * FROM prompt_backups ORDER BY created_at DESC LIMIT 10";
    let report = validate_prompt_restore_sql(bad_sql, PromptRestoreQueryKind::PromptBackupsRestore);

    assert!(!report.is_valid);
    match report.violation {
        Some(FifoOrderingViolation::ForbiddenLifoDetected { forbidden_clause, .. }) => {
            assert!(
                forbidden_clause.contains("created_at DESC")
                    || forbidden_clause.contains("ORDER BY created_at DESC")
            );
        }
        other => panic!("Expected ForbiddenLifoDetected, got: {:?}", other),
    }
}

#[test]
fn test_legacy_sanitize_or_enforce_fifo_clause() {
    let query_no_order = "SELECT * FROM active_prompts WHERE status = 'active'";
    let sanitized = sanitize_or_enforce_fifo_clause(query_no_order);
    assert!(sanitized.contains("ORDER BY created_at ASC, id ASC"));

    let query_with_limit = "SELECT * FROM active_prompts LIMIT 10";
    let sanitized_limit = sanitize_or_enforce_fifo_clause(query_with_limit);
    assert_eq!(
        sanitized_limit,
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 10"
    );
}
