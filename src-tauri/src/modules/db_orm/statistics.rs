//! Instant Column Statistics Analyzer for Split-SQLite databases.
//! Computes aggregate distribution metrics, distinct counts, nullity, and value ranges.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStatistics {
    pub table_name: String,
    pub column_name: String,
    pub total_rows: usize,
    pub non_null_count: usize,
    pub null_count: usize,
    pub distinct_count: usize,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub avg_value: Option<f64>,
}

/// Computes complete distribution statistics for a specific column in a SQLite table.
pub fn analyze_column_statistics(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<ColumnStatistics> {
    let clean_table = table_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let clean_column = column_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");

    // 1. Total rows
    let total_sql = format!("SELECT COUNT(*) FROM \"{}\"", clean_table);
    let total_rows: usize = conn.query_row(&total_sql, [], |r| r.get(0))?;

    // 2. Non-null and distinct counts
    let agg_sql = format!(
        "SELECT 
            COUNT(\"{}\"), 
            COUNT(DISTINCT \"{}\"), 
            MIN(CAST(\"{}\" AS TEXT)), 
            MAX(CAST(\"{}\" AS TEXT)), 
            AVG(CAST(\"{}\" AS REAL)) 
         FROM \"{}\"",
        clean_column, clean_column, clean_column, clean_column, clean_column, clean_table
    );

    let (non_null_count, distinct_count, min_value, max_value, avg_value): (
        usize,
        usize,
        Option<String>,
        Option<String>,
        Option<f64>,
    ) = conn.query_row(&agg_sql, [], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    })?;

    let null_count = total_rows.saturating_sub(non_null_count);

    Ok(ColumnStatistics {
        table_name: table_name.to_string(),
        column_name: column_name.to_string(),
        total_rows,
        non_null_count,
        null_count,
        distinct_count,
        min_value,
        max_value,
        avg_value,
    })
}
