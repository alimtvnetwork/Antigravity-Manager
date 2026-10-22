//! Interactive REST try-it-out endpoint dispatcher for Antigravity-Manager.
//! Executes internal loopback requests against the local proxy gateway with latency telemetry.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Request payload for the internal REST runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestExecuteRequest {
    pub method: String,
    pub path: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
    pub timeout_ms: Option<u64>,
}

/// Telemetry response containing headers, payload, status, and precise latency duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestExecuteResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub duration_ms: u64,
    pub is_success: bool,
    pub error: Option<String>,
}

/// Executes an internal HTTP request against the specified base URL.
pub async fn execute_internal_rest_request(
    req: RestExecuteRequest,
    base_url: &str,
) -> RestExecuteResponse {
    let start = Instant::now();
    let sanitized_path = if req.path.starts_with('/') {
        req.path.clone()
    } else {
        format!("/{}", req.path)
    };
    let full_url = format!("{}{}", base_url.trim_end_matches('/'), sanitized_path);

    let client_builder = reqwest::Client::builder();
    let timeout = std::time::Duration::from_millis(req.timeout_ms.unwrap_or(15000));
    let client = match client_builder.timeout(timeout).build() {
        Ok(c) => c,
        Err(e) => {
            return RestExecuteResponse {
                status: 500,
                headers: HashMap::new(),
                body: serde_json::json!({ "error": format!("Client build error: {e}") }),
                duration_ms: start.elapsed().as_millis() as u64,
                is_success: false,
                error: Some(format!("Failed to build HTTP client: {e}")),
            };
        }
    };

    let method = match req.method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let mut request_builder = client.request(method, &full_url);

    if let Some(headers_map) = req.headers {
        for (k, v) in headers_map {
            request_builder = request_builder.header(&k, &v);
        }
    }

    if let Some(body_json) = req.body {
        request_builder = request_builder.json(&body_json);
    }

    match request_builder.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let mut headers = HashMap::new();
            for (key, val) in resp.headers().iter() {
                if let Ok(v_str) = val.to_str() {
                    headers.insert(key.to_string(), v_str.to_string());
                }
            }

            let body_val = match resp.text().await {
                Ok(text) => serde_json::from_str::<serde_json::Value>(&text)
                    .unwrap_or_else(|_| serde_json::json!({ "raw": text })),
                Err(e) => serde_json::json!({ "read_error": format!("{e}") }),
            };

            let duration_ms = start.elapsed().as_millis() as u64;
            let is_success = (200..300).contains(&status);

            RestExecuteResponse {
                status,
                headers,
                body: body_val,
                duration_ms,
                is_success,
                error: None,
            }
        }
        Err(err) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            RestExecuteResponse {
                status: 502,
                headers: HashMap::new(),
                body: serde_json::json!({ "transport_error": format!("{err}") }),
                duration_ms,
                is_success: false,
                error: Some(format!("Request failed: {err}")),
            }
        }
    }
}
