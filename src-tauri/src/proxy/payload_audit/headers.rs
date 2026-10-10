use super::*;

const THINKING_SESSION_HEADERS: &[&str] = &[
    "x-session-id",
    "x-antigravity-session-id",
    "x-conversation-id",
    "conversation-id",
    "x-chat-id",
    "chat-id",
    "x-thread-id",
    "thread-id",
    "x-client-session-id",
    "x-cursor-session-id",
    "cursor-session-id",
    "x-vscode-session-id",
    "anthropic-session-id",
    "mcp-session-id",
];

fn is_thinking_session_header(name: &str) -> bool {
    THINKING_SESSION_HEADERS
        .iter()
        .any(|h| name.eq_ignore_ascii_case(h))
}

fn is_sensitive_header(name: &str) -> bool {
    if is_thinking_session_header(name) {
        return false;
    }
    let n = name.to_ascii_lowercase();
    matches!(
        n.as_str(),
        "authorization"
            | "proxy-authorization"
            | "x-api-key"
            | "api-key"
            | "x-goog-api-key"
            | "anthropic-api-key"
            | "x-auth-token"
            | "x-access-token"
            | "cookie"
            | "set-cookie"
    ) || n.contains("api-key")
        || n.contains("apikey")
        || n.contains("access-token")
        || n.contains("access_token")
        || if n.contains("token") {
            let is_session_or_count = n.contains("session") || n.contains("count");
            !is_session_or_count
        } else {
            false
        }
}

pub fn redact_header_value(name: &str, value: &str) -> String {
    if is_thinking_session_header(name) {
        return value.to_string();
    }
    if !is_sensitive_header(name) {
        return value.to_string();
    }
    let trimmed = value.trim();
    if trimmed
        .get(..7)
        .map_or(false, |p| p.eq_ignore_ascii_case("bearer "))
    {
        return "Bearer ***REDACTED***".to_string();
    }
    "***REDACTED***".to_string()
}

pub fn headers_to_redacted_json(headers: &HeaderMap) -> String {
    header_pairs_to_redacted_json(
        headers
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str(), s))),
    )
}

pub fn header_pairs_to_redacted_json<'a, I>(pairs: I) -> String
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut map = Map::new();
    for (key, raw) in pairs {
        let redacted = redact_header_value(key, raw);
        match map.get_mut(key) {
            Some(Value::Array(arr)) => arr.push(json!(redacted)),
            Some(existing) => {
                let prev = existing.clone();
                *existing = json!([prev, redacted]);
            }
            None => {
                map.insert(key.to_string(), json!(redacted));
            }
        }
    }
    serde_json::to_string(&Value::Object(map)).unwrap_or_else(|_| "{}".to_string())
}
