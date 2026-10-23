//! Unit tests for Strict FIFO Queue Composite Index Analyzer (`Imp-663`).

use super::fifo_queue_index_analyzer::{
    FifoIndexAnalysisReport, FifoIndexStatusType, FifoQueueIndexAnalyzer, FifoTableIndexStatus,
    IndexRiskLevelType, CANONICAL_FIFO_INDEX_COLUMNS, DEFAULT_PROMPT_QUEUE_TABLES,
};
use rusqlite::Connection;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("In-memory database creation failed");
    conn.execute_batch(
        r#"
        CREATE TABLE active_prompts (
            id TEXT PRIMARY KEY,
            prompt_text TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            status TEXT NOT NULL
        );
        CREATE TABLE prompt_backups (
            id TEXT PRIMARY KEY,
            prompt_text TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            status TEXT NOT NULL
        );
        CREATE TABLE repo_prompts (
            id TEXT PRIMARY KEY,
            repo_path TEXT NOT NULL,
            prompt_text TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            status TEXT NOT NULL
        );
        "#,
    )
    .expect("Schema setup failed");
    conn
}

#[test]
fn test_constants_and_defaults() {
    assert_eq!(CANONICAL_FIFO_INDEX_COLUMNS, &["created_at", "id"]);
    assert_eq!(
        DEFAULT_PROMPT_QUEUE_TABLES,
        &["active_prompts", "prompt_backups", "repo_prompts"]
    );

    let analyzer = FifoQueueIndexAnalyzer::new();
    assert_eq!(analyzer.target_tables().len(), 3);
    assert_eq!(analyzer.target_tables()[0], "active_prompts");
    assert_eq!(analyzer.target_tables()[1], "prompt_backups");
    assert_eq!(analyzer.target_tables()[2], "repo_prompts");
}

#[test]
fn test_generate_remediation_sql() {
    let sql_active = FifoQueueIndexAnalyzer::generate_remediation_sql("active_prompts");
    assert_eq!(
        sql_active,
        "CREATE INDEX IF NOT EXISTS idx_active_prompts_created_at_id ON active_prompts(created_at ASC, id ASC);"
    );

    let sql_backups = FifoQueueIndexAnalyzer::generate_remediation_sql("prompt_backups");
    assert_eq!(
        sql_backups,
        "CREATE INDEX IF NOT EXISTS idx_prompt_backups_created_at_id ON prompt_backups(created_at ASC, id ASC);"
    );

    let sql_repo = FifoQueueIndexAnalyzer::generate_remediation_sql("repo_prompts");
    assert_eq!(
        sql_repo,
        "CREATE INDEX IF NOT EXISTS idx_repo_prompts_created_at_id ON repo_prompts(created_at ASC, id ASC);"
    );
}

#[test]
fn test_empty_database_table_not_found() {
    let conn = Connection::open_in_memory().expect("In-memory db creation failed");
    let analyzer = FifoQueueIndexAnalyzer::new();

    let status = analyzer
        .analyze_table(&conn, "active_prompts")
        .expect("Analysis failed");

    assert_eq!(status.table_name(), "active_prompts");
    assert_eq!(status.status, FifoIndexStatusType::TableNotFoundType);
    assert!(!status.has_composite_index());
    assert!(!status.is_compliant());
    assert_eq!(status.risk_level(), IndexRiskLevelType::MediumType);
    assert!(status.remediation_sql().is_some());
    assert_eq!(status.existing_indices.len(), 0);
}

#[test]
fn test_missing_composite_index_detected() {
    let conn = setup_test_db();
    let analyzer = FifoQueueIndexAnalyzer::new();

    let status = analyzer
        .analyze_table(&conn, "active_prompts")
        .expect("Analysis failed");

    assert_eq!(status.table_name(), "active_prompts");
    assert_eq!(
        status.status,
        FifoIndexStatusType::MissingCompositeIndexType
    );
    assert!(!status.has_composite_index());
    assert_eq!(status.risk_level(), IndexRiskLevelType::HighType);
    assert!(status.matching_index_name.is_none());
    assert_eq!(
        status.remediation_sql(),
        Some("CREATE INDEX IF NOT EXISTS idx_active_prompts_created_at_id ON active_prompts(created_at ASC, id ASC);")
    );
}

#[test]
fn test_existing_composite_index_compliant() {
    let conn = setup_test_db();
    conn.execute(
        "CREATE INDEX idx_active_prompts_created_at_id ON active_prompts(created_at ASC, id ASC);",
        [],
    )
    .expect("Index creation failed");

    let analyzer = FifoQueueIndexAnalyzer::new();
    let status = analyzer
        .analyze_table(&conn, "active_prompts")
        .expect("Analysis failed");

    assert_eq!(status.status, FifoIndexStatusType::FullyCompliantType);
    assert!(status.has_composite_index());
    assert!(status.is_compliant());
    assert_eq!(status.risk_level(), IndexRiskLevelType::NoneType);
    assert_eq!(
        status.matching_index_name.as_deref(),
        Some("idx_active_prompts_created_at_id")
    );
    assert!(status.remediation_sql().is_none());
}

