use crate::proxy::config::LogRetentionConfig;
use rusqlite::{params, Connection, OpenFlags};
use std::path::PathBuf;

use super::*;

pub fn get_thinking_records_count() -> Result<usize, String> {
    let conn = thinking_db()?;
    let count: usize = conn
        .query_row("SELECT COUNT(*) FROM thinking_records", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    Ok(count)
}

pub fn cleanup_old_thinking_records(days: i64) -> Result<usize, String> {
    let cutoff = chrono::Utc::now().timestamp_millis() - (days * 24 * 3600 * 1000);
    let deleted_tools = connect_db()
        .ok()
        .and_then(|conn| {
            conn.execute(
                "DELETE FROM tool_signatures WHERE created_at < ?1",
                params![cutoff],
            )
            .ok()
        })
        .unwrap_or(0);
    let conn = thinking_db()?;
    let deleted_records = conn
        .execute(
            "DELETE FROM thinking_records WHERE session_key IN (
                SELECT session_key FROM thinking_sessions WHERE last_accessed < ?1
             ) OR (
                session_key NOT IN (SELECT session_key FROM thinking_sessions)
                AND COALESCE(last_accessed, created_at) < ?1
             )",
            params![cutoff],
        )
        .unwrap_or(0);
    // Justification: auxiliary session cleanup; the deleted-record count drives the return
    crate::error::record_ignored(
        conn.execute(
            "DELETE FROM thinking_sessions WHERE last_accessed < ?1",
            params![cutoff],
        ),
        "delete stale thinking sessions",
    );
    Ok(deleted_tools + deleted_records)
}

pub fn apply_retention(policy: &LogRetentionConfig) -> Result<(usize, usize), String> {
    let _guard = LOG_WRITE_LOCK.lock().map_err(|e| e.to_string())?;
    let conn = connect_db()?;
    apply_retention_with_connection(&conn, policy)
}

pub(crate) fn apply_retention_with_connection(
    conn: &Connection,
    policy: &LogRetentionConfig,
) -> Result<(usize, usize), String> {
    // The request body is no longer forcibly cleared according to time. It is completely managed by the capacity limit and the sliding window of the number of rows, and the complete message is retained.
    let bodies_cleared = 0;

    // Note: The logic of deleting entire rows by day based on max_age_days has been removed and replaced by sliding window elimination of upper limit of number of items and upper limit of space.
    let mut rows_deleted = 0;
    if policy.max_rows > 0 {
        rows_deleted += conn.execute(
            "DELETE FROM request_logs WHERE id NOT IN (SELECT id FROM request_logs ORDER BY timestamp DESC LIMIT ?1)",
            [policy.max_rows],
        ).map_err(|e| e.to_string())?;
    }

    // Perform 30% sliding window tail elimination based on space upper limit
    let budget = policy.budget_bytes();
    if budget > 0 && disk_bytes(conn).unwrap_or(0) > budget {
        let (evicted, _) = evict_sliding_window(conn, budget)?;
        rows_deleted += evicted;
    }

    reclaim_space(conn)?;
    Ok((bodies_cleared, rows_deleted))
}

pub(crate) fn reclaim_space(conn: &Connection) -> Result<(), String> {
    let checkpoint = || -> Result<(), String> {
        let busy: i64 = conn
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if busy != 0 {
            tracing::warn!("proxy log checkpoint busy");
        }
        Ok(())
    };
    checkpoint()?;

    let auto_vacuum: i64 = conn
        .pragma_query_value(None, "auto_vacuum", |r| r.get(0))
        .unwrap_or(0);

    if auto_vacuum == 2 {
        // Draining all free pages incrementally in batches
        for _ in 0..50 {
            let free: u64 = conn
                .pragma_query_value(None, "freelist_count", |r| r.get(0))
                .unwrap_or(0);
            if free == 0 {
                break;
            }
            let step = free.min(1000);
            let mut vacuum = conn
                .prepare(&format!("PRAGMA incremental_vacuum({})", step))
                .map_err(|e| e.to_string())?;
            let mut pages = vacuum.query([]).map_err(|e| e.to_string())?;
            while pages.next().map_err(|e| e.to_string())?.is_some() {}
            drop(pages);
        }
    } else {
        // Non-incremental or legacy database: full VACUUM to shrink disk size
        // Justification: space reclamation only; rows were already deleted and the database stays usable
        crate::error::record_ignored(conn.execute("VACUUM", []), "vacuum proxy logs database");
    }

    checkpoint()
}

