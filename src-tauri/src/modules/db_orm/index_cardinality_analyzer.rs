// Table Index Cardinality Analyzer (Imp-496)
// Inspects table and index distribution stats to identify low-cardinality indexes
// that may impair query plan efficiency.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexCardinalityMetric {
    pub index_name: String,
    pub table_name: String,
    pub unique_entry_count: i64,
    pub total_row_count: i64,
    pub selectivity_ratio: f64,
    pub is_low_selectivity: bool,
}

pub struct IndexCardinalityAnalyzer;

impl IndexCardinalityAnalyzer {
    /// Evaluates selectivity of a specific indexed column.
    pub fn evaluate_column_cardinality(
        conn: &Connection,
        table: &str,
        column: &str,
        index_name: &str,
    ) -> Result<IndexCardinalityMetric, rusqlite::Error> {
        let count_query = format!("SELECT COUNT(*), COUNT(DISTINCT {}) FROM {};", column, table);
        let mut stmt = conn.prepare(&count_query)?;
        
        let (total_rows, distinct_count): (i64, i64) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let selectivity = if total_rows > 0 {
            distinct_count as f64 / total_rows as f64
        } else {
            1.0
        };

        // If distinct count is less than 5% of rows and total rows > 100, flag as low selectivity
        let is_low_selectivity = total_rows > 100 && selectivity < 0.05;

        Ok(IndexCardinalityMetric {
            index_name: index_name.to_string(),
            table_name: table.to_string(),
            unique_entry_count: distinct_count,
            total_row_count: total_rows,
            selectivity_ratio: selectivity,
            is_low_selectivity,
        })
    }
}
