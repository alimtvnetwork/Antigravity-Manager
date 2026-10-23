#[cfg(test)]
mod tests {
    use super::super::fifo_queue_ordering_guard::{
        detect_lifo_patterns, detect_target_table, enforce_fifo_or_normalize,
        extract_order_by_clause, inspect_and_validate_query, is_fifo_monitored_query,
        normalize_to_fifo_dispatch_query, validate_dispatch_query, FifoOrderingViolation,
        CANONICAL_FIFO_ORDER_BY,
    };

    #[test]
    fn test_valid_fifo_queries_pass_validation() {
        let valid_queries = [
            "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC",
            "SELECT id, payload FROM prompt_backups WHERE status = 'queued' ORDER BY created_at ASC, id ASC LIMIT 10;",
            "SELECT * FROM repo_prompts.db WHERE ready = 1 ORDER BY created_at ASC, id ASC;",
            "SELECT p.id, p.status FROM active_prompts p ORDER BY p.created_at ASC, p.id ASC LIMIT 4",
        ];

        for query in valid_queries {
            assert!(
                validate_dispatch_query(query).is_ok(),
                "Query should be valid FIFO: {query}"
            );
        }
    }

    #[test]
    fn test_lifo_ordering_forbidden_updated_at_desc() {
        let query = "SELECT * FROM active_prompts ORDER BY updated_at DESC";
        let result = validate_dispatch_query(query);

        match result {
            Err(FifoOrderingViolation::LIFOOrderingForbidden { query: q, details }) => {
                assert_eq!(q, query);
                assert!(details.contains("updated_at DESC"));
            }
            other => panic!("Expected LIFOOrderingForbidden, got: {other:?}"),
        }
    }

    #[test]
    fn test_lifo_ordering_forbidden_created_at_desc() {
        let query = "SELECT * FROM prompt_backups ORDER BY created_at DESC LIMIT 5;";
        let result = validate_dispatch_query(query);

        match result {
            Err(FifoOrderingViolation::LIFOOrderingForbidden { details, .. }) => {
                assert!(details.contains("created_at DESC"));
            }
            other => panic!("Expected LIFOOrderingForbidden, got: {other:?}"),
        }
    }

    #[test]
    fn test_missing_ascending_tie_breaker_rejected() {
        let query = "SELECT * FROM active_prompts ORDER BY created_at ASC";
        let result = validate_dispatch_query(query);

        match result {
            Err(FifoOrderingViolation::MissingAscendingTieBreaker { found_order, .. }) => {
                assert_eq!(found_order, "created_at ASC");
            }
            other => panic!("Expected MissingAscendingTieBreaker, got: {other:?}"),
        }
    }

    #[test]
    fn test_missing_order_by_clause_rejected() {
        let query = "SELECT * FROM active_prompts WHERE status = 'pending';";
        let result = validate_dispatch_query(query);

        match result {
            Err(FifoOrderingViolation::MissingOrderByClause { target_table, .. }) => {
                assert_eq!(target_table, "active_prompts");
            }
            other => panic!("Expected MissingOrderByClause, got: {other:?}"),
        }
    }

    #[test]
    fn test_invalid_primary_order_column_rejected() {
        let query = "SELECT * FROM active_prompts ORDER BY priority ASC, id ASC";
        let result = validate_dispatch_query(query);

        match result {
            Err(FifoOrderingViolation::InvalidPrimaryOrderColumn { found_column, .. }) => {
                assert_eq!(found_column, "priority");
            }
            other => panic!("Expected InvalidPrimaryOrderColumn, got: {other:?}"),
        }
    }

    #[test]
    fn test_empty_query_rejected() {
        assert_eq!(
            validate_dispatch_query(""),
            Err(FifoOrderingViolation::EmptyQuery)
        );
        assert_eq!(
            validate_dispatch_query("   \n\t  "),
            Err(FifoOrderingViolation::EmptyQuery)
        );
    }

