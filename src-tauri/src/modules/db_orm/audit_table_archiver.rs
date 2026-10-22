//! Prompt Audit Table Auto-Archive Sentinel.
//! Scans historical active prompt records older than the archival threshold,
//! preparing batches for atomic archival into split `task_history.db` partitions.

use rusqlite::Connection;

/// Summary metrics of an archival pass.
#[derive(Debug, Clone, Default)]
pub struct ArchivalBatchSummary {
    pub candidate_records_found: usize,
    pub records_archived: usize,
    pub records_purged_from_source: usize,
}

/// Identifies candidate prompt IDs eligible for archival given a cutoff timestamp in seconds.
pub fn find_archivable_prompt_ids(
    conn: &Connection,
    cutoff_timestamp_secs: i64,
    batch_limit: usize,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id FROM active_prompts \
         WHERE status IN ('completed', 'cancelled', 'failed') \
         AND updated_at < ?1 \
         ORDER BY created_at ASC, id ASC \
         LIMIT ?2;",
    )?;

    let rows = stmt.query_map([cutoff_timestamp_secs, batch_limit as i64], |row| {
        row.get::<_, String>(0)
    })?;

    let mut ids = Vec::new();
    for row in rows {
        ids.push(row?);
    }

    Ok(ids)
}

/// Purges archived records from the source `active_prompts` table in chunks.
pub fn purge_archived_prompt_batch(
    conn: &Connection,
    prompt_ids: &[String],
) -> Result<usize, rusqlite::Error> {
    if prompt_ids.is_empty() {
        return Ok(0);
    }

    let mut deleted_total = 0;
    // Process in sub-batches of 100 to stay well under SQLite parameter limits
    for chunk in prompt_ids.chunks(100) {
        let placeholders: Vec<String> = (1..=chunk.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "DELETE FROM active_prompts WHERE id IN ({});",
            placeholders.join(", ")
        );
        let params: Vec<&dyn rusqlite::ToSql> = chunk.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        deleted_total += conn.execute(&sql, params.as_slice())?;
    }

    Ok(deleted_total)
}