#[test]
fn test_improper_sort_direction_detected() {
    let conn = setup_test_db();
    conn.execute(
        "CREATE INDEX idx_lifo_desc ON active_prompts(created_at DESC, id ASC);",
        [],
    )
    .expect("Index creation failed");

    let analyzer = FifoQueueIndexAnalyzer::new();
    let status = analyzer
        .analyze_table(&conn, "active_prompts")
        .expect("Analysis failed");

    assert_eq!(
        status.status,
        FifoIndexStatusType::ImproperSortDirectionType
    );
    assert!(!status.has_composite_index());
    assert_eq!(status.risk_level(), IndexRiskLevelType::HighType);
    assert_eq!(status.matching_index_name.as_deref(), Some("idx_lifo_desc"));
    assert!(status.remediation_sql().is_some());
}

#[test]
fn test_improper_column_order_detected() {
    let conn = setup_test_db();
    conn.execute(
        "CREATE INDEX idx_id_then_created ON active_prompts(id ASC, created_at ASC);",
        [],
    )
    .expect("Index creation failed");

    let analyzer = FifoQueueIndexAnalyzer::new();
    let status = analyzer
        .analyze_table(&conn, "active_prompts")
        .expect("Analysis failed");

    assert_eq!(status.status, FifoIndexStatusType::ImproperColumnOrderType);
    assert!(!status.has_composite_index());
    assert_eq!(status.risk_level(), IndexRiskLevelType::HighType);
    assert_eq!(
        status.matching_index_name.as_deref(),
        Some("idx_id_then_created")
    );
    assert!(status.remediation_sql().is_some());
}

#[test]
fn test_partial_single_column_index_detected() {
    let conn = setup_test_db();
    conn.execute(
        "CREATE INDEX idx_only_created_at ON active_prompts(created_at ASC);",
        [],
    )
    .expect("Index creation failed");

    let analyzer = FifoQueueIndexAnalyzer::new();
    let status = analyzer
        .analyze_table(&conn, "active_prompts")
        .expect("Analysis failed");

    assert_eq!(
        status.status,
        FifoIndexStatusType::MissingCompositeIndexType
    );
    assert!(!status.has_composite_index());
    assert_eq!(status.risk_level(), IndexRiskLevelType::MediumType);
    assert!(status.remediation_sql().is_some());
}

#[test]
fn test_full_analysis_report_all_tables() {
    let conn = setup_test_db();
    // Index only active_prompts; leave prompt_backups and repo_prompts unindexed
    conn.execute(
        "CREATE INDEX idx_active_prompts_created_at_id ON active_prompts(created_at ASC, id ASC);",
        [],
    )
    .expect("Index creation failed");

    let analyzer = FifoQueueIndexAnalyzer::new();
    let report: FifoIndexAnalysisReport = analyzer.analyze(&conn).expect("Report analysis failed");

    assert!(!report.is_fully_indexed());
    assert_eq!(report.missing_index_count(), 2);
    assert_eq!(report.inspected_tables().len(), 3);
    assert_eq!(report.remediation_sql().len(), 2);

    assert!(report.inspected_tables()[0].is_compliant());
    assert!(!report.inspected_tables()[1].is_compliant());
    assert!(!report.inspected_tables()[2].is_compliant());
}

#[test]
fn test_apply_remediations_achieves_full_compliance() {
    let conn = setup_test_db();
    let analyzer = FifoQueueIndexAnalyzer::new();

    let initial_report = analyzer.analyze(&conn).expect("Initial analysis failed");
    assert_eq!(initial_report.missing_index_count(), 3);
    assert!(!initial_report.is_fully_indexed());

    let applied_count = analyzer
        .apply_remediations(&conn, &initial_report)
        .expect("Remediation execution failed");
    assert_eq!(applied_count, 3);

    let final_report = analyzer
        .analyze(&conn)
        .expect("Post-remediation analysis failed");
    assert!(final_report.is_fully_indexed());
    assert_eq!(final_report.missing_index_count(), 0);
    assert!(final_report.remediation_sql().is_empty());

    for table in final_report.inspected_tables() {
        assert!(table.is_compliant());
        assert_eq!(table.risk_level(), IndexRiskLevelType::NoneType);
        assert!(table.remediation_sql().is_none());
    }
}

