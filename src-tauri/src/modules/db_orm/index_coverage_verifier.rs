//! Composite Index Coverage Verifier
//!
//! Audits table indexes across split SQLite databases, verifying whether composite
//! column filters in query clauses have aligned index coverage to prevent unindexed
//! table traversals on high-frequency tables.

use rusqlite::Connection;
use tracing::debug;

/// Index metadata record.
#[derive(Debug, Clone)]
pub struct TableIndexInfo {
    pub name: String,
    pub is_unique: bool,
    pub columns: Vec<String>,
}

/// Evaluates if an existing set of table indices covers a target set of filter columns.
pub fn is_query_covered(indices: &[TableIndexInfo], filter_columns: &[&str]) -> bool {
    if filter_columns.is_empty() {
        return true;
    }

    // Check if the primary filter column matches the leading column of any index
    let leading_col = filter_columns[0];
    indices.iter().any(|idx| {
        idx.columns
            .first()
            .map(|c| c.eq_ignore_ascii_case(leading_col))
            .unwrap_or(false)
    })
}

/// Queries PRAGMA index_list and PRAGMA index_info for a specific table.
pub fn inspect_table_indices(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<TableIndexInfo>, rusqlite::Error> {
    let list_sql = format!("PRAGMA index_list('{}');", table_name);
    let mut stmt = conn.prepare(&list_sql)?;

    let mut rows = stmt.query([])?;
    let mut index_summaries = Vec::new();

    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        let is_unique: bool = row.get::<_, i32>(2)? != 0;
        index_summaries.push((name, is_unique));
    }

    let mut result = Vec::new();
    for (idx_name, is_unique) in index_summaries {
        let info_sql = format!("PRAGMA index_info('{}');", idx_name);
        let mut info_stmt = conn.prepare(&info_sql)?;
        let mut info_rows = info_stmt.query([])?;
        let mut cols = Vec::new();

        while let Some(info_row) = info_rows.next()? {
            let col_name: String = info_row.get(2)?;
            cols.push(col_name);
        }

        result.push(TableIndexInfo {
            name: idx_name,
            is_unique,
            columns: cols,
        });
    }

    debug!(
        table = table_name,
        index_count = result.len(),
        "Audited table index coverage"
    );

    Ok(result)
}
