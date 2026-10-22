//! Schema Column Nullability Validator.
//! Inspects table definitions via `PRAGMA table_info;` to ensure required columns
//! (primary keys, foreign keys, timestamps) enforce strict NOT NULL constraints.

use rusqlite::Connection;

/// Summary of column definition extracted from table info pragma.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub cid: i32,
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub is_pk: bool,
}

/// Validation violation for column nullability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullabilityViolation {
    pub table_name: String,
    pub column_name: String,
    pub expected_not_null: bool,
}

/// Inspects all columns of a specified table.
pub fn inspect_table_columns(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<ColumnInfo>, rusqlite::Error> {
    let sql = format!("PRAGMA table_info({});", table_name);
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map([], |row| {
        Ok(ColumnInfo {
            cid: row.get(0)?,
            name: row.get(1)?,
            data_type: row.get(2)?,
            not_null: row.get::<_, i32>(3)? != 0,
            default_value: row.get(4)?,
            is_pk: row.get::<_, i32>(5)? != 0,
        })
    })?;

    let mut cols = Vec::new();
    for row in rows {
        cols.push(row?);
    }

    Ok(cols)
}

/// Verifies that a list of required columns in a table have NOT NULL set.
pub fn validate_required_not_null_columns(
    conn: &Connection,
    table_name: &str,
    required_columns: &[&str],
) -> Result<Vec<NullabilityViolation>, rusqlite::Error> {
    let columns = inspect_table_columns(conn, table_name)?;
    let mut violations = Vec::new();

    for required in required_columns {
        if let Some(col) = columns.iter().find(|c| c.name.eq_ignore_ascii_case(required)) {
            if !col.not_null && !col.is_pk {
                violations.push(NullabilityViolation {
                    table_name: table_name.to_string(),
                    column_name: required.to_string(),
                    expected_not_null: true,
                });
            }
        }
    }

    Ok(violations)
}