#[test]
fn test_custom_tables_analyzer() {
    let analyzer = FifoQueueIndexAnalyzer::with_tables(&["custom_queue", "legacy_queue"]);
    assert_eq!(analyzer.target_tables().len(), 2);
    assert_eq!(analyzer.target_tables()[0], "custom_queue");
    assert_eq!(analyzer.target_tables()[1], "legacy_queue");
}

#[test]
fn test_display_and_formatting() {
    assert_eq!(
        format!("{}", FifoIndexStatusType::FullyCompliantType),
        "Fully Compliant"
    );
    assert_eq!(
        format!("{}", FifoIndexStatusType::MissingCompositeIndexType),
        "Missing Composite Index"
    );
    assert_eq!(
        format!("{}", FifoIndexStatusType::ImproperColumnOrderType),
        "Improper Column Order"
    );
    assert_eq!(
        format!("{}", FifoIndexStatusType::ImproperSortDirectionType),
        "Improper Sort Direction"
    );
    assert_eq!(
        format!("{}", FifoIndexStatusType::TableNotFoundType),
        "Table Not Found"
    );

    assert_eq!(format!("{}", IndexRiskLevelType::NoneType), "None");
    assert_eq!(format!("{}", IndexRiskLevelType::LowType), "Low");
    assert_eq!(format!("{}", IndexRiskLevelType::MediumType), "Medium");
    assert_eq!(format!("{}", IndexRiskLevelType::HighType), "High");
    assert_eq!(format!("{}", IndexRiskLevelType::CriticalType), "Critical");
}

#[test]
fn test_combined_remediation_script() {
    let status_active = FifoTableIndexStatus {
        table_name: "active_prompts".to_string(),
        status: FifoIndexStatusType::MissingCompositeIndexType,
        has_composite_index: false,
        matching_index_name: None,
        risk_level: IndexRiskLevelType::HighType,
        remediation_sql: Some("CREATE INDEX idx1 ON active_prompts(created_at, id);".to_string()),
        existing_indices: Vec::new(),
    };
    let status_backups = FifoTableIndexStatus {
        table_name: "prompt_backups".to_string(),
        status: FifoIndexStatusType::MissingCompositeIndexType,
        has_composite_index: false,
        matching_index_name: None,
        risk_level: IndexRiskLevelType::HighType,
        remediation_sql: Some("CREATE INDEX idx2 ON prompt_backups(created_at, id);".to_string()),
        existing_indices: Vec::new(),
    };

    let report = FifoIndexAnalysisReport {
        inspected_tables: vec![status_active, status_backups],
        missing_index_count: 2,
        is_fully_indexed: false,
        remediation_sql: vec![
            "CREATE INDEX idx1 ON active_prompts(created_at, id);".to_string(),
            "CREATE INDEX idx2 ON prompt_backups(created_at, id);".to_string(),
        ],
    };

    let script = report.combined_remediation_script();
    assert!(script.contains("CREATE INDEX idx1"));
    assert!(script.contains("CREATE INDEX idx2"));
}

#[test]
fn test_query_plan_optimization_using_index() {
    let conn = setup_test_db();
    let analyzer = FifoQueueIndexAnalyzer::new();

    // Query plan before index: uses temp b-tree
    let mut stmt = conn
        .prepare("EXPLAIN QUERY PLAN SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC")
        .expect("Explain query plan failed");
    let mut rows = stmt.query([]).expect("Explain query failed");
    let mut details_before = Vec::new();
    while let Some(row) = rows.next().expect("Row fetch failed") {
        let detail: String = row.get(3).expect("Detail column fetch failed");
        details_before.push(detail);
    }
    let plan_before = details_before.join(" ");
    assert!(
        plan_before.contains("B-TREE") || plan_before.contains("SCAN"),
        "Unindexed query should trigger table scan or temp sort: {}",
        plan_before
    );

    // Create index
    let report = analyzer.analyze(&conn).expect("Analysis failed");
    analyzer
        .apply_remediations(&conn, &report)
        .expect("Remediations failed");

    // Query plan after index: uses index
    let mut stmt_after = conn
        .prepare("EXPLAIN QUERY PLAN SELECT * FROM active_prompts ORDER BY created_at ASC, id ASC")
        .expect("Explain query plan failed");
    let mut rows_after = stmt_after.query([]).expect("Explain query failed");
    let mut details_after = Vec::new();
    while let Some(row) = rows_after.next().expect("Row fetch failed") {
        let detail: String = row.get(3).expect("Detail column fetch failed");
        details_after.push(detail);
    }
    let plan_after = details_after.join(" ");
    assert!(
        plan_after.contains("USING INDEX idx_active_prompts_created_at_id"),
        "Indexed query must use idx_active_prompts_created_at_id: {}",
        plan_after
    );
}
