//! Pipeline Response Header Sanitizer (`Imp-685`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First:
//! "The pipeline stage uniformly handles the converted Gemini payloads, including thinking
//! block backfilling, thinking budget filtering and backfilling, unified context structural
//! alignment, prefix stability, and the sanitization of risky prompts and request headers."
//!
//! Sanitizes upstream LLM provider response headers (stripping internal tracking headers,
//! rate-limit leakages, cookies, server tokens, and hop-by-hop metadata) while preserving
//! and normalizing required Antigravity gateway streaming response headers
//! (`content-type: text/event-stream; charset=utf-8`, `cache-control: no-cache`).

use axum::http::header::{HeaderName, HeaderValue};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Categorization of header disposition during response sanitization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeaderDisposition {
    /// Safe header preserved in final output.
    Keep,
    /// Stripped internal telemetry, safety scores, quota, or backend server metadata.
    StripInternal,
    /// Stripped RFC hop-by-hop transport header.
    StripHopByHop,
    /// Standardized / injected Antigravity gateway header.
    NormalizeStandard,
}

impl fmt::Display for HeaderDisposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keep => write!(f, "Keep"),
            Self::StripInternal => write!(f, "StripInternal"),
            Self::StripHopByHop => write!(f, "StripHopByHop"),
            Self::NormalizeStandard => write!(f, "NormalizeStandard"),
        }
    }
}

/// Audit finding for an individual header during response sanitization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderSanitizationFinding {
    pub header_name: String,
    pub header_value: String,
    pub disposition: HeaderDisposition,
}

/// Hop-by-hop transport headers forbidden in response payloads.
pub const HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
    "alt-svc",
];

/// Upstream headers strictly stripped from gateway response payloads to avoid leaking server internals.
pub const FORBIDDEN_INTERNAL_HEADERS: &[&str] = &[
    "server",
    "set-cookie",
    "set-cookie2",
    "via",
    "x-cloud-trace-context",
    "x-envoy-upstream-service-time",
    "x-goog-safety-ratings",
    "x-goog-ratelimit-remaining",
    "x-goog-quota-project",
];

pub const DEFAULT_STREAM_CONTENT_TYPE: &str = "text/event-stream; charset=utf-8";
pub const DEFAULT_CACHE_CONTROL: &str = "no-cache";

/// Checks if a lowercased header name is an RFC hop-by-hop transport header.
pub fn is_disallowed_hop_by_hop(lower_name: &str) -> bool {
    HOP_BY_HOP_HEADERS.iter().any(|&item| item == lower_name)
}

/// Checks if a lowercased header name is an internal telemetry or server metadata header.
pub fn is_disallowed_internal(lower_name: &str) -> bool {
    FORBIDDEN_INTERNAL_HEADERS
        .iter()
        .any(|&item| item == lower_name)
        || lower_name.starts_with("x-goog-")
        || lower_name.starts_with("x-agm-")
        || lower_name.starts_with("x-internal-")
}

/// Backwards-compatible check if a header key belongs to forbidden response headers.
pub fn is_forbidden_response_header(key: &str) -> bool {
    let lower = key.trim().to_ascii_lowercase();
    is_disallowed_hop_by_hop(&lower) || is_disallowed_internal(&lower)
}

/// Classifies a header name into its appropriate disposition category.
pub fn classify_header(lower_name: &str) -> HeaderDisposition {
    if is_disallowed_hop_by_hop(lower_name) {
        HeaderDisposition::StripHopByHop
    } else if is_disallowed_internal(lower_name) {
        HeaderDisposition::StripInternal
    } else {
        HeaderDisposition::Keep
    }
}

/// Helper function to serialize HeaderMap as a map of strings.
fn serialize_header_map<S>(
    headers: &axum::http::HeaderMap,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(headers.len()))?;
    for (key, val) in headers {
        if let Ok(v_str) = val.to_str() {
            map.serialize_entry(key.as_str(), v_str)?;
        }
    }
    map.end()
}

/// Helper function to deserialize HeaderMap from a map of strings.
fn deserialize_header_map<'de, D>(deserializer: D) -> Result<axum::http::HeaderMap, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let map = std::collections::BTreeMap::<String, String>::deserialize(deserializer)?;
    let mut headers = axum::http::HeaderMap::new();
    for (k, v) in map {
        if let Ok(name) = axum::http::header::HeaderName::from_bytes(k.as_bytes()) {
            if let Ok(val) = axum::http::header::HeaderValue::from_str(&v) {
                headers.insert(name, val);
            }
        }
    }
    Ok(headers)
}

/// Complete report produced by response header sanitization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseHeaderSanitizationReport {
    pub stripped_headers: Vec<HeaderSanitizationFinding>,
    pub normalized_headers: Vec<HeaderSanitizationFinding>,
    #[serde(
        serialize_with = "serialize_header_map",
        deserialize_with = "deserialize_header_map"
    )]
    pub final_headers: axum::http::HeaderMap,
}

