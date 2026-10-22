//! Query Explain Query Plan Cost Estimator
//!
//! Evaluates SQL query execution plans via `EXPLAIN QUERY PLAN`, detecting unindexed
//! full-table scans (`SCAN TABLE`), temporary B-trees, and verifying efficient index usage
//! (`SEARCH TABLE ... USING INDEX`) to protect against performance regressions.

use rusqlite::Connection;
use tracing::{debug, warn};

/// Assessment outcome of an analyzed query plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryPlanAssessment {
    /// Optimal execution plan utilizing indices.
    Efficient,
    /// Query requires an unindexed full table scan.
    FullTableScan { table: String },
    /// Query requires creation of a temporary B-tree for sorting or grouping.
    TemporaryBTree,
}

/// Single record from EXPLAIN QUERY PLAN.
#[derive(Debug, Clone)]
pub struct QueryPlanStep {
    pub id: i32,
    pub parent: i32,
    pub notused: i32,
    pub detail: String,
}

/// Runs EXPLAIN QUERY PLAN on a query and evaluates its execution characteristics.
pub fn estimate_query_cost(
    conn: &Connection,
    sql: &str,
) -> Result<Vec<QueryPlanAssessment>, rusqlite::Error> {
    let explain_sql = format!("EXPLAIN QUERY PLAN {}", sql);
    let mut stmt = conn.prepare(&explain_sql)?;

    let mut rows = stmt.query([])?;
    let mut assessments = Vec::new();

    while let Some(row) = rows.next()? {
        let detail: String = row.get(3)?;

        if detail.contains("SCAN TABLE") {
            // Extract table name after "SCAN TABLE "
            let table = detail
                .strip_prefix("SCAN TABLE ")
                .unwrap_or(&detail)
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string();

            warn!(table = %table, detail = %detail, "Full table scan detected in query plan");
            assessments.push(QueryPlanAssessment::FullTableScan { table });
        } else if detail.contains("USE TEMP B-TREE") {
            warn!(detail = %detail, "Temporary B-tree required for query plan");
            assessments.push(QueryPlanAssessment::TemporaryBTree);
        }
    }

    if assessments.is_empty() {
        debug!(sql = %sql, "Query execution plan evaluated as efficient");
        assessments.push(QueryPlanAssessment::Efficient);
    }

    Ok(assessments)
}
