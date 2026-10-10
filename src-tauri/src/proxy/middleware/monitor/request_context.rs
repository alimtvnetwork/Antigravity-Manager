use super::*;
use std::time::Instant;

/// Request metadata extracted before the request body is consumed.
///
/// Extracted from `monitor_middleware` via extract-method (Phase 1: request setup).
/// Pure code motion — no logic changes.
pub(crate) struct MonitorRequestContext {
    pub(crate) method: String,
    pub(crate) uri: String,
    pub(crate) client_ip: Option<String>,
    pub(crate) user_agent: Option<String>,
    pub(crate) request_content_type: String,
    pub(crate) model: Option<String>,
    pub(crate) request_headers_json: String,
    pub(crate) user_token_identity: Option<UserTokenIdentity>,
    pub(crate) request_body_str: Option<String>,
    pub(crate) start: Instant,
}

/// Extract request metadata and (for POST) read + summarize the request body,
/// rebuilding the request with the buffered body.
pub(crate) async fn extract_request_context(
    request: Request,
    method: String,
    uri: String,
) -> (Request, MonitorRequestContext) {
    let start = Instant::now();

    // Extract client IP from headers (X-Forwarded-For or X-Real-IP)
    // IMPORTANT: Extract from Request headers, not Response headers (since we want the client's IP)
    // Note: We need to do this BEFORE consuming the request body if possible, or extract it from the original request
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let request_content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let mut model = if uri.contains("/v1beta/models/") {
        uri.split("/v1beta/models/")
            .nth(1)
            .and_then(|s| s.split(':').next())
            .map(|s| s.to_string())
    } else {
        None
    };

    let request_headers_json =
        crate::proxy::payload_audit::headers_to_redacted_json(request.headers());

    let request_body_str;

    // [FIX] 从请求 extensions 提取 UserTokenIdentity (由 Auth 中间件注入)
    // 必须在处理 request body 之前提取，因为 into_parts() 后需要保留这个值
    let user_token_identity = request.extensions().get::<UserTokenIdentity>().cloned();

    let request = if method == "POST" {
        let (parts, body) = request.into_parts();
        match axum::body::to_bytes(body, MAX_REQUEST_LOG_SIZE).await {
            Ok(bytes) => {
                request_body_str = if request_content_type.starts_with("multipart/form-data") {
                    if let Some((summary, multipart_model)) =
                        summarize_multipart_request(&bytes, &request_content_type, &uri)
                    {
                        if model.is_none() {
                            model = multipart_model;
                        }
                        Some(summary)
                    } else {
                        Some(format!(
                            "[Multipart Request Data: {} bytes, failed to parse summary]",
                            bytes.len()
                        ))
                    }
                } else if let Ok(s) = std::str::from_utf8(&bytes) {
                    if model.is_none() {
                        model = serde_json::from_slice::<Value>(&bytes).ok().and_then(|v| {
                            v.get("model")
                                .and_then(|m| m.as_str())
                                .map(|s| s.to_string())
                        });
                    }
                    Some(s.to_string())
                } else {
                    Some("[Binary Request Data]".to_string())
                };
                Request::from_parts(parts, Body::from(bytes))
            }
            Err(_) => {
                request_body_str = None;
                Request::from_parts(parts, Body::empty())
            }
        }
    } else {
        request_body_str = None;
        request
    };

    let ctx = MonitorRequestContext {
        method,
        uri,
        client_ip,
        user_agent,
        request_content_type,
        model,
        request_headers_json,
        user_token_identity,
        request_body_str,
        start,
    };
    (request, ctx)
}
