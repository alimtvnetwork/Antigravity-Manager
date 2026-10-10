use serde::{Deserialize, Serialize};

use super::*;

#[cfg(test)]
pub static TEST_SECURITY_DB_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// IP 访问日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAccessLog {
    pub id: String,
    pub client_ip: String,
    pub timestamp: i64,
    pub method: Option<String>,
    pub path: Option<String>,
    pub user_agent: Option<String>,
    pub status: Option<i32>,
    pub duration: Option<i64>,
    pub api_key_hash: Option<String>,
    pub blocked: bool,
    pub block_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// IP 黑名单条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpBlacklistEntry {
    pub id: String,
    pub ip_pattern: String,
    pub reason: Option<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub created_by: String,
    pub hit_count: i64,
}

/// IP 白名单条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistEntry {
    pub id: String,
    pub ip_pattern: String,
    pub description: Option<String>,
    pub created_at: i64,
}

/// IP 统计概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpStats {
    pub total_requests: u64,
    pub unique_ips: u64,
    pub blocked_count: u64,
    pub today_requests: u64,
    pub blacklist_count: u64,
    pub whitelist_count: u64,
}

/// IP 访问排行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRanking {
    pub client_ip: String,
    pub request_count: u64,
    pub last_seen: i64,
    pub is_blocked: bool,
}
