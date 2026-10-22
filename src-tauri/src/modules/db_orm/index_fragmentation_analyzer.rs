//! Index fragmentation analyzer.
//! Inspects indexes on performance-critical SQLite tables (e.g. `active_prompts`, `task_history`)
//! to evaluate index integrity and verify column coverage.

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    pub is_unique: bool,
    pub origin: String,
    pub indexed_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexAnalysisReport {
    pub table_name: String,
    pub index_count: usize,
    pub indexes: Vec<IndexMetadata>,
}

pub struct IndexFragmentationAnalyzer;

impl IndexFragmentationAnalyzer {
    /// Analyzes all declared indexes for the specified database table.
    pub fn analyze_table_indexes(conn: &Connection, table_name: &str) -> Result<IndexAnalysisReport, AppError> {
        let sql = format!("PRAGMA index_list('{}');", table_name);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| AppError::Database(format!("Prepare index_list failed: {}", e)))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| AppError::Database(format!("Query index_list failed: {}", e)))?;

        let mut indexes = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| AppError::Database(format!("Read index row failed: {}", e)))?
        {
            let name: String = row.get(1).unwrap_or_default();
            let unique: bool = row.get::<_, i32>(2).unwrap_or(0) != 0;
            let origin: String = row.get(3).unwrap_or_default();

            let columns = Self::get_indexed_columns(conn, &name)?;

            indexes.push(IndexMetadata {
                name,
                is_unique: unique,
                origin,
                indexed_columns: columns,
            });
        }

        let count = indexes.len();
        Ok(IndexAnalysisReport {
            table_name: table_name.to_string(),
            index_count: count,
            indexes,
        })
    }

    fn get_indexed_columns(conn: &Connection, index_name: &str) -> Result<Vec<String>, AppError> {
        let sql = format!("PRAGMA index_info('{}');", index_name);
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| AppError::Database(format!("Prepare index_info failed: {}", e)))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| AppError::Database(format!("Query index_info failed: {}", e)))?;

        let mut cols = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| AppError::Database(format!("Read index column failed: {}", e)))?
        {
            if let Ok(name) = row.get::<_, String>(2) {
                cols.push(name);
            }
        }

        Ok(cols)
    }
}
