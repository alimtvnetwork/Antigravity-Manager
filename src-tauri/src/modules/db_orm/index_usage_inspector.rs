//! Query Plan Index Coverage Inspector
//! Introspects EXPLAIN QUERY PLAN output to verify that queries utilize indexes
//! rather than triggering slow full-table scans.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInspectionResult {
    pub query: String,
    pub is_indexed: bool,
    pub has_table_scan: bool,
    pub plan_steps: Vec<String>,
}

pub struct IndexUsageInspector;

impl IndexUsageInspector {
    /// Executes EXPLAIN QUERY PLAN on the given SQL statement and inspects index usage.
    pub fn inspect_query(
        conn: &Connection,
        query: &str,
    ) -> Result<IndexInspectionResult, rusqlite::Error> {
        let explain_sql = format!("EXPLAIN QUERY PLAN {}", query);
        let mut stmt = conn.prepare(&explain_sql)?;

        let mut plan_steps = Vec::new();
        let mut has_table_scan = false;
        let mut has_index_search = false;

        let rows = stmt.query_map([], |row| {
            // Columns: id, parent, notused, detail
            let detail: String = row.get(3)?;
            Ok(detail)
        })?;

        for row in rows {
            let detail = row?;
            let detail_upper = detail.to_ascii_uppercase();

            if detail_upper.contains("SCAN TABLE") && !detail_upper.contains("USING COVERING INDEX")
            {
                has_table_scan = true;
            }
            if detail_upper.contains("USING INDEX")
                || detail_upper.contains("USING COVERING INDEX")
                || detail_upper.contains("SEARCH TABLE")
            {
                has_index_search = true;
            }

            plan_steps.push(detail);
        }

        let is_indexed = has_index_search && !has_table_scan;

        Ok(IndexInspectionResult {
            query: query.to_string(),
            is_indexed,
            has_table_scan,
            plan_steps,
        })
    }
}
