//! Split database logical relationship and orphan record checker.
//! Verifies cross-database relational integrity across independently split
//! SQLite database files (e.g. prompt_backups vs repo_prompts).

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitDbOrphanReport {
    pub total_records_checked: usize,
    pub orphaned_records_count: usize,
    pub orphaned_ids: Vec<String>,
}

pub struct SplitDbOrphanChecker;

impl SplitDbOrphanChecker {
    /// Identifies prompt backup IDs whose parent conversation_id is not present in the active valid set.
    pub fn check_orphaned_prompt_backups(
        backup_conn: &Connection,
        valid_conversation_ids: &HashSet<String>,
        check_limit: usize,
    ) -> Result<SplitDbOrphanReport, AppError> {
        let sql = format!(
            "SELECT id, conversation_id FROM prompt_backups ORDER BY rowid DESC LIMIT {}",
            check_limit
        );

        let mut stmt = backup_conn
            .prepare(&sql)
            .map_err(AppError::Database)?;

        let mut rows = stmt
            .query([])
            .map_err(AppError::Database)?;

        let mut total = 0usize;
        let mut orphans = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(AppError::Database)?
        {
            let id: String = row.get(0).unwrap_or_default();
            let conv_id: Option<String> = row.get(1).ok();

            total += 1;

            if let Some(cid) = conv_id {
                if !valid_conversation_ids.is_empty() && !valid_conversation_ids.contains(&cid) {
                    orphans.push(id);
                }
            }
        }

        let count = orphans.len();
        Ok(SplitDbOrphanReport {
            total_records_checked: total,
            orphaned_records_count: count,
            orphaned_ids: orphans,
        })
    }
}
