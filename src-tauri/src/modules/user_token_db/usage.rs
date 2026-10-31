use chrono::{FixedOffset, Timelike, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use super::*;

/// Get all IP bindings for the token
pub fn get_token_ips(token_id: &str) -> Result<Vec<TokenIpBinding>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare("SELECT * FROM token_ip_bindings WHERE token_id = ?1 ORDER BY last_seen_at DESC")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let iter = stmt
        .query_map(params![token_id], |row| {
            Ok(TokenIpBinding {
                id: row.get("id")?,
                token_id: row.get("token_id")?,
                ip_address: row.get("ip_address")?,
                first_seen_at: row.get("first_seen_at")?,
                last_seen_at: row.get("last_seen_at")?,
                request_count: row.get("request_count")?,
                user_agent: row.get("user_agent")?,
            })
        })
        .map_err(|e| format!("Failed to query token IPs: {}", e))?;

    let mut bindings = Vec::new();
    for b in iter {
        bindings.push(b.map_err(|e| format!("Failed to parse binding row: {}", e))?);
    }

    Ok(bindings)
}

/// Log/update token usage (handles both user_tokens and token_ip_bindings)
pub fn record_token_usage_and_ip(
    token_id: &str,
    ip: &str,
    model: &str,
    input_tokens: i32,
    output_tokens: i32,
    status: u16,
    user_agent: Option<String>,
) -> Result<(), String> {
    let mut conn = connect_db()?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to create transaction: {}", e))?;
    let now = Utc::now().timestamp();

    // 1. Update user_tokens main table
    tx.execute(
        "UPDATE user_tokens SET 
            last_used_at = ?1, 
            total_requests = total_requests + 1, 
            total_tokens_used = total_tokens_used + ?2 
        WHERE id = ?3",
        params![now, input_tokens + output_tokens, token_id],
    )
    .map_err(|e| format!("Failed to update user_tokens stats: {}", e))?;

    // 2. Update or insert the token_ip_bindings table
    let binding_exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM token_ip_bindings WHERE token_id = ?1 AND ip_address = ?2)",
        params![token_id, ip],
        |row| row.get(0),
    ).unwrap_or(false);

    if binding_exists {
        tx.execute(
            "UPDATE token_ip_bindings SET 
                last_seen_at = ?1, 
                request_count = request_count + 1,
                user_agent = COALESCE(?2, user_agent)
            WHERE token_id = ?3 AND ip_address = ?4",
            params![now, user_agent, token_id, ip],
        )
        .map_err(|e| format!("Failed to update ip binding: {}", e))?;
    } else {
        let binding_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO token_ip_bindings (
                id, token_id, ip_address, first_seen_at, last_seen_at, request_count, user_agent
            ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
            params![binding_id, token_id, ip, now, now, user_agent],
        )
        .map_err(|e| format!("Failed to insert ip binding: {}", e))?;
    }

    // 3. Insert into token_usage_logs table
    let log_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO token_usage_logs (
            id, token_id, ip_address, model, input_tokens, output_tokens, request_time, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            log_id,
            token_id,
            ip,
            model,
            input_tokens,
            output_tokens,
            now,
            status
        ],
    )
    .map_err(|e| format!("Failed to insert usage log: {}", e))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(())
}

/// Check whether the Token is valid (including expiration time check and IP restriction check)
/// Return: (valid, rejection reason)
pub fn validate_token(token_str: &str, ip: &str) -> Result<(bool, Option<String>), String> {
    let token_opt = get_token_by_value(token_str)?;

    if let Some(token) = token_opt {
        // 1. Check the expiration date
        if token.expires_type != "never" {
            if let Some(expires_at) = token.expires_at {
                if expires_at < Utc::now().timestamp() {
                    return Ok((
                        false,
                        Some(
                            "Your token has expired. Please contact the administrator to renew it."
                                .to_string(),
                        ),
                    ));
                }
            }
        }

        // 2. Check IP restrictions
        if token.max_ips > 0 {
            let conn = connect_db()?;

            // Check if the current IP is bound
            let is_bound: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM token_ip_bindings WHERE token_id = ?1 AND ip_address = ?2)",
                params![token.id, ip],
                |row| row.get(0)
            ).unwrap_or(false);

            if !is_bound {
                // If not bound, check whether the upper limit has been reached
                let current_ip_count: i32 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM token_ip_bindings WHERE token_id = ?1",
                        params![token.id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if current_ip_count >= token.max_ips {
                    return Ok((false, Some(format!("IP limit reached ({}/{}). Please contact the administrator to increase the limit.", current_ip_count, token.max_ips))));
                }
            }
        }

        // 3. Check curfew times (Curfew)
        // Logic: If the current Beijing time is between start and end, reject
        // Format: HH:MM
        // Uses fixed UTC+8 (Beijing time) and does not rely on the server's local time zone
        if let (Some(start_str), Some(end_str)) = (&token.curfew_start, &token.curfew_end) {
            if !start_str.is_empty() && !end_str.is_empty() {
                let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
                let now_beijing = Utc::now().with_timezone(&beijing_offset);
                let current_time_str =
                    format!("{:02}:{:02}", now_beijing.hour(), now_beijing.minute());

                // Processing across midnight: start > end (e.g. 23:00 to 06:00)
                // Normal: start < end (e.g. 09:00 to 18:00)
                let is_curfew = if start_str > end_str {
                    current_time_str >= *start_str || current_time_str < *end_str
                } else {
                    current_time_str >= *start_str && current_time_str < *end_str
                };

                if is_curfew {
                    return Ok((false, Some(format!("Service is not available between {} and {} Beijing Time (Curfew enabled). Current Beijing time: {}", start_str, end_str, current_time_str))));
                }
            }
        }

        // Everything is fine, the Token is valid
        Ok((true, None))
    } else {
        Ok((
            false,
            Some("Invalid token. Please check your API key.".to_string()),
        ))
    }
}

/// Get the username associated with the IP (used in the IP management page)
/// Returns the username to which the Token that last used this IP belongs.
pub fn get_username_for_ip(ip: &str) -> Result<Option<String>, String> {
    let conn = connect_db()?;
    let result: Option<String> = conn
        .query_row(
            "SELECT t.username
         FROM token_ip_bindings b 
         JOIN user_tokens t ON b.token_id = t.id 
         WHERE b.ip_address = ?1 
         ORDER BY b.last_seen_at DESC 
         LIMIT 1",
            params![ip],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Failed to query username by ip: {}", e))?;

    Ok(result)
}
