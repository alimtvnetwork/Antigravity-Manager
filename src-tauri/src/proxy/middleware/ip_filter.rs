use crate::modules::security_db;
use crate::proxy::server::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

/// IP black and white list filtering middleware
pub async fn ip_filter_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Extract client IP
    let client_ip = extract_client_ip(&request);

    if let Some(ip) = &client_ip {
        // Read security configuration
        let security_config = state.security.read().await;

        // 1. Check the whitelist (if whitelist mode is enabled, only whitelist IPs are allowed)
        if security_config.security_monitor.whitelist.enabled {
            match security_db::is_ip_in_whitelist(ip) {
                Ok(true) => {
                    // In the whitelist, directly allow
                    tracing::debug!("[IP Filter] IP {} is in whitelist, allowing", ip);
                    return next.run(request).await;
                }
                Ok(false) => {
                    // Not in the whitelist, and whitelist mode is enabled, access is denied
                    tracing::warn!("[IP Filter] IP {} not in whitelist, blocking", ip);
                    return create_blocked_response(
                        ip,
                        "Access denied. Your IP is not in the whitelist.",
                    );
                }
                Err(e) => {
                    tracing::error!("[IP Filter] Failed to check whitelist: {}", e);
                }
            }
        } else {
            // Whitelist priority mode: If in the whitelist, skip the blacklist check
            if security_config
                .security_monitor
                .whitelist
                .whitelist_priority
            {
                match security_db::is_ip_in_whitelist(ip) {
                    Ok(true) => {
                        tracing::debug!("[IP Filter] IP {} is in whitelist (priority mode), skipping blacklist check", ip);
                        return next.run(request).await;
                    }
                    Ok(false) => {
                        // Keep checking the blacklist
                    }
                    Err(e) => {
                        tracing::error!("[IP Filter] Failed to check whitelist: {}", e);
                    }
                }
            }
        }

        // 2. Check the blacklist
        if security_config.security_monitor.blacklist.enabled {
            match security_db::get_blacklist_entry_for_ip(ip) {
                Ok(Some(entry)) => {
                    tracing::warn!("[IP Filter] IP {} is in blacklist, blocking", ip);

                    // Construct detailed ban messages
                    let reason = entry
                        .reason
                        .as_deref()
                        .unwrap_or("Malicious activity detected");
                    let ban_type = if let Some(expires_at) = entry.expires_at {
                        let now = chrono::Utc::now().timestamp();
                        let remaining_seconds = expires_at - now;

                        if remaining_seconds > 0 {
                            let hours = remaining_seconds / 3600;
                            let minutes = (remaining_seconds % 3600) / 60;

                            if hours > 24 {
                                let days = hours / 24;
                                format!("Temporary ban. Please try again after {} day(s).", days)
                            } else if hours > 0 {
                                format!("Temporary ban. Please try again after {} hour(s) and {} minute(s).", hours, minutes)
                            } else {
                                format!(
                                    "Temporary ban. Please try again after {} minute(s).",
                                    minutes
                                )
                            }
                        } else {
                            "Temporary ban (expired, will be removed soon).".to_string()
                        }
                    } else {
                        "Permanent ban.".to_string()
                    };

                    let detailed_message =
                        format!("Access denied. Reason: {}. {}", reason, ban_type);

                    // Record blocked access logs
                    let log = security_db::IpAccessLog {
                        id: uuid::Uuid::new_v4().to_string(),
                        client_ip: ip.clone(),
                        timestamp: chrono::Utc::now().timestamp(),
                        method: Some(request.method().to_string()),
                        path: Some(request.uri().to_string()),
                        user_agent: request
                            .headers()
                            .get("user-agent")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string()),
                        status: Some(403),
                        duration: Some(0),
                        api_key_hash: None,
                        blocked: true,
                        block_reason: Some(format!("IP in blacklist: {}", reason)),
                        username: None,
                    };

                    tokio::spawn(async move {
                        if let Err(e) = security_db::save_ip_access_log(&log) {
                            tracing::error!("[IP Filter] Failed to save blocked access log: {}", e);
                        }
                    });

                    return create_blocked_response(ip, &detailed_message);
                }
                Ok(None) => {
                    // Not in the blacklist, let go
                    tracing::debug!("[IP Filter] IP {} not in blacklist, allowing", ip);
                }
                Err(e) => {
                    tracing::error!("[IP Filter] Failed to check blacklist: {}", e);
                }
            }
        }
    } else {
        tracing::warn!("[IP Filter] Unable to extract client IP from request");
    }

    // Release request
    next.run(request).await
}

