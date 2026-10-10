use rusqlite::{params, Connection};

use super::*;

// ============================================================================
// IP 访问日志操作
// ============================================================================

/// 保存 IP 访问日志
pub fn save_ip_access_log(log: &IpAccessLog) -> Result<(), String> {
    let conn = connect_db()?;

    conn.execute(
        "INSERT INTO ip_access_logs (id, client_ip, timestamp, method, path, user_agent, status, duration, api_key_hash, blocked, block_reason, username)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            log.id,
            log.client_ip,
            log.timestamp,
            log.method,
            log.path,
            log.user_agent,
            log.status,
            log.duration,
            log.api_key_hash,
            log.blocked,
            log.block_reason,
            log.username,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取 IP 访问日志 (分页)
pub fn get_ip_access_logs(
    limit: usize,
    offset: usize,
    ip_filter: Option<&str>,
    blocked_only: bool,
) -> Result<Vec<IpAccessLog>, String> {
    let conn = connect_db()?;

    let sql = if blocked_only {
        if let Some(ip) = ip_filter {
            format!(
                "SELECT id, client_ip, timestamp, method, path, user_agent, status, duration, api_key_hash, blocked, block_reason, username
                 FROM ip_access_logs
                 WHERE blocked = 1 AND client_ip LIKE '%{}%'
                 ORDER BY timestamp DESC
                 LIMIT {} OFFSET {}",
                ip, limit, offset
            )
        } else {
            format!(
                "SELECT id, client_ip, timestamp, method, path, user_agent, status, duration, api_key_hash, blocked, block_reason, username
                 FROM ip_access_logs
                 WHERE blocked = 1
                 ORDER BY timestamp DESC
                 LIMIT {} OFFSET {}",
                limit, offset
            )
        }
    } else if let Some(ip) = ip_filter {
        format!(
            "SELECT id, client_ip, timestamp, method, path, user_agent, status, duration, api_key_hash, blocked, block_reason, username
             FROM ip_access_logs
             WHERE client_ip LIKE '%{}%'
             ORDER BY timestamp DESC
             LIMIT {} OFFSET {}",
            ip, limit, offset
        )
    } else {
        format!(
            "SELECT id, client_ip, timestamp, method, path, user_agent, status, duration, api_key_hash, blocked, block_reason, username
             FROM ip_access_logs
             ORDER BY timestamp DESC
             LIMIT {} OFFSET {}",
            limit, offset
        )
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let logs_iter = stmt
        .query_map([], |row| {
            Ok(IpAccessLog {
                id: row.get(0)?,
                client_ip: row.get(1)?,
                timestamp: row.get(2)?,
                method: row.get(3)?,
                path: row.get(4)?,
                user_agent: row.get(5)?,
                status: row.get(6)?,
                duration: row.get(7)?,
                api_key_hash: row.get(8)?,
                blocked: row.get::<_, i32>(9)? != 0,
                block_reason: row.get(10)?,
                username: row.get(11).unwrap_or(None),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for log in logs_iter {
        logs.push(log.map_err(|e| e.to_string())?);
    }
    Ok(logs)
}

/// 获取 IP 统计概览
pub fn get_ip_stats() -> Result<IpStats, String> {
    let conn = connect_db()?;

    let today_start = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();

    let (total_requests, unique_ips, blocked_count, today_requests): (u64, u64, u64, u64) = conn
        .query_row(
            "SELECT
                COALESCE(COUNT(*), 0) as total,
                COALESCE(COUNT(DISTINCT client_ip), 0) as unique_ips,
                COALESCE(SUM(CASE WHEN blocked = 1 THEN 1 ELSE 0 END), 0) as blocked,
                COALESCE(SUM(CASE WHEN timestamp >= ?1 THEN 1 ELSE 0 END), 0) as today
             FROM ip_access_logs",
            [today_start],
            |row| {
                Ok((
                    row.get::<_, Option<u64>>(0)?.unwrap_or(0),
                    row.get::<_, Option<u64>>(1)?.unwrap_or(0),
                    row.get::<_, Option<u64>>(2)?.unwrap_or(0),
                    row.get::<_, Option<u64>>(3)?.unwrap_or(0),
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let blacklist_count: u64 = conn
        .query_row(
            "SELECT COALESCE(COUNT(*), 0) FROM ip_blacklist",
            [],
            |row| Ok(row.get::<_, Option<u64>>(0)?.unwrap_or(0)),
        )
        .map_err(|e| e.to_string())?;

    let whitelist_count: u64 = conn
        .query_row(
            "SELECT COALESCE(COUNT(*), 0) FROM ip_whitelist",
            [],
            |row| Ok(row.get::<_, Option<u64>>(0)?.unwrap_or(0)),
        )
        .map_err(|e| e.to_string())?;

    Ok(IpStats {
        total_requests,
        unique_ips,
        blocked_count,
        today_requests,
        blacklist_count,
        whitelist_count,
    })
}

/// 获取 TOP N IP 访问排行
pub fn get_top_ips(limit: usize, hours: i64) -> Result<Vec<IpRanking>, String> {
    let conn = connect_db()?;

    let since = chrono::Utc::now().timestamp() - (hours * 3600);

    let mut stmt = conn
        .prepare(
            "SELECT client_ip, COUNT(*) as cnt, MAX(timestamp) as last_seen
             FROM ip_access_logs
             WHERE timestamp >= ?1
             GROUP BY client_ip
             ORDER BY cnt DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let rankings_iter = stmt
        .query_map([since, limit as i64], |row| {
            Ok(IpRanking {
                client_ip: row.get(0)?,
                request_count: row.get(1)?,
                last_seen: row.get(2)?,
                is_blocked: false, // 稍后填充
            })
        })
        .map_err(|e| e.to_string())?;

    let mut rankings = Vec::new();
    for r in rankings_iter {
        let mut ranking = r.map_err(|e| e.to_string())?;
        // 检查是否在黑名单中
        ranking.is_blocked = is_ip_in_blacklist(&ranking.client_ip)?;
        rankings.push(ranking);
    }

    Ok(rankings)
}

/// 清理旧的 IP 访问日志
pub fn cleanup_old_ip_logs(days: i64) -> Result<usize, String> {
    let conn = connect_db()?;

    let cutoff_timestamp = chrono::Utc::now().timestamp() - (days * 24 * 3600);

    let deleted = conn
        .execute(
            "DELETE FROM ip_access_logs WHERE timestamp < ?1",
            [cutoff_timestamp],
        )
        .map_err(|e| e.to_string())?;

    // VACUUM to reclaim space
    conn.execute("VACUUM", []).map_err(|e| e.to_string())?;

    Ok(deleted)
}

/// 清空所有 IP 访问日志
pub fn clear_ip_access_logs() -> Result<(), String> {
    let conn = connect_db()?;
    conn.execute("DELETE FROM ip_access_logs", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取 IP 访问日志总数
pub fn get_ip_access_logs_count(
    ip_filter: Option<&str>,
    blocked_only: bool,
) -> Result<u64, String> {
    let conn = connect_db()?;

    let sql = if blocked_only {
        if let Some(ip) = ip_filter {
            format!(
                "SELECT COUNT(*) FROM ip_access_logs WHERE blocked = 1 AND client_ip LIKE '%{}%'",
                ip
            )
        } else {
            "SELECT COUNT(*) FROM ip_access_logs WHERE blocked = 1".to_string()
        }
    } else if let Some(ip) = ip_filter {
        format!(
            "SELECT COUNT(*) FROM ip_access_logs WHERE client_ip LIKE '%{}%'",
            ip
        )
    } else {
        "SELECT COUNT(*) FROM ip_access_logs".to_string()
    };

    let count: u64 = conn
        .query_row(&sql, [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(count)
}
