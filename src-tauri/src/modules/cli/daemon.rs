use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

/// Check if the local Antigravity runtime daemon is running on 127.0.0.1:8045
pub fn is_daemon_running() -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], 8045));
    TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok()
}

/// Helper to forward CLI actions to the local authenticated REST API daemon
pub fn forward_to_local_rest<T: serde::de::DeserializeOwned>(
    method: reqwest::Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<T, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let admin_token = std::env::var("AGM_ADMIN_TOKEN")
        .or_else(|_| std::env::var("ABV_API_KEY"))
        .unwrap_or_else(|_| {
            crate::modules::config::load_app_config()
                .ok()
                .and_then(|c| {
                    c.proxy
                        .admin_password
                        .filter(|p| !p.is_empty())
                        .or(Some(c.proxy.api_key))
                })
                .unwrap_or_default()
        });

    let norm_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    let url = format!("http://127.0.0.1:8045/api{}", norm_path);
    let mut req = client.request(method, &url);
    if !admin_token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", admin_token));
        req = req.header("x-api-key", &admin_token);
    }
    if let Some(json_payload) = body {
        req = req.json(&json_payload);
    }

    let resp = req
        .send()
        .map_err(|e| format!("Daemon communication failed: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Daemon returned HTTP status {}", resp.status()));
    }

    resp.json::<T>()
        .map_err(|e| format!("Failed to parse response: {}", e))
}
