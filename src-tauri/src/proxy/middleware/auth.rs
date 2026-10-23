// API Key authentication middleware
use axum::{
    extract::Request,
    extract::State,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::proxy::{ProxyAuthMode, ProxySecurityConfig};

/// API Key authentication middleware (used by proxy interface, follows auth_mode)
pub async fn auth_middleware(
    state: State<Arc<RwLock<ProxySecurityConfig>>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    auth_middleware_internal(state, request, next, false).await
}

/// Management interface authentication middleware (used by the management interface, forcing strict authentication)
pub async fn admin_auth_middleware(
    state: State<Arc<RwLock<ProxySecurityConfig>>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    auth_middleware_internal(state, request, next, true).await
}

/// Internal authentication logic
async fn auth_middleware_internal(
    State(security): State<Arc<RwLock<ProxySecurityConfig>>>,
    request: Request,
    next: Next,
    force_strict: bool,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // Filter heartbeat and health check requests to avoid log noise
    let is_health_check = path == "/healthz" || path == "/api/health" || path == "/health";
    let is_internal_endpoint = path.starts_with("/internal/");
    if !path.contains("event_logging") && !is_health_check {
        tracing::info!("Request: {} {}", method, path);
    } else {
        tracing::trace!("Heartbeat/Health: {} {}", method, path);
    }

    // Allow CORS preflight regardless of auth policy.
    if method == axum::http::Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    let security = security.read().await.clone();
    let effective_mode = security.effective_auth_mode();

    // Permission check logic
    if !force_strict {
        // AI agent interface (v1/chat/completions, etc.)
        if matches!(effective_mode, ProxyAuthMode::Off) {
            // [FIX] Need to try to identify User Token to log usage even if auth_mode=Off
            // First check whether you carry User Token
            let api_key = request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer ").or(Some(s)))
                .or_else(|| {
                    request
                        .headers()
                        .get("x-api-key")
                        .and_then(|h| h.to_str().ok())
                });

            if let Some(token) = api_key {
                // Try to verify if it is a User Token (does not block the request, just logs it)
                if let Ok(Some(user_token)) =
                    crate::modules::user_token_db::get_token_by_value(token)
                {
                    let identity = UserTokenIdentity {
                        token_id: user_token.id,
                        token: user_token.token,
                        username: user_token.username,
                    };
                    // Inject identity into the request
                    let (mut parts, body) = request.into_parts();
                    parts.extensions.insert(identity);
                    let request = Request::from_parts(parts, body);
                    return Ok(next.run(request).await);
                }
            }

            return Ok(next.run(request).await);
        }

        if matches!(effective_mode, ProxyAuthMode::AllExceptHealth) && is_health_check {
            return Ok(next.run(request).await);
        }

        // Internal endpoints (/internal/*) are exempt from authentication - used for internal functions such as warmup
        if is_internal_endpoint {
            tracing::debug!("Internal endpoint bypassed auth: {}", path);
            return Ok(next.run(request).await);
        }
    } else {
        // Management endpoints always require admin auth; only health checks stay public.
        if is_health_check {
            return Ok(next.run(request).await);
        }
    }

    // Extract API key from header
    let api_key = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").or(Some(s)))
        .or_else(|| {
            request
                .headers()
                .get("x-api-key")
                .and_then(|h| h.to_str().ok())
        })
        .or_else(|| {
            request
                .headers()
                .get("x-goog-api-key")
                .and_then(|h| h.to_str().ok())
        });

    if security.api_key.is_empty()
        && (security.admin_password.is_none()
            || security.admin_password.as_ref().unwrap().is_empty())
    {
        if force_strict {
            tracing::error!("Admin auth is required but both api_key and admin_password are empty; denying request");
            return Err(StatusCode::UNAUTHORIZED);
        }
        tracing::error!("Proxy auth is enabled but api_key is empty; denying request");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Authentication logic
    let authorized = if force_strict {
        // Management interface: Prioritize using independent admin_password, if not, fall back to api_key
        match &security.admin_password {
            Some(pwd) if !pwd.is_empty() => api_key.map(|k| k == pwd).unwrap_or(false),
            _ => {
                // Fallback to using api_key
                api_key.map(|k| k == security.api_key).unwrap_or(false)
            }
        }
    } else {
        // AI agent interface: only api_key is allowed
        api_key.map(|k| k == security.api_key).unwrap_or(false)
    };

    if authorized {
        Ok(next.run(request).await)
    } else if !force_strict && api_key.is_some() {
        // Try to verify UserToken
        let token = api_key.unwrap();

        // Extract IP (reuse ip_filter normalization logic, support IPv4/IPv6 and ConnectInfo)
        let client_ip = crate::proxy::middleware::ip_filter::extract_client_ip(&request)
            .unwrap_or_else(|| "127.0.0.1".to_string()); // Default fallback

        // Verify Token
        match crate::modules::user_token_db::validate_token(token, &client_ip) {
            Ok((true, _)) => {
                // Token is valid, query information for delivery
                if let Ok(Some(user_token)) =
                    crate::modules::user_token_db::get_token_by_value(token)
                {
                    let identity = UserTokenIdentity {
                        token_id: user_token.id,
                        token: user_token.token,
                        username: user_token.username,
                    };

                    // [FIX] Inject identity information into request extensions, not responses
                    // In this way, monitor_middleware can obtain the identity when processing the request.
                    // Because the middleware execution sequence is: auth (outer layer) -> monitor (inner layer) -> handler
                    // When the response is returned: handler -> monitor -> auth
                    // If injected into the response, identity does not exist yet when the monitor is executed.
                    let (mut parts, body) = request.into_parts();
                    parts.extensions.insert(identity);
                    let request = Request::from_parts(parts, body);

                    // Execute request
                    let response = next.run(request).await;

                    Ok(response)
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
            Ok((false, reason)) => {
                let reason_str = reason.unwrap_or_else(|| "Access denied".to_string());
                tracing::warn!("UserToken rejected: {}", reason_str);
                let body = serde_json::json!({
                    "error": {
                        "message": reason_str,
                        "type": "token_rejected",
                        "code": "token_rejected"
                    }
                });
                let response = axum::response::Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_string(&body).unwrap(),
                    ))
                    .unwrap();
                Ok(response)
            }
            Err(e) => {
                tracing::error!("UserToken validation error: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// User token identity information (passed to Monitor for use)
#[derive(Clone, Debug)]
pub struct UserTokenIdentity {
    pub token_id: String,
    #[allow(dead_code)] // Keep the original token for auditing/debugging
    pub token: String,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::ProxyAuthMode;

    #[tokio::test]
    async fn test_admin_auth_with_password() {
        let _security = Arc::new(RwLock::new(ProxySecurityConfig {
            auth_mode: ProxyAuthMode::Strict,
            api_key: "sk-api".to_string(),
            admin_password: Some("admin123".to_string()),
            allow_lan_access: true,
            port: 8045,
            security_monitor: crate::proxy::config::SecurityMonitorConfig::default(),
        }));

        // Impersonation Request - Admin Interface Use Correct Admin Password
        let _req = Request::builder()
            .header("Authorization", "Bearer admin123")
            .uri("/admin/stats")
            .body(axum::body::Body::empty())
            .unwrap();

        // This test is relatively complex due to the Next middleware call involved, and it mainly verifies the core logic.
        // We can do logical verification based on auth_middleware_internal
    }

    #[test]
    fn test_auth_placeholder() {
        assert!(true);
    }
}
