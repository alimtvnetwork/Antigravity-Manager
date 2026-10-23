//! Table column default value validator.
//! Verifies table schema constraints and ensures that nullable and boolean columns
//! maintain appropriate DEFAULT value specifications across split databases.

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefaultInfo {
    pub cid: i64,
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnAudit {
    pub table_name: String,
    pub total_columns: usize,
    pub columns: Vec<ColumnDefaultInfo>,
    pub columns_missing_defaults: Vec<String>,
}

pub struct ColumnDefaultValidator;

impl ColumnDefaultValidator {
    /// Validates column definitions for the specified table.
    pub fn audit_table(conn: &Connection, table_name: &str) -> Result<TableColumnAudit, AppError> {
        let sql = format!("PRAGMA table_info('{}');", table_name);
        let mut stmt = conn.prepare(&sql).map_err(AppError::Database)?;

        let mut rows = stmt.query([]).map_err(AppError::Database)?;

        let mut columns = Vec::new();
        let mut missing_defaults = Vec::new();

        while let Some(row) = rows.next().map_err(AppError::Database)? {
            let cid: i64 = row.get(0).unwrap_or(0);
            let name: String = row.get(1).unwrap_or_default();
            let data_type: String = row.get(2).unwrap_or_default();
            let not_null: bool = row.get::<_, i32>(3).unwrap_or(0) != 0;
            let dflt_value: Option<String> = row.get(4).ok();

            // Highlight columns that are not null and lack a default value (excluding primary keys)
            let is_pk: bool = row.get::<_, i32>(5).unwrap_or(0) != 0;
            if not_null && dflt_value.is_none() && !is_pk {
                missing_defaults.push(name.clone());
            }

            columns.push(ColumnDefaultInfo {
                cid,
                name,
                data_type,
                not_null,
                default_value: dflt_value,
            });
        }

        let total = columns.len();
        Ok(TableColumnAudit {
            table_name: table_name.to_string(),
            total_columns: total,
            columns,
            columns_missing_defaults: missing_defaults,
        })
    }
}