    #[test]
    fn test_is_fifo_monitored_query_and_detect_table() {
        assert!(is_fifo_monitored_query("SELECT * FROM active_prompts"));
        assert!(is_fifo_monitored_query("INSERT INTO prompt_backups VALUES (...)"));
        assert!(is_fifo_monitored_query("ATTACH 'repo_prompts.db' AS rdb"));
        assert!(!is_fifo_monitored_query("SELECT * FROM user_settings"));

        assert_eq!(
            detect_target_table("SELECT * FROM active_prompts"),
            Some("active_prompts".to_string())
        );
        assert_eq!(
            detect_target_table("SELECT * FROM prompt_backups"),
            Some("prompt_backups".to_string())
        );
        assert_eq!(
            detect_target_table("SELECT * FROM user_settings"),
            None
        );
    }

    #[test]
    fn test_extract_order_by_clause() {
        assert_eq!(
            extract_order_by_clause("SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 5"),
            Some("created_at ASC, id ASC".to_string())
        );
        assert_eq!(
            extract_order_by_clause("SELECT * FROM active_prompts ORDER BY updated_at DESC;"),
            Some("updated_at DESC".to_string())
        );
        assert_eq!(
            extract_order_by_clause("SELECT * FROM active_prompts WHERE id = 1"),
            None
        );
    }

    #[test]
    fn test_detect_lifo_patterns() {
        assert!(detect_lifo_patterns("ORDER BY updated_at DESC").is_some());
        assert!(detect_lifo_patterns("ORDER BY created_at DESC").is_some());
        assert!(detect_lifo_patterns("ORDER BY id DESC").is_some());
        assert!(detect_lifo_patterns("ORDER BY created_at ASC, id ASC").is_none());
    }

    #[test]
    fn test_normalize_replaces_lifo_order() {
        let query = "SELECT * FROM active_prompts ORDER BY updated_at DESC LIMIT 5;";
        let normalized = normalize_to_fifo_dispatch_query(query);
        assert_eq!(
            normalized,
            "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 5;"
        );
        assert!(validate_dispatch_query(&normalized).is_ok());
    }

    #[test]
    fn test_normalize_inserts_missing_order_by_before_limit() {
        let query = "SELECT * FROM prompt_backups WHERE status = 'pending' LIMIT 10;";
        let normalized = normalize_to_fifo_dispatch_query(query);
        assert_eq!(
            normalized,
            "SELECT * FROM prompt_backups WHERE status = 'pending' ORDER BY created_at ASC, id ASC LIMIT 10;"
        );
        assert!(validate_dispatch_query(&normalized).is_ok());
    }

    #[test]
    fn test_normalize_inserts_missing_order_by_before_semicolon() {
        let query = "SELECT * FROM active_prompts;";
        let normalized = normalize_to_fifo_dispatch_query(query);
        assert_eq!(
            normalized,
            "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC;"
        );
        assert!(validate_dispatch_query(&normalized).is_ok());
    }

    #[test]
    fn test_normalize_fixes_missing_tie_breaker() {
        let query = "SELECT * FROM repo_prompts.db ORDER BY created_at ASC";
        let normalized = normalize_to_fifo_dispatch_query(query);
        assert_eq!(
            normalized,
            "SELECT * FROM repo_prompts.db ORDER BY created_at ASC, id ASC"
        );
        assert!(validate_dispatch_query(&normalized).is_ok());
    }

    #[test]
    fn test_inspect_and_validate_query_reports() {
        let compliant = "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC";
        let report = inspect_and_validate_query(compliant);
        assert!(report.is_fifo_compliant);
        assert_eq!(report.target_detected, Some("active_prompts".to_string()));
        assert_eq!(report.extracted_order_by, Some("created_at ASC, id ASC".to_string()));
        assert!(report.violations.is_empty());
        assert_eq!(report.normalized_query, None);

        let non_compliant = "SELECT * FROM active_prompts ORDER BY updated_at DESC";
        let report2 = inspect_and_validate_query(non_compliant);
        assert!(!report2.is_fifo_compliant);
        assert_eq!(report2.violations.len(), 1);
        assert_eq!(
            report2.normalized_query,
            Some("SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC".to_string())
        );
    }

    #[test]
    fn test_enforce_fifo_or_normalize() {
        let valid = "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC";
        assert_eq!(enforce_fifo_or_normalize(valid), valid);

        let invalid = "SELECT * FROM active_prompts ORDER BY updated_at DESC";
        assert_eq!(
            enforce_fifo_or_normalize(invalid),
            "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC"
        );
    }
}
