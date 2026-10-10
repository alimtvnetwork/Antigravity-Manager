use rusqlite::{params, Connection};

use super::*;

/// Record token usage from a request
pub fn record_usage(
    account_email: &str,
    model: &str,
    input_tokens: u32,
    output_tokens: u32,
    cached_tokens: u32,
) -> Result<(), String> {
    let conn = connect_db()?;
    let timestamp = chrono::Local::now().timestamp();
    let total_tokens = input_tokens + output_tokens;

    // Insert into raw usage table
    conn.execute(
        "INSERT INTO token_usage (timestamp, account_email, model, input_tokens, output_tokens, cached_tokens, total_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![timestamp, account_email, model, input_tokens, output_tokens, cached_tokens, total_tokens],
    ).map_err(|e| e.to_string())?;

    let hour_bucket = chrono::Local::now().format("%Y-%m-%d %H:00").to_string();
    conn.execute(
        "INSERT INTO token_stats_hourly (hour_bucket, account_email, total_input_tokens, total_output_tokens, total_cached_tokens, total_tokens, request_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)
         ON CONFLICT(hour_bucket, account_email) DO UPDATE SET
            total_input_tokens = total_input_tokens + ?3,
            total_output_tokens = total_output_tokens + ?4,
            total_cached_tokens = total_cached_tokens + ?5,
            total_tokens = total_tokens + ?6,
            request_count = request_count + 1",
        params![hour_bucket, account_email, input_tokens, output_tokens, cached_tokens, total_tokens],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Attach current weekly usage without loading raw history or persisting derived totals.
pub(crate) fn populate_weekly_usage(accounts: &mut [crate::models::Account]) -> Result<(), String> {
    if !accounts.iter().any(|account| {
        account
            .quota
            .as_ref()
            .is_some_and(|q| q.quota_groups.is_some())
    }) {
        return Ok(());
    }
    let conn = connect_db()?;
    populate_weekly_usage_with_conn(&conn, accounts, chrono::Utc::now().timestamp())
}

pub(crate) fn populate_weekly_usage_with_conn(
    conn: &Connection,
    accounts: &mut [crate::models::Account],
    now: i64,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(SUM(input_tokens + output_tokens), 0) FROM token_usage
         WHERE account_email = ?1 AND timestamp >= ?2 AND timestamp < ?3 AND timestamp <= ?4
         AND ((?5 = 0 AND model LIKE 'gemini%')
           OR (?5 = 1 AND (model LIKE 'claude%' OR model LIKE 'gpt%')))",
        )
        .map_err(|e| e.to_string())?;
    for account in accounts {
        let Some(groups) = account
            .quota
            .as_mut()
            .and_then(|quota| quota.quota_groups.as_mut())
        else {
            continue;
        };
        for group in groups {
            let name = group.display_name.to_lowercase();
            for bucket in &mut group.buckets {
                bucket.cycle_tokens = None;
                let Some((start, end)) = bucket.weekly_cycle_bounds(now) else {
                    continue;
                };
                let id = bucket.bucket_id.to_lowercase();
                let third_party =
                    name.contains("claude") || name.contains("gpt") || id.contains("3p");
                if !third_party && !name.contains("gemini") && !id.contains("gemini") {
                    continue;
                }
                bucket.cycle_tokens = Some(
                    stmt.query_row(
                        params![account.email, start, end, now, third_party],
                        |row| row.get(0),
                    )
                    .map_err(|e| e.to_string())?,
                );
            }
        }
    }
    Ok(())
}
