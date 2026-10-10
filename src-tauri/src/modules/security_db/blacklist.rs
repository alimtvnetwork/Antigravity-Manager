use rusqlite::{params, Connection};

use super::*;

// ============================================================================
// 黑名单操作
// ============================================================================

/// 添加 IP 到黑名单
pub fn add_to_blacklist(
    ip_pattern: &str,
    reason: Option<&str>,
    expires_at: Option<i64>,
    created_by: &str,
) -> Result<IpBlacklistEntry, String> {
    let conn = connect_db()?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO ip_blacklist (id, ip_pattern, reason, created_at, expires_at, created_by, hit_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
        params![id, ip_pattern, reason, now, expires_at, created_by],
    )
    .map_err(|e| e.to_string())?;

    Ok(IpBlacklistEntry {
        id,
        ip_pattern: ip_pattern.to_string(),
        reason: reason.map(|s| s.to_string()),
        created_at: now,
        expires_at,
        created_by: created_by.to_string(),
        hit_count: 0,
    })
}

/// 从黑名单移除
pub fn remove_from_blacklist(id: &str) -> Result<(), String> {
    let conn = connect_db()?;

    conn.execute("DELETE FROM ip_blacklist WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取黑名单列表
pub fn get_blacklist() -> Result<Vec<IpBlacklistEntry>, String> {
    let conn = connect_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, ip_pattern, reason, created_at, expires_at, created_by, hit_count
             FROM ip_blacklist
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let entries_iter = stmt
        .query_map([], |row| {
            Ok(IpBlacklistEntry {
                id: row.get(0)?,
                ip_pattern: row.get(1)?,
                reason: row.get(2)?,
                created_at: row.get(3)?,
                expires_at: row.get(4)?,
                created_by: row.get(5)?,
                hit_count: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for e in entries_iter {
        entries.push(e.map_err(|e| e.to_string())?);
    }
    Ok(entries)
}

/// 检查 IP 是否在黑名单中
pub fn is_ip_in_blacklist(ip: &str) -> Result<bool, String> {
    get_blacklist_entry_for_ip(ip).map(|entry| entry.is_some())
}

/// 获取 IP 对应的黑名单条目（如果存在）
pub fn get_blacklist_entry_for_ip(ip: &str) -> Result<Option<IpBlacklistEntry>, String> {
    let conn = connect_db()?;
    let now = chrono::Utc::now().timestamp();

    // 清理过期的黑名单条目
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "DELETE FROM ip_blacklist WHERE expires_at IS NOT NULL AND expires_at < ?1",
            [now],
        ),
        "db execute",
    );

    // 精确匹配
    let entry_result = conn.query_row(
        "SELECT id, ip_pattern, reason, created_at, expires_at, created_by, hit_count
         FROM ip_blacklist WHERE ip_pattern = ?1",
        [ip],
        |row| {
            Ok(IpBlacklistEntry {
                id: row.get(0)?,
                ip_pattern: row.get(1)?,
                reason: row.get(2)?,
                created_at: row.get(3)?,
                expires_at: row.get(4)?,
                created_by: row.get(5)?,
                hit_count: row.get(6)?,
            })
        },
    );

    if let Ok(entry) = entry_result {
        // 增加命中计数
        // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
        crate::error::record_ignored(
            conn.execute(
                "UPDATE ip_blacklist SET hit_count = hit_count + 1 WHERE ip_pattern = ?1",
                [ip],
            ),
            "db execute",
        );
        return Ok(Some(entry));
    }

    // CIDR 匹配与 IP 等价匹配
    let entries = get_blacklist()?;
    for entry in entries {
        if entry.ip_pattern.contains('/') {
            if cidr_match(ip, &entry.ip_pattern) {
                // 增加命中计数
                // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE ip_blacklist SET hit_count = hit_count + 1 WHERE id = ?1",
                        [&entry.id],
                    ),
                    "db execute",
                );
                return Ok(Some(entry));
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
                // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE ip_blacklist SET hit_count = hit_count + 1 WHERE id = ?1",
                        [&entry.id],
                    ),
                    "db execute",
                );
                return Ok(Some(entry));
            }
        }
    }

    Ok(None)
}

/// CIDR 匹配 (同时支持 IPv4 和 IPv6)
pub(crate) fn cidr_match(ip: &str, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let network = parts[0].trim();
    let prefix_len: u8 = match parts[1].trim().parse() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let ip_clean = ip.trim().trim_matches('[').trim_matches(']');
    let net_clean = network.trim_matches('[').trim_matches(']');

    let ip_addr: std::net::IpAddr = match ip_clean.parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    let net_addr: std::net::IpAddr = match net_clean.parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };

    // 尝试展开 IPv4-mapped
    let ip_addr = match ip_addr {
        std::net::IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                std::net::IpAddr::V4(v4)
            } else {
                std::net::IpAddr::V6(v6)
            }
        }
        v4 => v4,
    };
    let net_addr = match net_addr {
        std::net::IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                std::net::IpAddr::V4(v4)
            } else {
                std::net::IpAddr::V6(v6)
            }
        }
        v4 => v4,
    };

    match (ip_addr, net_addr) {
        (std::net::IpAddr::V4(ip_v4), std::net::IpAddr::V4(net_v4)) => {
            if prefix_len > 32 {
                return false;
            }
            let ip_u32 = u32::from_be_bytes(ip_v4.octets());
            let net_u32 = u32::from_be_bytes(net_v4.octets());
            let mask = if prefix_len == 0 {
                0
            } else {
                !0u32 << (32 - prefix_len)
            };
            (ip_u32 & mask) == (net_u32 & mask)
        }
        (std::net::IpAddr::V6(ip_v6), std::net::IpAddr::V6(net_v6)) => {
            if prefix_len > 128 {
                return false;
            }
            let ip_u128 = u128::from_be_bytes(ip_v6.octets());
            let net_u128 = u128::from_be_bytes(net_v6.octets());
            let mask = if prefix_len == 0 {
                0
            } else {
                !0u128 << (128 - prefix_len)
            };
            (ip_u128 & mask) == (net_u128 & mask)
        }
        _ => false,
    }
}
