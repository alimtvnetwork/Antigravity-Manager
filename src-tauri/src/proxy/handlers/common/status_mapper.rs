//! Upstream HTTP status code mapper and canonical error payload synthesizer.
//! Translates provider-specific HTTP errors into uniform JSON error payloads.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalErrorPayload {
    pub error_type: String,
    pub message: String,
    pub status: u16,
    pub upstream_status: Option<u16>,
    pub retry_after_secs: Option<u64>,
}

/// Normalizes provider upstream response status codes into canonical gateway error representations.
pub fn map_upstream_status_to_canonical(
    upstream_status: u16,
    raw_message: &str,
) -> CanonicalErrorPayload {
    let clean_msg = if raw_message.trim().is_empty() {
        "Upstream provider error".to_string()
    } else {
        raw_message.to_string()
    };

    match upstream_status {
        400 => CanonicalErrorPayload {
            error_type: "invalid_request_error".to_string(),
            message: format!("Bad request to upstream: {}", clean_msg),
            status: 400,
            upstream_status: Some(400),
            retry_after_secs: None,
        },
        401 => CanonicalErrorPayload {
            error_type: "authentication_error".to_string(),
            message: "Authentication token invalid or expired; re-authenticate account".to_string(),
            status: 401,
            upstream_status: Some(401),
            retry_after_secs: None,
        },
        403 => CanonicalErrorPayload {
            error_type: "permission_denied_error".to_string(),
            message: "Access forbidden by upstream provider policy or project quota".to_string(),
            status: 403,
            upstream_status: Some(403),
            retry_after_secs: None,
        },
        404 => CanonicalErrorPayload {
            error_type: "not_found_error".to_string(),
            message: format!("Requested model or resource not found: {}", clean_msg),
            status: 404,
            upstream_status: Some(404),
            retry_after_secs: None,
        },
        429 => CanonicalErrorPayload {
            error_type: "rate_limit_exceeded".to_string(),
            message: "Upstream quota limit exceeded; automated fallback initiated".to_string(),
            status: 429,
            upstream_status: Some(429),
            retry_after_secs: Some(30),
        },
        502 | 503 | 504 => CanonicalErrorPayload {
            error_type: "upstream_unavailable".to_string(),
            message: format!(
                "Upstream service temporarily unreachable ({})",
                upstream_status
            ),
            status: 503,
            upstream_status: Some(upstream_status),
            retry_after_secs: Some(5),
        },
        other => CanonicalErrorPayload {
            error_type: "gateway_internal_error".to_string(),
            message: format!("Unhandled upstream error {}: {}", other, clean_msg),
            status: 500,
            upstream_status: Some(other),
            retry_after_secs: None,
        },
    }
}
