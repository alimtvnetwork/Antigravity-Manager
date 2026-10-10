use chrono::{FixedOffset, Timelike, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use super::*;

/// 创建新令牌
pub fn create_token(
    username: String,
    expires_type: String,
    description: Option<String>,
    max_ips: i32,
    curfew_start: Option<String>,
    curfew_end: Option<String>,
    custom_expires_at: Option<i64>, // 自定义过期时间戳 (秒)
) -> Result<UserToken, String> {
    let conn = connect_db()?;
    let id = Uuid::new_v4().to_string();
    let token = format!("sk-{}", Uuid::new_v4().to_string().replace("-", ""));
    let now = Utc::now().timestamp();

    let expires_at = match expires_type.as_str() {
        "day" => Some(
            Utc::now()
                .checked_add_signed(chrono::Duration::days(1))
                .unwrap()
                .timestamp(),
        ),
        "week" => Some(
            Utc::now()
                .checked_add_signed(chrono::Duration::weeks(1))
                .unwrap()
                .timestamp(),
        ),
        "month" => Some(
            Utc::now()
                .checked_add_signed(chrono::Duration::days(30))
                .unwrap()
                .timestamp(),
        ),
        "custom" => custom_expires_at, // 使用自定义时间戳
        _ => None,                     // "never" or other
    };

    let user_token = UserToken {
        id: id.clone(),
        token: token.clone(),
        username: username.clone(),
        description: description.clone(),
        enabled: true,
        expires_type: expires_type.clone(),
        expires_at,
        max_ips,
        curfew_start: curfew_start.clone(),
        curfew_end: curfew_end.clone(),
        created_at: now,
        updated_at: now,
        last_used_at: None,
        total_requests: 0,
        total_tokens_used: 0,
    };

    conn.execute(
        "INSERT INTO user_tokens (
            id, token, username, description, enabled, expires_type, expires_at, max_ips,
            curfew_start, curfew_end,
            created_at, updated_at, total_requests, total_tokens_used
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            user_token.id,
            user_token.token,
            user_token.username,
            user_token.description,
            user_token.enabled,
            user_token.expires_type,
            user_token.expires_at,
            user_token.max_ips,
            user_token.curfew_start,
            user_token.curfew_end,
            user_token.created_at,
            user_token.updated_at,
            user_token.total_requests,
            user_token.total_tokens_used,
        ],
    )
    .map_err(|e| format!("Failed to insert user token: {}", e))?;

    Ok(user_token)
}

/// 列出所有令牌
pub fn list_tokens() -> Result<Vec<UserToken>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT * FROM user_tokens ORDER BY created_at DESC")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let token_iter = stmt
        .query_map([], |row| {
            Ok(UserToken {
                id: row.get("id")?,
                token: row.get("token")?,
                username: row.get("username")?,
                description: row.get("description")?,
                enabled: row.get("enabled").unwrap_or(true), // 防御性默认值
                expires_type: row.get("expires_type").unwrap_or("never".to_string()), // 防御性默认值
                expires_at: row.get("expires_at").unwrap_or(None),
                max_ips: row.get("max_ips").unwrap_or(0),
                curfew_start: row.get("curfew_start").unwrap_or(None),
                curfew_end: row.get("curfew_end").unwrap_or(None),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                last_used_at: row.get("last_used_at").unwrap_or(None),
                total_requests: row.get("total_requests").unwrap_or(0),
                total_tokens_used: row.get("total_tokens_used").unwrap_or(0),
            })
        })
        .map_err(|e| format!("Failed to query tokens: {}", e))?;

    let mut tokens = Vec::new();
    for token in token_iter {
        tokens.push(token.map_err(|e| format!("Failed to parse token row: {}", e))?);
    }

    Ok(tokens)
}

