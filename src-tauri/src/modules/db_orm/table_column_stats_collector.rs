// Split Table Column Statistics Collector (Imp-526)
// Gathers column distribution metrics, row counts, and null ratios across split
// database tables to assist query planning and detect schema integrity anomalies.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub null_count: i64,
    pub null_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnStatsReport {
    pub table_name: String,
    pub total_rows: i64,
    pub columns: Vec<ColumnStats>,
}

pub struct TableColumnStatsCollector;

impl TableColumnStatsCollector {
    /// Collects basic column distribution statistics for a given table.
    pub fn collect_table_stats(conn: &Connection, table_name: &str) -> Result<TableColumnStatsReport, rusqlite::Error> {
        let count_query = format!("SELECT COUNT(*) FROM {};", table_name);
        let mut count_stmt = conn.prepare(&count_query)?;
        let total_rows: i64 = count_stmt.query_row([], |row| row.get(0))?;

        let info_query = format!("PRAGMA table_info({});", table_name);
        let mut info_stmt = conn.prepare(&info_query)?;
        let column_rows = info_stmt.query_map([], |row| {
            let col_name: String = row.get(1)?;
            let col_type: String = row.get(2)?;
            let not_null: i32 = row.get(3)?;
            Ok((col_name, col_type, not_null == 0))
        })?;

        let mut columns = Vec::new();
        for col_res in column_rows {
            let (name, col_type, nullable) = col_res?;
            let null_count = if nullable && total_rows > 0 {
                let null_query = format!("SELECT COUNT(*) FROM {} WHERE {} IS NULL;", table_name, name);
                let mut null_stmt = conn.prepare(&null_query)?;
                null_stmt.query_row([], |row| row.get(0)).unwrap_or(0)
            } else {
                0
            };

            let null_ratio = if total_rows > 0 {
                null_count as f64 / total_rows as f64
            } else {
                0.0
            };

            columns.push(ColumnStats {
                column_name: name,
                data_type: col_type,
                is_nullable: nullable,
                null_count,
                null_ratio,
            });
        }

        Ok(TableColumnStatsReport {
            table_name: table_name.to_string(),
            total_rows,
            columns,
        })
    }
}
