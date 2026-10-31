//! Request header sanitizer.
//! Filters out hop-by-hop, proxy-internal, and client identity headers before
//! upstream dispatch to prevent header leakage and upstream HTTP/2 protocol violations.

use axum::http::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashSet;

pub struct RequestHeaderSanitizer;

impl RequestHeaderSanitizer {
    /// Sanitizes an incoming HeaderMap, retaining only safe client and protocol headers.
    pub fn sanitize(incoming: &HeaderMap) -> HeaderMap {
        let mut clean = HeaderMap::new();

        // Disallowed headers (hop-by-hop, transport-level, or internal leakage)
        let blocked: HashSet<&'static str> = [
            "connection",
            "keep-alive",
            "proxy-authenticate",
            "proxy-authorization",
            "te",
            "trailers",
            "transfer-encoding",
            "upgrade",
            "host",
            "content-length", // Recalculated by client body
            "x-forwarded-for",
            "x-real-ip",
            "cookie",
        ]
        .into_iter()
        .collect();

        for (key, val) in incoming.iter() {
            let key_str = key.as_str().to_ascii_lowercase();

            // Skip blocked headers or internal agm telemetry headers
            if blocked.contains(key_str.as_str()) || key_str.starts_with("x-agm-") {
                continue;
            }

            clean.insert(key.clone(), val.clone());
        }

        clean
    }
}