/// 获取单个令牌信息
pub fn get_token_by_id(id: &str) -> Result<Option<UserToken>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT * FROM user_tokens WHERE id = ?1")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let token = stmt
        .query_row(params![id], |row| {
            Ok(UserToken {
                id: row.get("id")?,
                token: row.get("token")?,
                username: row.get("username")?,
                description: row.get("description")?,
                enabled: row.get("enabled")?,
                expires_type: row.get("expires_type")?,
                expires_at: row.get("expires_at")?,
                max_ips: row.get("max_ips")?,
                curfew_start: row.get("curfew_start").unwrap_or(None),
                curfew_end: row.get("curfew_end").unwrap_or(None),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                last_used_at: row.get("last_used_at")?,
                total_requests: row.get("total_requests")?,
                total_tokens_used: row.get("total_tokens_used")?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query token: {}", e))?;

    Ok(token)
}

/// 根据 Token 值获取令牌信息
pub fn get_token_by_value(token: &str) -> Result<Option<UserToken>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT * FROM user_tokens WHERE token = ?1")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let token = stmt
        .query_row(params![token], |row| {
            Ok(UserToken {
                id: row.get("id")?,
                token: row.get("token")?,
                username: row.get("username")?,
                description: row.get("description")?,
                enabled: row.get("enabled")?,
                expires_type: row.get("expires_type")?,
                expires_at: row.get("expires_at")?,
                max_ips: row.get("max_ips")?,
                curfew_start: row.get("curfew_start").unwrap_or(None),
                curfew_end: row.get("curfew_end").unwrap_or(None),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                last_used_at: row.get("last_used_at")?,
                total_requests: row.get("total_requests")?,
                total_tokens_used: row.get("total_tokens_used")?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query token: {}", e))?;

    Ok(token)
}

/// 更新令牌状态/备注等
pub fn update_token(
    id: &str,
    username: Option<String>,
    description: Option<String>,
    enabled: Option<bool>,
    max_ips: Option<i32>,
    curfew_start: Option<Option<String>>,
    curfew_end: Option<Option<String>>,
) -> Result<(), String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let mut query = "UPDATE user_tokens SET updated_at = ?1".to_string();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];
    let mut param_idx = 2;

    if let Some(user) = username {
        query.push_str(&format!(", username = ?{}", param_idx));
        params_vec.push(Box::new(user));
        param_idx += 1;
    }

    if let Some(desc) = description {
        query.push_str(&format!(", description = ?{}", param_idx));
        params_vec.push(Box::new(desc));
        param_idx += 1;
    }

    if let Some(en) = enabled {
        query.push_str(&format!(", enabled = ?{}", param_idx));
        params_vec.push(Box::new(en));
        param_idx += 1;
    }

    if let Some(ips) = max_ips {
        query.push_str(&format!(", max_ips = ?{}", param_idx));
        params_vec.push(Box::new(ips));
        param_idx += 1;
    }

    if let Some(start) = curfew_start {
        query.push_str(&format!(", curfew_start = ?{}", param_idx));
        params_vec.push(Box::new(start));
        param_idx += 1;
    }

    if let Some(end) = curfew_end {
        query.push_str(&format!(", curfew_end = ?{}", param_idx));
        params_vec.push(Box::new(end));
        param_idx += 1;
    }

    query.push_str(&format!(" WHERE id = ?{}", param_idx));
    params_vec.push(Box::new(id.to_string()));

    // 将 Vec<Box<dyn ToSql>> 转换为 &[&dyn ToSql]
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    conn.execute(&query, params_refs.as_slice())
        .map_err(|e| format!("Failed to update user token: {}", e))?;

    Ok(())
}

/// 续期令牌
pub fn renew_token(id: &str, expires_type: &str) -> Result<(), String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let expires_at = match expires_type {
        "day" => Some(
            Utc::now()
                .checked_add_signed(chrono::Duration::days(1))
                .unwrap()
                .timestamp(),
        ),
        "week" => Some(
            Utc::now()
                .checked_add_signed(chrono::Duration::weeks(1))
                .unwrap()
                .timestamp(),
        ),
        "month" => Some(
            Utc::now()
                .checked_add_signed(chrono::Duration::days(30))
                .unwrap()
                .timestamp(),
        ),
        _ => None, // "never" or other
    };

    conn.execute(
        "UPDATE user_tokens SET expires_type = ?1, expires_at = ?2, updated_at = ?3, enabled = 1 WHERE id = ?4",
        params![expires_type, expires_at, now, id],
    ).map_err(|e| format!("Failed to renew token: {}", e))?;

    Ok(())
}

/// 删除令牌
pub fn delete_token(id: &str) -> Result<(), String> {
    let conn = connect_db()?;
    conn.execute("DELETE FROM user_tokens WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete token: {}", e))?;
    Ok(())
}
