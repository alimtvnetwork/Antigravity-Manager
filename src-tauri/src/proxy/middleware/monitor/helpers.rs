use super::*;

/// Small generic utilities for request/response logging.
///
/// Pure code motion from `monitor.rs` — no logic changes.
pub(crate) async fn next_chunk_while_receiver_open<S, T>(
    stream: &mut S,
    tx: &tokio::sync::mpsc::Sender<T>,
) -> Option<S::Item>
where
    S: Stream + Unpin,
{
    tokio::select! {
        biased;
        _ = tx.closed() => None,
        chunk = stream.next() => chunk,
    }
}

pub(crate) fn truncate_for_log(value: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in value.chars().enumerate() {
        if idx >= max_chars {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
    out
}

pub(crate) fn extract_quoted_param(header: &str, key: &str) -> Option<String> {
    let needle = format!("{}=\"", key);
    let start = header.find(&needle)? + needle.len();
    let rest = &header[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(crate) fn extract_boundary(content_type: &str) -> Option<String> {
    content_type.split(';').find_map(|part| {
        let trimmed = part.trim();
        let value = trimmed.strip_prefix("boundary=")?;
        Some(value.trim_matches('"').to_string())
    })
}

/// 噪音请求日志抑制判定。
///
/// 面板「捕获健康请求」胶囊开关（`capture_health_logs`）**默认关闭**。关闭时：
/// **所有 GET 成功请求一律不记录、不落库**（含 `/v1/models`、`/v1beta/models`、`/v1/models/claude`
/// 等模型列表轮询，以及 `/health` `/healthz` `/api/health` 探针）。
///
/// 之所以从"仅过滤 `/health` 探针"放宽到"全部 GET 成功"：真实客户端的健康/能力探测会高频轮询
/// **模型列表**接口（issue #3498 的探针刷屏 + 面板中 `/v1/models` 200 连片淹没业务日志），
/// 这类请求没有对话语义、无业务价值，却会把真实业务日志挤出视图。
///
/// 边界（刻意保守，绝不影响排障）：
/// - 失败请求（非 2xx）**始终记录**，无论路径与方法；
/// - 非 GET（POST 等真实业务请求）**始终记录**；
/// - 开关开启（或环境变量 `ABV_LOG_HEALTH_CHECKS` 为真）时全部记录并落库。

pub(crate) fn should_skip_request_log(
    method: &str,
    status: axum::http::StatusCode,
    capture_enabled: bool,
) -> bool {
    !capture_enabled && method.eq_ignore_ascii_case("GET") && status.is_success()
}

pub(crate) fn should_log_health_checks() -> bool {
    std::env::var("ABV_LOG_HEALTH_CHECKS")
        .map(|val| {
            let v = val.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        })
        .unwrap_or(false)
}

pub(crate) fn find_subslice(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|idx| start + idx)
}

pub(crate) fn trim_part_tail(mut part: &[u8]) -> &[u8] {
    if part.ends_with(b"\r\n") {
        part = &part[..part.len() - 2];
    } else if part.ends_with(b"\n") {
        part = &part[..part.len() - 1];
    }
    part
}

pub(crate) fn record_user_token_usage(
    user_token_identity: &Option<UserTokenIdentity>,
    log: &ProxyRequestLog,
    user_agent: Option<String>,
) {
    if let Some(identity) = user_token_identity {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::user_token_db::record_token_usage_and_ip(
                &identity.token_id,
                log.client_ip.as_deref().unwrap_or("127.0.0.1"),
                log.model.as_deref().unwrap_or("unknown"),
                log.input_tokens.unwrap_or(0) as i32,
                log.output_tokens.unwrap_or(0) as i32,
                log.status as u16,
                user_agent,
            ),
            "record_token_usage_and_ip",
        );
    }
}

pub(crate) fn extract_cached_tokens(usage: &Value) -> Option<u32> {
    let c = crate::proxy::pipeline::CanonicalUsage::from_gemini(usage);
    if c.cached_tokens > 0 {
        Some(c.cached_tokens)
    } else {
        None
    }
}

pub(crate) fn extract_input_tokens(usage: &Value) -> Option<u32> {
    let has_field = usage.get("prompt_tokens").is_some()
        || usage.get("input_tokens").is_some()
        || usage.get("total_input_tokens").is_some()
        || usage.get("promptTokenCount").is_some();
    if !has_field {
        return None;
    }
    let c = crate::proxy::pipeline::CanonicalUsage::from_gemini(usage);
    Some(c.total_input_tokens)
}

pub(crate) fn extract_reasoning_tokens(usage: &Value) -> Option<u32> {
    let c = crate::proxy::pipeline::CanonicalUsage::from_gemini(usage);
    if c.reasoning_tokens > 0 {
        Some(c.reasoning_tokens)
    } else {
        None
    }
}

pub(crate) fn extract_output_tokens(usage: &Value) -> Option<u32> {
    let has_field = usage.get("completion_tokens").is_some()
        || usage.get("output_tokens").is_some()
        || usage.get("total_output_tokens").is_some()
        || usage.get("candidatesTokenCount").is_some();
    if !has_field {
        return None;
    }
    let c = crate::proxy::pipeline::CanonicalUsage::from_gemini(usage);
    Some(c.output_tokens)
}
