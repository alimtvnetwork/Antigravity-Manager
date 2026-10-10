use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub id: String,
    pub token: String,
    pub username: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub expires_type: String, // "day", "week", "month", "never"
    pub expires_at: Option<i64>,
    pub max_ips: i32,                 // 0 = unlimited
    pub curfew_start: Option<String>, // "HH:MM" 宵禁开始时间
    pub curfew_end: Option<String>,   // "HH:MM" 宵禁结束时间
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
    pub total_requests: i64,
    pub total_tokens_used: i64,
}

/// 令牌 IP 绑定结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIpBinding {
    pub id: String,
    pub token_id: String,
    pub ip_address: String,
    pub first_seen_at: i64,
    pub last_seen_at: i64,
    pub request_count: i64,
    pub user_agent: Option<String>,
}

/// 令牌使用日志结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageLog {
    pub id: String,
    pub token_id: String,
    pub ip_address: String,
    pub model: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub request_time: i64,
    pub status: u16,
}
