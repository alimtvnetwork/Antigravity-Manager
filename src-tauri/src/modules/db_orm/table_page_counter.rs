// Table Page Space Allocation Counter (Imp-498)
// Computes table storage allocations across split databases to detect
// anomalies or unbounded table bloat early.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSpaceMetric {
    pub table_name: String,
    pub row_count: i64,
    pub estimated_size_bytes: i64,
}

pub struct TablePageCounter;

impl TablePageCounter {
    /// Queries the row count for a specified table and estimates size assuming default page size.
    pub fn inspect_table_space(
        conn: &Connection,
        table_name: &str,
    ) -> Result<TableSpaceMetric, rusqlite::Error> {
        let count_query = format!("SELECT COUNT(*) FROM {};", table_name);
        let mut stmt = conn.prepare(&count_query)?;
        let row_count: i64 = stmt.query_row([], |row| row.get(0))?;

        // Query database page size
        let mut page_size_stmt = conn.prepare("PRAGMA page_size;")?;
        let page_size: i64 = page_size_stmt
            .query_row([], |row| row.get(0))
            .unwrap_or(4096);

        // Approximate 128 bytes average per row for lightweight metadata
        let estimated_size_bytes = (row_count * 128).max(page_size);

        Ok(TableSpaceMetric {
            table_name: table_name.to_string(),
            row_count,
            estimated_size_bytes,
        })
    }
}
