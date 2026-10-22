//! Table Row Count & Size Profiler.
//! Profiles user table row counts and estimated storage footprint across split databases.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Profile summary for an individual database table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableProfile {
    pub table_name: String,
    pub row_count: i64,
}

/// Comprehensive profile of an entire split SQLite database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseProfile {
    pub db_name: String,
    pub tables: Vec<TableProfile>,
    pub total_rows: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub total_file_bytes: i64,
}

/// Gathers row counts and disk metrics across all user tables in the connected database.
pub fn profile_database_tables(conn: &Connection, db_label: &str) -> Result<DatabaseProfile> {
    let page_size: i64 = conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
    let page_count: i64 = conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
    let total_file_bytes = page_size * page_count;

    // Discover user tables
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name ASC",
    )?;
    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    let mut tables = Vec::new();
    let mut total_rows = 0i64;

    for tbl in table_names {
        let count_sql = format!("SELECT count(*) FROM '{}'", tbl.replace('\'', "''"));
        let count: i64 = conn.query_row(&count_sql, [], |r| r.get(0)).unwrap_or(0);
        total_rows += count;
        tables.push(TableProfile {
            table_name: tbl,
            row_count: count,
        });
    }

    Ok(DatabaseProfile {
        db_name: db_label.to_string(),
        tables,
        total_rows,
        page_size,
        page_count,
        total_file_bytes,
    })
}