impl ResponseHeaderSanitizationReport {
    /// True if no headers were stripped during sanitization.
    pub fn is_clean(&self) -> bool {
        self.stripped_headers.is_empty()
    }

    /// True if any headers were stripped.
    pub fn has_stripped(&self) -> bool {
        !self.stripped_headers.is_empty()
    }

    /// True if any headers were normalized or injected.
    pub fn has_normalized(&self) -> bool {
        !self.normalized_headers.is_empty()
    }

    /// Count of headers stripped.
    pub fn stripped_count(&self) -> usize {
        self.stripped_headers.len()
    }

    /// Count of headers normalized or injected.
    pub fn normalized_count(&self) -> usize {
        self.normalized_headers.len()
    }

    /// Look up a header value by key in `final_headers` as string.
    pub fn get_final_header_str(&self, key: &str) -> Option<&str> {
        self.final_headers.get(key).and_then(|v| v.to_str().ok())
    }
}

/// Sanitizes response headers received from upstream providers.
///
/// Strips hop-by-hop headers (`connection`, `keep-alive`, `transfer-encoding`, `upgrade`,
/// `alt-svc`, `set-cookie`, `server`, and internal `x-goog-*` metrics) while preserving
/// required Antigravity headers (`content-type: text/event-stream; charset=utf-8`,
/// `cache-control: no-cache`, etc.).
pub fn sanitize_response_headers(
    headers: &axum::http::HeaderMap,
) -> ResponseHeaderSanitizationReport {
    let mut stripped_headers = Vec::new();
    let mut normalized_headers = Vec::new();
    let mut final_headers = HeaderMap::new();

    let mut has_content_type = false;
    let mut has_cache_control = false;

    for (key, val) in headers {
        let key_str = key.as_str();
        let key_lower = key_str.to_ascii_lowercase();
        let val_str = match val.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::from_utf8_lossy(val.as_bytes()).to_string(),
        };

        if is_disallowed_hop_by_hop(&key_lower) {
            stripped_headers.push(HeaderSanitizationFinding {
                header_name: key_str.to_string(),
                header_value: val_str,
                disposition: HeaderDisposition::StripHopByHop,
            });
            continue;
        }

        if is_disallowed_internal(&key_lower) {
            stripped_headers.push(HeaderSanitizationFinding {
                header_name: key_str.to_string(),
                header_value: val_str,
                disposition: HeaderDisposition::StripInternal,
            });
            continue;
        }

        if key_lower == "content-type" {
            has_content_type = true;
            let val_lower = val_str.to_ascii_lowercase();
            let target_ct = if val_lower.contains("text/event-stream") {
                DEFAULT_STREAM_CONTENT_TYPE
            } else if val_lower.contains("application/json") {
                "application/json; charset=utf-8"
            } else {
                &val_str
            };

            if val_str != target_ct {
                normalized_headers.push(HeaderSanitizationFinding {
                    header_name: key_str.to_string(),
                    header_value: target_ct.to_string(),
                    disposition: HeaderDisposition::NormalizeStandard,
                });
                if let Ok(v) = HeaderValue::from_str(target_ct) {
                    final_headers.insert(HeaderName::from_static("content-type"), v);
                }
            } else {
                final_headers.insert(key.clone(), val.clone());
            }
            continue;
        }

        if key_lower == "cache-control" {
            has_cache_control = true;
            let val_lower = val_str.to_ascii_lowercase();
            let target_cc = if val_lower == "no-cache" || val_lower == "no-cache, no-transform" {
                &val_str
            } else {
                DEFAULT_CACHE_CONTROL
            };

            if val_str != target_cc {
                normalized_headers.push(HeaderSanitizationFinding {
                    header_name: key_str.to_string(),
                    header_value: target_cc.to_string(),
                    disposition: HeaderDisposition::NormalizeStandard,
                });
                if let Ok(v) = HeaderValue::from_str(target_cc) {
                    final_headers.insert(HeaderName::from_static("cache-control"), v);
                }
            } else {
                final_headers.insert(key.clone(), val.clone());
            }
            continue;
        }

        final_headers.insert(key.clone(), val.clone());
    }

    if !has_content_type {
        normalized_headers.push(HeaderSanitizationFinding {
            header_name: "content-type".to_string(),
            header_value: DEFAULT_STREAM_CONTENT_TYPE.to_string(),
            disposition: HeaderDisposition::NormalizeStandard,
        });
        final_headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static(DEFAULT_STREAM_CONTENT_TYPE),
        );
    }

    if !has_cache_control {
        normalized_headers.push(HeaderSanitizationFinding {
            header_name: "cache-control".to_string(),
            header_value: DEFAULT_CACHE_CONTROL.to_string(),
            disposition: HeaderDisposition::NormalizeStandard,
        });
        final_headers.insert(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static(DEFAULT_CACHE_CONTROL),
        );
    }

    ResponseHeaderSanitizationReport {
        stripped_headers,
        normalized_headers,
        final_headers,
    }
}
