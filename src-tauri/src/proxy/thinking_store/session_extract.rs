use axum::http::HeaderMap;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Extract session/conversation identifier from URL Query string
pub fn extract_session_from_query_str(query: &str) -> Option<String> {
    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        let key = k.to_ascii_lowercase();
        if matches!(
            key.as_str(),
            "session_id" | "sid" | "cid" | "conversation_id" | "chat_id" | "thread_id" | "channel"
        ) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                let sanitized = sanitize_session_id(trimmed);
                if !sanitized.is_empty() && sanitized != "sid-unknown" {
                    return Some(sanitized);
                }
            }
        }
    }
    None
}

/// Full-ecological explicit session ID resolution (including URL Query, Header extension and Body extension)
pub fn explicit_session_id_with_query(
    headers: &HeaderMap,
    body: Option<&Value>,
    query: Option<&str>,
) -> Option<String> {
    // 1. Explicit URL Query parameters (highest priority: user configuration Base URL direct mount?session_id=win1)
    if let Some(q) = query {
        if let Some(sid) = extract_session_from_query_str(q) {
            return Some(sid);
        }
    }

    // 2. Grab URL Query (x-forwarded-uri, x-original-uri) from the reverse request header
    for uri_h in ["x-forwarded-uri", "x-original-uri"] {
        if let Some(raw_uri) = headers.get(uri_h).and_then(|h| h.to_str().ok()) {
            if let Some(pos) = raw_uri.find('?') {
                if let Some(sid) = extract_session_from_query_str(&raw_uri[pos + 1..]) {
                    return Some(sid);
                }
            }
        }
    }

    // 3. Sniff the Query from the Web client Referer
    if let Some(referer) = headers.get("referer").and_then(|h| h.to_str().ok()) {
        if let Some(pos) = referer.find('?') {
            if let Some(sid) = extract_session_from_query_str(&referer[pos + 1..]) {
                return Some(sid);
            }
        }
    }

    // 4. Full ecological HTTP Headers: precise list first, then wildcard x-*-session-id / x-*-sessionid
    if let Some(sid) = session_id_from_headers(headers) {
        return Some(sid);
    }

    // 5. JSON Body and Metadata in-depth extraction
    if let Some(body) = body {
        for field in [
            "session_id",
            "conversation_id",
            "chat_id",
            "thread_id",
            "client_session_id",
            "previous_response_id",
        ] {
            if let Some(v) = body.get(field).and_then(|v| v.as_str()) {
                let v = v.trim();
                if !v.is_empty() {
                    return Some(sanitize_session_id(v));
                }
            }
        }
        if let Some(metadata) = body.get("metadata") {
            for field in [
                "conversation_id",
                "chat_id",
                "session_id",
                "thread_id",
                "user_id",
            ] {
                if let Some(v) = metadata.get(field).and_then(|v| v.as_str()) {
                    let v = v.trim();
                    if !v.is_empty() && !v.contains("session-") {
                        return Some(sanitize_session_id(v));
                    }
                }
            }
        }
    }

    None
}

pub fn explicit_session_id(headers: &HeaderMap, body: Option<&Value>) -> Option<String> {
    explicit_session_id_with_query(headers, body, None)
}

/// Product-specific `x-**-session-id` / `x-**-sessionid`. Checked before generic `x-session-id`.
const PRODUCT_SESSION_HEADERS: &[&str] = &[
    "x-jeikcode-sessionid",
    "x-jeikcode-session-id",
    "x-atomcode-session-id",
    "x-atomcode-sessionid",
    "x-antigravity-session-id",
    "x-client-session-id",
    "x-cursor-session-id",
    "cursor-session-id",
    "x-vscode-session-id",
    "anthropic-session-id",
];

const GENERIC_SESSION_HEADER: &str = "x-session-id";

/// Non-session-named aliases. Lowest header priority after `x-session-id`.
const ALIAS_SESSION_HEADERS: &[&str] = &[
    "x-conversation-id",
    "conversation-id",
    "x-chat-id",
    "chat-id",
    "x-thread-id",
    "thread-id",
];

