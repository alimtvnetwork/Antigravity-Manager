#[cfg(test)]
mod tests {
    use crate::modules::error_db::{
        clear_errors, count_errors, list_errors, record_error, run_self_healing, ErrorQueryOptions,
        NewErrorRecord,
    };

    #[test]
    fn test_error_db_lifecycle() {
        // Record test error
        let new_err = NewErrorRecord {
            id: None,
            error_code: "E1001".to_string(),
            severity: "error".to_string(),
            message: "Test network timeout".to_string(),
            stack_trace: Some("at test_error_db_lifecycle".to_string()),
            source: Some("unit_test".to_string()),
            trigger_action: Some("test_action".to_string()),
            metadata_json: Some(r#"{"host":"127.0.0.1"}"#.to_string()),
            created_at: None,
        };

        let err_id = record_error(&new_err).expect("Failed to record error");
        assert!(!err_id.is_empty());

        // Count errors
        let total = count_errors(None).expect("Failed to count errors");
        assert!(total >= 1);

        // List errors
        let list = list_errors(&ErrorQueryOptions {
            limit: Some(10),
            offset: None,
            severity: Some("error".to_string()),
            error_code: Some("E1001".to_string()),
            search: Some("Test network".to_string()),
        })
        .expect("Failed to list errors");

        assert!(list
            .iter()
            .any(|r| r.id == err_id && r.error_code == "E1001"));

        // Clear errors
        let cleared = clear_errors().expect("Failed to clear errors");
        assert!(cleared >= 1);

        let after_count = count_errors(None).expect("Failed to count after clear");
        assert_eq!(after_count, 0);
    }

    #[test]
    fn test_self_healing_dry_run() {
        let report = run_self_healing(false).expect("Self healing dry run failed");
        assert!(report.total_checks >= 7);
    }
}