pub(crate) fn disk_bytes(conn: &Connection) -> Result<u64, String> {
    let path = conn.path().ok_or("proxy log database has no file path")?;
    [PathBuf::from(path), PathBuf::from(format!("{path}-wal"))]
        .iter()
        .try_fold(0u64, |total, path| match std::fs::metadata(path) {
            Ok(metadata) => Ok(total.saturating_add(metadata.len())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(total),
            Err(e) => Err(e.to_string()),
        })
}

pub fn get_proxy_db_disk_bytes() -> Result<u64, String> {
    let conn = connect_db()?;
    disk_bytes(&conn)
}

/// Sliding window tail elimination mechanism:
/// When the log database reaches or is about to exceed the budget limit, the tailmost (earliest) logs are automatically cleaned.
/// Squeeze out 30% of the maximum storage space at once (i.e., let the volume fall back to <= 70% of the budget),
/// And record the log, and then return the number of cleaned records and the number of released bytes.
pub fn evict_sliding_window(conn: &Connection, budget: u64) -> Result<(usize, u64), String> {
    if budget == 0 {
        return Ok((0, 0));
    }
    let before_bytes = disk_bytes(conn)?;
    // Squeeze out 30% of the maximum space at a time (i.e. target retention <= 70% of the maximum limit)
    let evict_quota = (budget as f64 * 0.30) as u64;
    let target_bytes = budget.saturating_sub(evict_quota);

    if before_bytes <= target_bytes {
        return Ok((0, 0));
    }

    let mut total_deleted: usize = 0;
    // Loop cleans from the end of the batch (earliest record, timestamp ASC)
    for _ in 0..100 {
        let deleted = conn
            .execute(
                "DELETE FROM request_logs WHERE id IN (
                SELECT id FROM request_logs ORDER BY timestamp ASC LIMIT 250
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        if deleted == 0 {
            break;
        }
        total_deleted += deleted;
        reclaim_space(conn)?;

        let current_bytes = disk_bytes(conn)?;
        if current_bytes <= target_bytes {
            break;
        }
    }

    let after_bytes = disk_bytes(conn)?;
    let freed_bytes = before_bytes.saturating_sub(after_bytes);

    if total_deleted > 0 {
        tracing::info!(
            "[ProxyLog Sliding Window] Disk budget reached ({:.2} GB limit). Evicted {} tail records, freed {:.2} MB (target 30% quota: {:.2} MB). Current size: {:.2} MB.",
            budget as f64 / 1_073_741_824.0,
            total_deleted,
            freed_bytes as f64 / 1_048_576.0,
            evict_quota as f64 / 1_048_576.0,
            after_bytes as f64 / 1_048_576.0
        );
    }

    Ok((total_deleted, freed_bytes))
}

pub(crate) fn projected_bytes(conn: &Connection, log_bytes: u64) -> Result<u64, String> {
    let free: u64 = conn
        .pragma_query_value(None, "freelist_count", |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let page_size: u64 = conn
        .pragma_query_value(None, "page_size", |r| r.get(0))
        .map_err(|e| e.to_string())?;
    // Free pages avoid database growth, but still need WAL frames during the transaction.
    Ok(disk_bytes(conn)?
        .saturating_add(log_bytes.saturating_mul(2))
        .saturating_add(log_bytes.saturating_sub(free.saturating_mul(page_size)))
        .saturating_add(64 * 1024))
}

pub(crate) fn make_room(conn: &Connection, budget: u64, log_bytes: u64) -> Result<(), String> {
    if budget == 0 {
        return Err("proxy log disk budget is 0".to_string());
    }
    if projected_bytes(conn, log_bytes)? <= budget {
        return Ok(());
    }
    reclaim_space(conn)?;
    if projected_bytes(conn, log_bytes)? <= budget {
        return Ok(());
    }

    let auto_vacuum: i64 = conn
        .pragma_query_value(None, "auto_vacuum", |r| r.get(0))
        .unwrap_or(0);
    // Legacy files cannot shrink: even reusing all free pages still needs WAL headroom.
    if auto_vacuum == 0
        && disk_bytes(conn)?
            .saturating_add(log_bytes.saturating_mul(2))
            .saturating_add(64 * 1024)
            > budget
    {
        return Err("legacy proxy log database cannot shrink within budget".to_string());
    }

    // Prioritize triggering the 30% sliding window mechanism to clean up the last historical logs
    let (evicted, _) = evict_sliding_window(conn, budget)?;
    if evicted > 0 {
        reclaim_space(conn)?;
    }

    if projected_bytes(conn, log_bytes)? <= budget {
        return Ok(());
    }

    let target = budget.saturating_mul(7) / 10;
    // Bounded work per write, oldest bodies first, then oldest summaries. No full-body reads.
    for _ in 0..8 {
        let before = projected_bytes(conn, log_bytes)?;
        let cleared = conn.execute(
            "UPDATE request_logs SET request_body = NULL, upstream_request_body = NULL, response_body = NULL,
             request_headers = NULL, upstream_request_headers = NULL, response_headers = NULL WHERE id IN
             (SELECT id FROM request_logs WHERE request_body IS NOT NULL OR upstream_request_body IS NOT NULL OR response_body IS NOT NULL ORDER BY timestamp ASC LIMIT 64)", []
        ).map_err(|e| e.to_string())?;
        if cleared == 0 {
            conn.execute("DELETE FROM request_logs WHERE id IN (SELECT id FROM request_logs ORDER BY timestamp ASC LIMIT 64)", [])
                .map_err(|e| e.to_string())?;
        }
        reclaim_space(conn)?;
        let after = projected_bytes(conn, log_bytes)?;
        if after <= target {
            return Ok(());
        }
        if after >= before {
            break;
        }
    }
    if projected_bytes(conn, log_bytes)? <= budget {
        Ok(())
    } else {
        Err("proxy log disk budget exhausted".to_string())
    }
}
