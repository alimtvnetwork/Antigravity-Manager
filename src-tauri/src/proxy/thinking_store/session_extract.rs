use axum::http::HeaderMap;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// 从 URL Query 字符串中提取 session / conversation 标识符
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

/// 全生态显式会话标识解析（包含 URL Query、Header 扩展与 Body 扩展）
pub fn explicit_session_id_with_query(
    headers: &HeaderMap,
    body: Option<&Value>,
    query: Option<&str>,
) -> Option<String> {
    // 1. 显式 URL Query 参数（最高优先级：用户配置 Base URL 直接挂载 ?session_id=win1）
    if let Some(q) = query {
        if let Some(sid) = extract_session_from_query_str(q) {
            return Some(sid);
        }
    }

    // 2. 从反代请求头中抓取 URL Query (x-forwarded-uri, x-original-uri)
    for uri_h in ["x-forwarded-uri", "x-original-uri"] {
        if let Some(raw_uri) = headers.get(uri_h).and_then(|h| h.to_str().ok()) {
            if let Some(pos) = raw_uri.find('?') {
                if let Some(sid) = extract_session_from_query_str(&raw_uri[pos + 1..]) {
                    return Some(sid);
                }
            }
        }
    }

    // 3. 从 Web 客户端 Referer 中嗅探 Query
    if let Some(referer) = headers.get("referer").and_then(|h| h.to_str().ok()) {
        if let Some(pos) = referer.find('?') {
            if let Some(sid) = extract_session_from_query_str(&referer[pos + 1..]) {
                return Some(sid);
            }
        }
    }

    // 4. 全生态 HTTP Headers：先精确名单，再通配 x-*-session-id / x-*-sessionid
    if let Some(sid) = session_id_from_headers(headers) {
        return Some(sid);
    }

    // 5. JSON Body 及 Metadata 深度提取
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

/// 兼容 AtomCode / JeikCode / Cursor 等客户端自定义会话头：
/// 优先 `x-*-session-id` / `x-*-sessionid`，其次通用 `x-session-id`。
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

/// 判断 HTTP Header 是否属于会话语义相关头（严格排除易变随机头如 x-request-id 等）
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

/// 收集所有具有会话隔离语义的 HTTP Header（键按字典序保存在 BTreeMap 中）
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

/// 从 URL Query、代理跳转 Header (x-forwarded-uri, x-original-uri) 以及 Referer 中提取会话参数
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

/// 从 JSON Body 及 metadata 中提取显式指定的会话字段
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

/// 3D 正交确定性会话混淆哈希生成：
/// 1. 租户隔离 (Tenant Key)
/// 2. 客户端显式会话语义头集合 (Sorted Session Headers + Query + Body)
/// 3. 会话根锚点指纹 (Fallback Root User Prompt + Full System Prompt + Tools)
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
