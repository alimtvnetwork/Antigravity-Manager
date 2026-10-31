//! Database Index Health Checker.
//! Inspects index coverage and query plan performance across critical Split-SQLite tables,
//! verifying that hot queries utilize indexes rather than performing full table scans.

use rusqlite::Connection;

/// Summary report of an index health check.
#[derive(Debug, Clone)]
pub struct IndexHealthReport {
    pub table_name: String,
    pub index_name: String,
    pub is_unique: bool,
    pub column_count: usize,
}

/// Query plan analysis outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryPlanEvaluation {
    IndexUsed { index_name: String },
    FullTableScanWarning { detail: String },
}

/// Lists all indexes defined on a specific table.
pub fn inspect_table_indexes(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<IndexHealthReport>, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("PRAGMA index_list({});", table_name))?;
    let mut reports = Vec::new();

    let rows = stmt.query_map([], |row| {
        let index_name: String = row.get(1)?;
        let is_unique: bool = row.get(2)?;
        Ok((index_name, is_unique))
    })?;

    for row in rows {
        let (index_name, is_unique) = row?;
        let col_count: usize = conn
            .query_row(&format!("PRAGMA index_info({});", index_name), [], |_| {
                Ok(())
            })
            .map(|_| 1)
            .unwrap_or(0);

        reports.push(IndexHealthReport {
            table_name: table_name.to_string(),
            index_name,
            is_unique,
            column_count: col_count,
        });
    }

    Ok(reports)
}

/// Evaluates a query plan using `EXPLAIN QUERY PLAN <sql>` to detect table scans.
pub fn evaluate_query_plan(
    conn: &Connection,
    query_sql: &str,
) -> Result<QueryPlanEvaluation, rusqlite::Error> {
    let explain_sql = format!("EXPLAIN QUERY PLAN {}", query_sql);
    let mut stmt = conn.prepare(&explain_sql)?;

    let mut plans = Vec::new();
    let rows = stmt.query_map([], |row| {
        let detail: String = row.get(3)?;
        Ok(detail)
    })?;

    for row in rows {
        plans.push(row?);
    }

    for detail in &plans {
        if detail.contains("SCAN")
            && !detail.contains("USING INDEX")
            && !detail.contains("USING COVERING INDEX")
        {
            return Ok(QueryPlanEvaluation::FullTableScanWarning {
                detail: detail.clone(),
            });
        }
        if detail.contains("USING INDEX") || detail.contains("USING COVERING INDEX") {
            let index_name = detail
                .split("USING INDEX ")
                .nth(1)
                .or_else(|| detail.split("USING COVERING INDEX ").nth(1))
                .unwrap_or("unknown")
                .split(' ')
                .next()
                .unwrap_or("unknown")
                .to_string();
            return Ok(QueryPlanEvaluation::IndexUsed { index_name });
        }
    }

    Ok(QueryPlanEvaluation::IndexUsed {
        index_name: "implicit/primary_key".to_string(),
    })
}
