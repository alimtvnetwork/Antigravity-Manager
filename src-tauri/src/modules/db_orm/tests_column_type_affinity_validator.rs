//! Unit tests for ColumnTypeAffinityValidator (Imp-577)

use rusqlite::Connection;

use super::column_type_affinity_validator::{inspect_table_columns, SqliteAffinity};

#[test]
fn test_sqlite_affinity_from_declared_type() {
    assert_eq!(
        SqliteAffinity::from_declared_type("INT"),
        SqliteAffinity::Integer
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("INTEGER"),
        SqliteAffinity::Integer
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("BIGINT"),
        SqliteAffinity::Integer
    );

    assert_eq!(
        SqliteAffinity::from_declared_type("TEXT"),
        SqliteAffinity::Text
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("VARCHAR(255)"),
        SqliteAffinity::Text
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("CLOB"),
        SqliteAffinity::Text
    );

    assert_eq!(
        SqliteAffinity::from_declared_type("BLOB"),
        SqliteAffinity::Blob
    );
    assert_eq!(SqliteAffinity::from_declared_type(""), SqliteAffinity::Blob);

    assert_eq!(
        SqliteAffinity::from_declared_type("REAL"),
        SqliteAffinity::Real
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("FLOAT"),
        SqliteAffinity::Real
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("DOUBLE"),
        SqliteAffinity::Real
    );

    assert_eq!(
        SqliteAffinity::from_declared_type("NUMERIC"),
        SqliteAffinity::Numeric
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("DECIMAL(10,5)"),
        SqliteAffinity::Numeric
    );
    assert_eq!(
        SqliteAffinity::from_declared_type("BOOLEAN"),
        SqliteAffinity::Numeric
    );
}

#[test]
fn test_inspect_table_columns_affinity_extraction() {
    let conn = Connection::open_in_memory().expect("Open in-memory db");
    conn.execute_batch(
        "CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            weight REAL,
            raw_payload BLOB,
            is_enabled BOOLEAN DEFAULT 1
        );",
    )
    .expect("Create test_table");

    let cols = inspect_table_columns(&conn, "test_table").expect("Inspect columns");
    assert_eq!(cols.len(), 5);

    let id_col = cols.iter().find(|c| c.name == "id").unwrap();
    assert_eq!(id_col.affinity, SqliteAffinity::Integer);
    assert!(id_col.primary_key);

    let name_col = cols.iter().find(|c| c.name == "name").unwrap();
    assert_eq!(name_col.affinity, SqliteAffinity::Text);
    assert!(name_col.not_null);

    let weight_col = cols.iter().find(|c| c.name == "weight").unwrap();
    assert_eq!(weight_col.affinity, SqliteAffinity::Real);

    let blob_col = cols.iter().find(|c| c.name == "raw_payload").unwrap();
    assert_eq!(blob_col.affinity, SqliteAffinity::Blob);

    let bool_col = cols.iter().find(|c| c.name == "is_enabled").unwrap();
    assert_eq!(bool_col.affinity, SqliteAffinity::Numeric);
}