/// Normalized IP address string:
/// - Clean up leading and trailing spaces and square brackets `[...]`
/// - Strip the client port (e.g. `192.168.1.1:54321` or `[2409:8a55::1]:8046`)
/// - Revert an IPv4-mapped IPv6 address (e.g. `::ffff:192.168.1.1`) to native IPv4 (`192.168.1.1`)
pub fn normalize_ip_str(raw: &str) -> String {
    let trimmed = raw.trim();

    // Prioritize trying to resolve directly by IP address
    let clean = trimmed.trim_matches('[').trim_matches(']');
    if let Ok(ip) = clean.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V6(v6) => {
                if let Some(v4) = v6.to_ipv4_mapped() {
                    v4.to_string()
                } else {
                    v6.to_string()
                }
            }
            std::net::IpAddr::V4(v4) => v4.to_string(),
        };
    }

    // If there is a port (such as 192.168.1.100:12345 or [::1]:8080), try to resolve it to SocketAddr
    if let Ok(socket_addr) = trimmed.parse::<std::net::SocketAddr>() {
        return match socket_addr.ip() {
            std::net::IpAddr::V6(v6) => {
                if let Some(v4) = v6.to_ipv4_mapped() {
                    v4.to_string()
                } else {
                    v6.to_string()
                }
            }
            std::net::IpAddr::V4(v4) => v4.to_string(),
        };
    }

    // Manual prefix fallback detection
    if let Some(stripped) = clean.strip_prefix("::ffff:") {
        if stripped.parse::<std::net::Ipv4Addr>().is_ok() {
            return stripped.to_string();
        }
    }

    clean.to_string()
}

/// Extract client IP from request (supports X-Forwarded-For, X-Real-IP, ConnectInfo, and automatically normalizes IPv4/IPv6)
pub fn extract_client_ip(request: &Request) -> Option<String> {
    // 1. Extract from X-Forwarded-For first (take the first IP)
    let raw = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim())
        .or_else(|| {
            // 2. Alternative extraction from X-Real-IP
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim())
        });

    if let Some(ip_str) = raw {
        if !ip_str.is_empty() {
            return Some(normalize_ip_str(ip_str));
        }
    }

    // 3. Finally try to get (TCP connection IP) from ConnectInfo
    request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|info| match info.0.ip() {
            std::net::IpAddr::V6(v6) => {
                if let Some(v4) = v6.to_ipv4_mapped() {
                    v4.to_string()
                } else {
                    v6.to_string()
                }
            }
            std::net::IpAddr::V4(v4) => v4.to_string(),
        })
}

/// Create a blocked response
fn create_blocked_response(ip: &str, message: &str) -> Response {
    let body = serde_json::json!({
        "error": {
            "message": message,
            "type": "ip_blocked",
            "code": "ip_blocked",
            "ip": ip,
        }
    });

    (
        StatusCode::FORBIDDEN,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&body).unwrap_or_else(|_| message.to_string()),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_ip_str() {
        assert_eq!(normalize_ip_str("192.168.1.1"), "192.168.1.1");
        assert_eq!(normalize_ip_str("192.168.1.1:8080"), "192.168.1.1");
        assert_eq!(normalize_ip_str("::ffff:192.168.1.1"), "192.168.1.1");
        assert_eq!(normalize_ip_str("[::ffff:192.168.1.1]:8046"), "192.168.1.1");
        assert_eq!(normalize_ip_str("::1"), "::1");
        assert_eq!(normalize_ip_str("[::1]:8046"), "::1");
        assert_eq!(
            normalize_ip_str("2409:8a55:a21:2e60:2e2:69ff:fe17:95cb"),
            "2409:8a55:a21:2e60:2e2:69ff:fe17:95cb"
        );
    }
}
