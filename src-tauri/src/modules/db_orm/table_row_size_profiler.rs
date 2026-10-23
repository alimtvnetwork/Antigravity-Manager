//! Table row size profiler.
//! Samples and computes byte distributions across table records to identify
//! oversized payloads requiring compaction or external blob offloading.

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRowSizeStats {
    pub table_name: String,
    pub sampled_rows: usize,
    pub avg_row_bytes: usize,
    pub max_row_bytes: usize,
    pub oversized_rows_gt_64kb: usize,
}

pub struct TableRowSizeProfiler;

impl TableRowSizeProfiler {
    /// Profiles row sizes for the specified table and text/blob column.
    pub fn profile_column(
        conn: &Connection,
        table_name: &str,
        payload_column: &str,
        sample_limit: usize,
    ) -> Result<TableRowSizeStats, AppError> {
        let sql = format!(
            "SELECT length(CAST({} AS BLOB)) FROM {} ORDER BY rowid DESC LIMIT {}",
            payload_column, table_name, sample_limit
        );

        let mut stmt = conn.prepare(&sql).map_err(AppError::Database)?;

        let mut rows = stmt.query([]).map_err(AppError::Database)?;

        let mut total_bytes = 0usize;
        let mut max_bytes = 0usize;
        let mut sampled = 0usize;
        let mut oversized = 0usize;

        while let Some(row) = rows.next().map_err(AppError::Database)? {
            let bytes: usize = row.get(0).unwrap_or(0);
            total_bytes += bytes;
            if bytes > max_bytes {
                max_bytes = bytes;
            }
            if bytes > 65536 {
                oversized += 1;
            }
            sampled += 1;
        }

        let avg = if sampled > 0 {
            total_bytes / sampled
        } else {
            0
        };

        Ok(TableRowSizeStats {
            table_name: table_name.to_string(),
            sampled_rows: sampled,
            avg_row_bytes: avg,
            max_row_bytes: max_bytes,
            oversized_rows_gt_64kb: oversized,
        })
    }
}
