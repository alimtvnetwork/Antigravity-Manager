//! SQLite Table Row Size Forensics.
//! Measures average and maximum row payload byte sizes to identify table bloat.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRowSizeStats {
    pub table_name: String,
    pub row_count: usize,
    pub avg_row_bytes: f64,
    pub max_row_bytes: usize,
    pub total_estimated_bytes: usize,
}

/// Analyzes average and maximum byte consumption per row across all columns of a table.
pub fn analyze_table_row_sizes(conn: &Connection, table_name: &str) -> Result<TableRowSizeStats> {
    let clean_table = table_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");

    // 1. Get column names
    let pragma_sql = format!("PRAGMA table_info('{}')", clean_table);
    let mut stmt = conn.prepare(&pragma_sql)?;
    let col_names: Vec<String> = stmt
        .query_map([], |r| r.get(1))?
        .filter_map(Result::ok)
        .collect();

    if col_names.is_empty() {
        return Ok(TableRowSizeStats {
            table_name: table_name.to_string(),
            row_count: 0,
            avg_row_bytes: 0.0,
            max_row_bytes: 0,
            total_estimated_bytes: 0,
        });
    }

    // 2. Build row byte size expression: (length(COALESCE(c1, '')) + length(COALESCE(c2, '')) + ...)
    let length_parts: Vec<String> = col_names
        .iter()
        .map(|c| format!("length(CAST(COALESCE(\"{}\", '') AS BLOB))", c))
        .collect();
    let row_len_expr = length_parts.join(" + ");

    let query_sql = format!(
        "SELECT 
            COUNT(*), 
            AVG({}), 
            MAX({}), 
            SUM({}) 
         FROM \"{}\"",
        row_len_expr, row_len_expr, row_len_expr, clean_table
    );

    let (row_count, avg_bytes, max_bytes, total_bytes): (
        usize,
        Option<f64>,
        Option<usize>,
        Option<usize>,
    ) = conn.query_row(&query_sql, [], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?;

    Ok(TableRowSizeStats {
        table_name: table_name.to_string(),
        row_count,
        avg_row_bytes: avg_bytes.unwrap_or(0.0),
        max_row_bytes: max_bytes.unwrap_or(0),
        total_estimated_bytes: total_bytes.unwrap_or(0),
    })
}
