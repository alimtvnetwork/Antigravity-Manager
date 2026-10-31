use crate::modules::security_db;
use serde::{Deserialize, Serialize};
use tauri::State;

// ==================== Request/Response Structure ====================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct IpAccessLogQuery {
    pub page: usize,
    pub page_size: usize,
    pub search: Option<String>,
    pub blocked_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpAccessLogResponse {
    pub logs: Vec<security_db::IpAccessLog>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBlacklistRequest {
    pub ip_pattern: String,
    pub reason: Option<String>,
    pub expires_at: Option<i64>, // Unix timestamp
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWhitelistRequest {
    pub ip_pattern: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpStatsResponse {
    pub total_requests: usize,
    pub unique_ips: usize,
    pub blocked_requests: usize,
    pub top_ips: Vec<security_db::IpRanking>,
}

// ==================== IP Access Log Command ====================

/// Get IP access log list
#[tauri::command]
pub async fn get_ip_access_logs(
    query: Option<IpAccessLogQuery>,
    page: Option<usize>,
    page_size: Option<usize>,
    search: Option<String>,
    blocked_only: Option<bool>,
) -> Result<IpAccessLogResponse, String> {
    let resolved_page = query.as_ref().map(|q| q.page).or(page).unwrap_or(1);
    let resolved_page_size = query
        .as_ref()
        .map(|q| q.page_size)
        .or(page_size)
        .unwrap_or(50);
    let resolved_search = query.as_ref().and_then(|q| q.search.clone()).or(search);
    let resolved_blocked_only = query
        .as_ref()
        .map(|q| q.blocked_only)
        .or(blocked_only)
        .unwrap_or(false);

    let offset = (resolved_page.max(1) - 1) * resolved_page_size;

    let logs = security_db::get_ip_access_logs(
        resolved_page_size,
        offset,
        resolved_search.as_deref(),
        resolved_blocked_only,
    )?;

    // Simple calculation of the total (if you need precise paging, you can add the count function)
    let total = logs.len();

    Ok(IpAccessLogResponse { logs, total })
}

/// Get IP statistics
#[tauri::command]
pub async fn get_ip_stats() -> Result<IpStatsResponse, String> {
    let stats = security_db::get_ip_stats()?;
    let top_ips = security_db::get_top_ips(10, 24)?; // Top 10 IPs in last 24 hours

    Ok(IpStatsResponse {
        total_requests: stats.total_requests as usize,
        unique_ips: stats.unique_ips as usize,
        blocked_requests: stats.blocked_count as usize,
        top_ips,
    })
}

/// Clear IP access logs
#[tauri::command]
pub async fn clear_ip_access_logs() -> Result<(), String> {
    security_db::clear_ip_access_logs()
}

// ==================== IP Blacklist Command ====================

/// Get IP blacklist list
#[tauri::command]
pub async fn get_ip_blacklist() -> Result<Vec<security_db::IpBlacklistEntry>, String> {
    security_db::get_blacklist()
}

/// Add IP to blacklist
#[tauri::command]
pub async fn add_ip_to_blacklist(request: AddBlacklistRequest) -> Result<(), String> {
    // Verify IP format
    if !is_valid_ip_pattern(&request.ip_pattern) {
        return Err(
            "Invalid IP pattern. Use IP address or CIDR notation (e.g., 192.168.1.0/24)"
                .to_string(),
        );
    }

    security_db::add_to_blacklist(
        &request.ip_pattern,
        request.reason.as_deref(),
        request.expires_at,
        "manual",
    )?;
    Ok(())
}

/// Remove IP from blacklist
#[tauri::command]
pub async fn remove_ip_from_blacklist(ip_pattern: String) -> Result<(), String> {
    // First get the blacklist list and find the corresponding id
    let entries = security_db::get_blacklist()?;
    let entry = entries.iter().find(|e| e.ip_pattern == ip_pattern);

    if let Some(entry) = entry {
        security_db::remove_from_blacklist(&entry.id)
    } else {
        Err(format!("IP pattern {} not found in blacklist", ip_pattern))
    }
}

/// Clear blacklist
#[tauri::command]
pub async fn clear_ip_blacklist() -> Result<(), String> {
    // Get all blacklist entries and delete them one by one
    let entries = security_db::get_blacklist()?;
    for entry in entries {
        security_db::remove_from_blacklist(&entry.ip_pattern)?;
    }
    Ok(())
}

/// Check if IP is in blacklist
#[tauri::command]
pub async fn check_ip_in_blacklist(ip: String) -> Result<bool, String> {
    security_db::is_ip_in_blacklist(&ip)
}

// ==================== IP whitelist command ====================

/// Get IP whitelist list
#[tauri::command]
pub async fn get_ip_whitelist() -> Result<Vec<security_db::IpWhitelistEntry>, String> {
    security_db::get_whitelist()
}

/// Add IP to whitelist
#[tauri::command]
pub async fn add_ip_to_whitelist(request: AddWhitelistRequest) -> Result<(), String> {
    // Verify IP format
    if !is_valid_ip_pattern(&request.ip_pattern) {
        return Err(
            "Invalid IP pattern. Use IP address or CIDR notation (e.g., 192.168.1.0/24)"
                .to_string(),
        );
    }

    security_db::add_to_whitelist(&request.ip_pattern, request.description.as_deref())?;
    Ok(())
}

/// Remove IP from whitelist
#[tauri::command]
pub async fn remove_ip_from_whitelist(ip_pattern: String) -> Result<(), String> {
    // First get the whitelist list and find the corresponding id
    let entries = security_db::get_whitelist()?;
    let entry = entries.iter().find(|e| e.ip_pattern == ip_pattern);

    if let Some(entry) = entry {
        security_db::remove_from_whitelist(&entry.id)
    } else {
        Err(format!("IP pattern {} not found in whitelist", ip_pattern))
    }
}

/// Clear empty list
#[tauri::command]
pub async fn clear_ip_whitelist() -> Result<(), String> {
    // Get all whitelist entries and delete them one by one
    let entries = security_db::get_whitelist()?;
    for entry in entries {
        security_db::remove_from_whitelist(&entry.ip_pattern)?;
    }
    Ok(())
}

/// Check if the IP is in the whitelist
#[tauri::command]
pub async fn check_ip_in_whitelist(ip: String) -> Result<bool, String> {
    security_db::is_ip_in_whitelist(&ip)
}

// ==================== Security configuration commands ====================

/// Get security monitoring configuration
#[tauri::command]
pub async fn get_security_config(
    app_state: State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<crate::proxy::config::SecurityMonitorConfig, String> {
    // 1. Try to get from the running instance (may be the latest configuration in memory)
    let instance_lock = app_state.instance.read().await;
    if let Some(instance) = instance_lock.as_ref() {
        return Ok(instance.config.security_monitor.clone());
    }

    // 2. If the service is not running, load it from disk
    let app_config = crate::modules::config::load_app_config()
        .map_err(|e| format!("Failed to load config: {}", e))?;
    Ok(app_config.proxy.security_monitor)
}

/// Update security monitoring configuration
#[tauri::command]
pub async fn update_security_config(
    config: crate::proxy::config::SecurityMonitorConfig,
    app_state: State<'_, crate::commands::proxy::ProxyServiceState>,
) -> Result<(), String> {
    // 1. Synchronously save to configuration file
    let mut app_config = crate::modules::config::load_app_config()
        .map_err(|e| format!("Failed to load config: {}", e))?;
    app_config.proxy.security_monitor = config.clone();
    crate::modules::config::save_app_config(&app_config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    // 2. Update the in-memory configuration (if the service is running)
    {
        let mut instance_lock = app_state.instance.write().await;
        if let Some(instance) = instance_lock.as_mut() {
            instance.config.security_monitor = config.clone();
            // [FIX] Call update_security to hot update running middleware configuration
            // This is a critical step! The middleware reads AppState.security (Arc<RwLock<ProxySecurityConfig>>)
            // Update_security() must be called to make the black and white list configuration take effect in real time
            instance.axum_server.update_security(&instance.config).await;
            tracing::info!("[Security] Runtime security config hot-reloaded");
        }
    }

    tracing::info!("[Security] Security monitor config updated and saved");
    Ok(())
}

// ==================== Statistical analysis commands ====================

/// Get IP Token consumption statistics
#[tauri::command]
pub async fn get_ip_token_stats(
    limit: Option<usize>,
    hours: Option<i64>,
) -> Result<Vec<crate::modules::proxy_db::IpTokenStats>, String> {
    crate::modules::proxy_db::get_token_usage_by_ip(limit.unwrap_or(100), hours.unwrap_or(720))
}

// ==================== Auxiliary functions ====================

/// Verify IP pattern format (single IP and CIDR supported, IPv4 and IPv6 supported)
fn is_valid_ip_pattern(pattern: &str) -> bool {
    let clean = pattern.trim().trim_matches('[').trim_matches(']');
    // Check if CIDR format
    if clean.contains('/') {
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        let ip_part = parts[0].trim().trim_matches('[').trim_matches(']');
        let mask_part = parts[1].trim();

        if let Ok(ip_addr) = ip_part.parse::<std::net::IpAddr>() {
            if let Ok(mask) = mask_part.parse::<u8>() {
                return match ip_addr {
                    std::net::IpAddr::V4(_) => mask <= 32,
                    std::net::IpAddr::V6(_) => mask <= 128,
                };
            }
        }
        return false;
    }

    // Single IP address
    is_valid_ip(clean)
}

/// Verify IP address format (supports IPv4 and IPv6)
fn is_valid_ip(ip: &str) -> bool {
    let clean = ip.trim().trim_matches('[').trim_matches(']');
    clean.parse::<std::net::IpAddr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ip_patterns() {
        // IPv4
        assert!(is_valid_ip_pattern("192.168.1.1"));
        assert!(is_valid_ip_pattern("10.0.0.0/8"));
        assert!(is_valid_ip_pattern("172.16.0.0/16"));
        assert!(is_valid_ip_pattern("192.168.1.0/24"));
        assert!(is_valid_ip_pattern("8.8.8.8/32"));
        // IPv6
        assert!(is_valid_ip_pattern("::1"));
        assert!(is_valid_ip_pattern("2409:8a55:a21:2e60:2e2:69ff:fe17:95cb"));
        assert!(is_valid_ip_pattern("2409:8a55::/32"));
        assert!(is_valid_ip_pattern("fe80::/10"));
        assert!(is_valid_ip_pattern("::/0"));
        assert!(is_valid_ip_pattern("[::1]"));
    }

    #[test]
    fn test_invalid_ip_patterns() {
        assert!(!is_valid_ip_pattern("256.1.1.1"));
        assert!(!is_valid_ip_pattern("192.168.1"));
        assert!(!is_valid_ip_pattern("192.168.1.1/33"));
        assert!(!is_valid_ip_pattern("192.168.1.1/"));
        assert!(!is_valid_ip_pattern("2409:8a55::/129"));
        assert!(!is_valid_ip_pattern("invalid"));
    }
}
