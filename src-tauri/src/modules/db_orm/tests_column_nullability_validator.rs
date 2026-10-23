//! Unit tests for ColumnNullabilityValidator (Imp-576)

use rusqlite::Connection;

use super::column_nullability_validator::{
    inspect_table_columns, validate_required_not_null_columns, NullabilityViolation,
};

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Open in-memory db");
    conn.execute_batch(
        "CREATE TABLE accounts (
            id INTEGER PRIMARY KEY,
            account_id TEXT NOT NULL,
            email TEXT NOT NULL,
            notes TEXT,
            quota_limit INTEGER
        );",
    )
    .expect("Create accounts table");
    conn
}

#[test]
fn test_inspect_table_columns_extracts_all_fields() {
    let conn = setup_test_db();
    let cols = inspect_table_columns(&conn, "accounts").expect("Inspect table columns");

    assert_eq!(cols.len(), 5);

    let id_col = cols.iter().find(|c| c.name == "id").unwrap();
    assert!(id_col.is_pk);

    let account_id_col = cols.iter().find(|c| c.name == "account_id").unwrap();
    assert!(account_id_col.not_null);
    assert!(!account_id_col.is_pk);

    let notes_col = cols.iter().find(|c| c.name == "notes").unwrap();
    assert!(!notes_col.not_null);
    assert!(!notes_col.is_pk);
}

#[test]
fn test_validate_required_not_null_columns_passes_when_compliant() {
    let conn = setup_test_db();
    let violations =
        validate_required_not_null_columns(&conn, "accounts", &["id", "account_id", "email"])
            .expect("Validation query should succeed");

    assert!(
        violations.is_empty(),
        "Compliant columns should produce zero violations"
    );
}

#[test]
fn test_validate_required_not_null_columns_catches_violations() {
    let conn = setup_test_db();
    let violations = validate_required_not_null_columns(
        &conn,
        "accounts",
        &["account_id", "notes", "quota_limit"],
    )
    .expect("Validation query should succeed");

    assert_eq!(violations.len(), 2);
    assert!(violations.contains(&NullabilityViolation {
        table_name: "accounts".to_string(),
        column_name: "notes".to_string(),
        expected_not_null: true,
    }));
    assert!(violations.contains(&NullabilityViolation {
        table_name: "accounts".to_string(),
        column_name: "quota_limit".to_string(),
        expected_not_null: true,
    }));
}
