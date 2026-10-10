use rusqlite::{params, Connection};

use super::*;

// ============================================================================
// 白名单操作
// ============================================================================

/// 添加 IP 到白名单
pub fn add_to_whitelist(
    ip_pattern: &str,
    description: Option<&str>,
) -> Result<IpWhitelistEntry, String> {
    let conn = connect_db()?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO ip_whitelist (id, ip_pattern, description, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, ip_pattern, description, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(IpWhitelistEntry {
        id,
        ip_pattern: ip_pattern.to_string(),
        description: description.map(|s| s.to_string()),
        created_at: now,
    })
}

/// 从白名单移除
pub fn remove_from_whitelist(id: &str) -> Result<(), String> {
    let conn = connect_db()?;

    conn.execute("DELETE FROM ip_whitelist WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取白名单列表
pub fn get_whitelist() -> Result<Vec<IpWhitelistEntry>, String> {
    let conn = connect_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, ip_pattern, description, created_at
             FROM ip_whitelist
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let entries_iter = stmt
        .query_map([], |row| {
            Ok(IpWhitelistEntry {
                id: row.get(0)?,
                ip_pattern: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for e in entries_iter {
        entries.push(e.map_err(|e| e.to_string())?);
    }
    Ok(entries)
}

/// 检查 IP 是否在白名单中
pub fn is_ip_in_whitelist(ip: &str) -> Result<bool, String> {
    let conn = connect_db()?;

    // 精确匹配
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ip_whitelist WHERE ip_pattern = ?1",
            [ip],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Ok(true);
    }

    // CIDR 匹配与 IP 等价匹配
    let entries = get_whitelist()?;
    for entry in entries {
        if entry.ip_pattern.contains('/') {
            if cidr_match(ip, &entry.ip_pattern) {
                return Ok(true);
            }
        } else if let (Ok(client_addr), Ok(entry_addr)) = (
            ip.trim()
                .trim_matches('[')
                .trim_matches(']')
                .parse::<std::net::IpAddr>(),
            entry
                .ip_pattern
                .trim()
                .trim_matches('[')
                .trim_matches(']')
                .parse::<std::net::IpAddr>(),
        ) {
            if client_addr == entry_addr {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