fn header_session_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|h| h.to_str().ok())
        .and_then(|v| {
            let v = v.trim();
            if v.is_empty() {
                None
            } else {
                Some(sanitize_session_id(v))
            }
        })
}

fn is_generic_x_session_id(name: &str) -> bool {
    name.eq_ignore_ascii_case(GENERIC_SESSION_HEADER)
}

/// Compatible with client-side custom session headers such as AtomCode / JeikCode / Cursor:
/// Priority is given to `x-*-session-id` / `x-*-sessionid`, followed by `x-session-id`.
fn is_wildcard_session_header(name: &str) -> bool {
    let key = name.trim().to_ascii_lowercase().replace('_', "-");
    if key == "mcp-session-id" {
        return false;
    }
    if key.ends_with("-request-id")
        || key.ends_with("-trace-id")
        || key.ends_with("-correlation-id")
        || key == "x-request-id"
        || key == "request-id"
    {
        return false;
    }
    let compact = key.replace('-', "");
    compact.contains("session") && compact.ends_with("id")
}

fn session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    // 1. Product-specific x-**-session-id / x-**-sessionid
    for name in PRODUCT_SESSION_HEADERS {
        if let Some(sid) = header_session_value(headers, name) {
            return Some(sid);
        }
    }
    for (name, value) in headers.iter() {
        if is_generic_x_session_id(name.as_str()) {
            continue;
        }
        if !is_wildcard_session_header(name.as_str()) {
            continue;
        }
        if let Ok(v) = value.to_str() {
            let v = v.trim();
            if !v.is_empty() {
                return Some(sanitize_session_id(v));
            }
        }
    }

    // 2. Generic x-session-id
    if let Some(sid) = header_session_value(headers, GENERIC_SESSION_HEADER) {
        return Some(sid);
    }

    // 3. Other conversation/chat/thread aliases
    for name in ALIAS_SESSION_HEADERS {
        if let Some(sid) = header_session_value(headers, name) {
            return Some(sid);
        }
    }
    None
}

pub(crate) fn tenant_from_headers(headers: &HeaderMap) -> String {
    let raw = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").or(Some(s)))
        .or_else(|| headers.get("x-api-key").and_then(|h| h.to_str().ok()))
        .or_else(|| headers.get("x-goog-api-key").and_then(|h| h.to_str().ok()))
        .unwrap_or("anon");
    let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));
    hash[..16].to_string()
}

pub fn sanitize_session_id(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars().take(128) {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':') {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "sid-unknown".to_string()
    } else {
        out
    }
}

/// Determine whether the HTTP Header is a session semantic related header (variable random headers such as x-request-id, etc. are strictly excluded)
pub fn is_session_semantic_header(name: &str) -> bool {
    let key = name.trim().to_ascii_lowercase().replace('_', "-");
    if key == "mcp-session-id" {
        return false;
    }
    if key.ends_with("-request-id")
        || key.ends_with("-trace-id")
        || key.ends_with("-correlation-id")
        || key == "x-request-id"
        || key == "request-id"
        || key == "traceparent"
        || key == "tracestate"
        || key == "content-length"
        || key == "content-type"
        || key == "host"
        || key == "user-agent"
        || key.starts_with("sec-")
        || key.starts_with("cf-")
        || key.starts_with("x-forwarded-")
        || key.starts_with("x-real-")
    {
        return false;
    }

    if PRODUCT_SESSION_HEADERS.iter().any(|h| key == *h) {
        return true;
    }
    if ALIAS_SESSION_HEADERS.iter().any(|h| key == *h) {
        return true;
    }
    if key == GENERIC_SESSION_HEADER || key == "session-id" {
        return true;
    }

    let compact = key.replace('-', "");
    (compact.contains("session")
        || compact.contains("conversation")
        || compact.contains("chat")
        || compact.contains("thread"))
        && compact.ends_with("id")
}

