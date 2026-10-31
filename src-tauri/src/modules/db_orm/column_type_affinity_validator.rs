//! Column Type Affinity & Constraint Validator
//!
//! Validates declared column definitions and type affinities across split SQLite
//! database tables, verifying that columns conform to standard SQLite affinities
//! (INTEGER, TEXT, BLOB, REAL, NUMERIC) and checking primary key constraints.

use rusqlite::Connection;
use tracing::debug;

/// Recognized SQLite type affinities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteAffinity {
    Integer,
    Text,
    Blob,
    Real,
    Numeric,
}

impl SqliteAffinity {
    /// Determines the SQLite type affinity from a declared column data type string.
    /// Rules follow Section 3.1 of the SQLite specification.
    pub fn from_declared_type(declared: &str) -> Self {
        let upper = declared.to_ascii_uppercase();

        if upper.contains("INT") {
            SqliteAffinity::Integer
        } else if upper.contains("CHAR") || upper.contains("CLOB") || upper.contains("TEXT") {
            SqliteAffinity::Text
        } else if upper.contains("BLOB") || upper.is_empty() {
            SqliteAffinity::Blob
        } else if upper.contains("REAL") || upper.contains("FLOA") || upper.contains("DOUB") {
            SqliteAffinity::Real
        } else {
            SqliteAffinity::Numeric
        }
    }
}

/// Column metadata extracted from PRAGMA table_info.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub cid: i32,
    pub name: String,
    pub declared_type: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
    pub affinity: SqliteAffinity,
}

/// Inspects and validates column definitions for a given table.
pub fn inspect_table_columns(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<ColumnInfo>, rusqlite::Error> {
    let sql = format!("PRAGMA table_info('{}');", table_name);
    let mut stmt = conn.prepare(&sql)?;

    let mut rows = stmt.query([])?;
    let mut columns = Vec::new();

    while let Some(row) = rows.next()? {
        let cid: i32 = row.get(0)?;
        let name: String = row.get(1)?;
        let declared_type: String = row.get(2)?;
        let not_null: bool = row.get::<_, i32>(3)? != 0;
        let default_value: Option<String> = row.get(4)?;
        let primary_key: bool = row.get::<_, i32>(5)? != 0;
        let affinity = SqliteAffinity::from_declared_type(&declared_type);

        columns.push(ColumnInfo {
            cid,
            name,
            declared_type,
            not_null,
            default_value,
            primary_key,
            affinity,
        });
    }

    debug!(
        table = table_name,
        column_count = columns.len(),
        "Inspected table columns and type affinities"
    );

    Ok(columns)
}
