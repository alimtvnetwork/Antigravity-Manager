//! Comprehensive test suite for `FifoPromptOrderEnforcer` (`Imp-654`).

use super::fifo_prompt_order_enforcer::{
    detect_target_table, enforce_fifo_ordering, is_prompt_queue_table, FifoEnforcementAction,
    FifoEnforcementReport, CANONICAL_FIFO_ORDER_BY,
};

#[test]
fn test_already_compliant_query() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 50;";
    let report = enforce_fifo_ordering(sql);

    assert_eq!(report.action(), &FifoEnforcementAction::AlreadyCompliant);
    assert!(!report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert!(report.is_compliant());
    assert_eq!(report.enforced_sql(), sql);
}

#[test]
fn test_case_insensitive_compliant_query() {
    let sql = "select id, payload from repo_prompts order by created_at asc, id asc";
    let report = enforce_fifo_ordering(sql);

    assert_eq!(report.action(), &FifoEnforcementAction::AlreadyCompliant);
    assert!(!report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert!(report.is_compliant());
    assert_eq!(report.enforced_sql(), sql);
}

#[test]
fn test_replaces_lifo_desc_order() {
    let sql = "SELECT * FROM active_prompts ORDER BY updated_at DESC LIMIT 10;";
    let report = enforce_fifo_ordering(sql);

    match report.action() {
        FifoEnforcementAction::ReplacedLIFOOrder { original_clause, canonical_clause } => {
            assert!(original_clause.contains("updated_at DESC"));
            assert_eq!(canonical_clause, CANONICAL_FIFO_ORDER_BY);
        }
        _ => panic!("Expected ReplacedLIFOOrder action"),
    }

    assert!(report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert!(!report.is_compliant());
    assert_eq!(
        report.enforced_sql(),
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 10;"
    );
}

#[test]
fn test_replaces_inverted_created_at_desc() {
    let sql = "SELECT * FROM prompt_backups ORDER BY created_at DESC;";
    let report = enforce_fifo_ordering(sql);

    assert!(report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert_eq!(
        report.enforced_sql(),
        "SELECT * FROM prompt_backups ORDER BY created_at ASC, id ASC;"
    );
}

#[test]
fn test_replaces_order_missing_id_tie_breaker() {
    let sql = "SELECT * FROM active_prompts ORDER BY created_at ASC LIMIT 1;";
    let report = enforce_fifo_ordering(sql);

    assert!(report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert_eq!(
        report.enforced_sql(),
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC LIMIT 1;"
    );
}

#[test]
fn test_appends_canonical_order_when_missing() {
    let sql = "SELECT * FROM prompts_queue WHERE status = 'pending' LIMIT 5;";
    let report = enforce_fifo_ordering(sql);

    assert_eq!(report.action(), &FifoEnforcementAction::AppendedCanonicalOrder);
    assert!(report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert_eq!(
        report.enforced_sql(),
        "SELECT * FROM prompts_queue WHERE status = 'pending' ORDER BY created_at ASC, id ASC LIMIT 5;"
    );
}

#[test]
fn test_appends_order_at_end_of_query_without_limit() {
    let sql = "SELECT id, prompt_id FROM repo_prompts";
    let report = enforce_fifo_ordering(sql);

    assert_eq!(report.action(), &FifoEnforcementAction::AppendedCanonicalOrder);
    assert!(report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert_eq!(
        report.enforced_sql(),
        "SELECT id, prompt_id FROM repo_prompts ORDER BY created_at ASC, id ASC"
    );
}

#[test]
fn test_non_target_table_passthrough() {
    let sql = "SELECT * FROM user_accounts ORDER BY last_login DESC LIMIT 10;";
    let report = enforce_fifo_ordering(sql);

    assert_eq!(report.action(), &FifoEnforcementAction::NonTargetQueryPassThrough);
    assert!(!report.is_modified());
    assert!(report.is_fifo_guaranteed());
    assert_eq!(report.enforced_sql(), sql);
}

#[test]
fn test_empty_query_rejected() {
    let sql = "   ";
    let report = enforce_fifo_ordering(sql);

    match report.action() {
        FifoEnforcementAction::RejectedUnsanitizableQuery { reason } => {
            assert!(reason.contains("empty"));
        }
        _ => panic!("Expected RejectedUnsanitizableQuery action"),
    }
    assert!(!report.is_modified());
    assert!(!report.is_fifo_guaranteed());
}

#[test]
fn test_is_prompt_queue_table_helper() {
    assert!(is_prompt_queue_table("active_prompts"));
    assert!(is_prompt_queue_table("prompt_backups"));
    assert!(is_prompt_queue_table("repo_prompts"));
    assert!(is_prompt_queue_table("prompts_queue"));
    assert!(is_prompt_queue_table("`active_prompts`"));
    assert!(is_prompt_queue_table("\"repo_prompts\""));
    assert!(is_prompt_queue_table("main.active_prompts"));

    assert!(!is_prompt_queue_table("users"));
    assert!(!is_prompt_queue_table("account_settings"));
}

#[test]
fn test_detect_target_table() {
    assert_eq!(
        detect_target_table("SELECT * FROM active_prompts WHERE id = 1"),
        Some("active_prompts".to_string())
    );
    assert_eq!(
        detect_target_table("SELECT * FROM custom_table"),
        Some("custom_table".to_string())
    );
}

#[test]
fn test_serde_roundtrip_report() {
    let original = FifoEnforcementReport::new(
        FifoEnforcementAction::AppendedCanonicalOrder,
        "SELECT * FROM active_prompts".to_string(),
        "SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC".to_string(),
        true,
        true,
    );

    let json = serde_json::to_string(&original).expect("serialize report");
    let recovered: FifoEnforcementReport = serde_json::from_str(&json).expect("deserialize report");

    assert_eq!(recovered, original);
}