/// Collect all HTTP Headers with session isolation semantics (keys are stored in BTreeMap in lexicographic order)
pub fn collect_session_semantic_headers(
    headers: &HeaderMap,
) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    for (name, val) in headers.iter() {
        let key = name.as_str().to_ascii_lowercase();
        if is_session_semantic_header(&key) {
            if let Ok(v) = val.to_str() {
                let trimmed = v.trim();
                if !trimmed.is_empty() {
                    let sanitized = sanitize_session_id(trimmed);
                    if !sanitized.is_empty() && sanitized != "sid-unknown" {
                        map.insert(key, sanitized);
                    }
                }
            }
        }
    }
    map
}

/// Extract session parameters from URL Query, proxy redirect header (x-forwarded-uri, x-original-uri) and referer
pub fn extract_query_session_id(headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    if let Some(q) = query {
        if let Some(sid) = extract_session_from_query_str(q) {
            return Some(sid);
        }
    }
    for uri_h in ["x-forwarded-uri", "x-original-uri"] {
        if let Some(raw_uri) = headers.get(uri_h).and_then(|h| h.to_str().ok()) {
            if let Some(pos) = raw_uri.find('?') {
                if let Some(sid) = extract_session_from_query_str(&raw_uri[pos + 1..]) {
                    return Some(sid);
                }
            }
        }
    }
    if let Some(referer) = headers.get("referer").and_then(|h| h.to_str().ok()) {
        if let Some(pos) = referer.find('?') {
            if let Some(sid) = extract_session_from_query_str(&referer[pos + 1..]) {
                return Some(sid);
            }
        }
    }
    None
}

/// Extract explicitly specified session fields from JSON Body and metadata
pub fn extract_body_session_id(body: Option<&Value>) -> Option<String> {
    let body = body?;
    for field in [
        "session_id",
        "conversation_id",
        "chat_id",
        "thread_id",
        "client_session_id",
        "previous_response_id",
    ] {
        if let Some(v) = body.get(field).and_then(|v| v.as_str()) {
            let v = v.trim();
            if !v.is_empty() {
                let sanitized = sanitize_session_id(v);
                if !sanitized.is_empty() && sanitized != "sid-unknown" {
                    return Some(sanitized);
                }
            }
        }
    }
    if let Some(metadata) = body.get("metadata") {
        for field in ["conversation_id", "chat_id", "session_id", "thread_id"] {
            if let Some(v) = metadata.get(field).and_then(|v| v.as_str()) {
                let v = v.trim();
                if !v.is_empty() && !v.contains("session-") {
                    let sanitized = sanitize_session_id(v);
                    if !sanitized.is_empty() && sanitized != "sid-unknown" {
                        return Some(sanitized);
                    }
                }
            }
        }
    }
    None
}

/// 3D orthogonal deterministic session obfuscated hash generation:
/// 1. Tenant Key
/// 2. Client-side explicit session semantic header collection (Sorted Session Headers + Query + Body)
/// 3. Session root anchor fingerprint (Fallback Root User Prompt + Full System Prompt + Tools)
pub fn derive_blended_session_id(
    tenant: &str,
    session_headers: &std::collections::BTreeMap<String, String>,
    query_sid: Option<&str>,
    body_sid: Option<&str>,
    fallback: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"v2|");
    hasher.update(tenant.as_bytes());
    hasher.update([0xff]);

    for (k, v) in session_headers {
        hasher.update(k.as_bytes());
        hasher.update(b"=");
        hasher.update(v.as_bytes());
        hasher.update([0xfe]);
    }

    if let Some(q) = query_sid {
        hasher.update(b"query=");
        hasher.update(q.as_bytes());
        hasher.update([0xfd]);
    }

    if let Some(b) = body_sid {
        hasher.update(b"body=");
        hasher.update(b.as_bytes());
        hasher.update([0xfc]);
    }

    let clean_fallback = fallback.trim();
    if !clean_fallback.is_empty() {
        hasher.update(b"anchor=");
        hasher.update(clean_fallback.as_bytes());
    }

    let hash = format!("{:x}", hasher.finalize());
    format!("sess-{}", &hash[..16])
}

pub(crate) fn client_id_from_store_key(store_key: &str) -> &str {
    store_key
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or(store_key)
}
