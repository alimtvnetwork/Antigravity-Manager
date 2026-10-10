use super::*;

/// Detects model capabilities and configuration
/// POST /v1/models/detect
pub async fn handle_detect_model(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let model_name = body.get("model").and_then(|v| v.as_str()).unwrap_or("");

    if model_name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing 'model' field").into_response();
    }

    // 1. Resolve mapping
    let mapped_model = crate::proxy::common::model_mapping::resolve_model_route(
        model_name,
        &*state.custom_mapping.read().await,
    );

    // 2. Resolve capabilities
    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        model_name,
        &mapped_model,
        &None, // We don't check tools for static capability detection
        None,  // size
        None,  // quality
        None,  // image_size
        None,  // body (not needed for static detection)
    );

    // 3. Construct response
    let mut response = json!({
        "model": model_name,
        "mapped_model": mapped_model,
        "type": config.request_type,
        "features": {
            "has_web_search": config.inject_google_search,
            "is_image_gen": config.request_type == "image_gen"
        }
    });

    if let Some(img_conf) = config.image_config {
        if let Some(obj) = response.as_object_mut() {
            obj.insert("config".to_string(), img_conf);
        }
    }

    Json(response).into_response()
}

/// [Issue #3414] 从形如 "All accounts limited. Wait 29s." 或其他明确冷却提示中解析等待秒数
pub fn extract_retry_after_seconds(error_text: &str) -> Option<u64> {
    if let Some(pos) = error_text.find("Wait ") {
        let rest = &error_text[pos + 5..];
        if let Some(s_pos) = rest.find('s') {
            if let Ok(sec) = rest[..s_pos].trim().parse::<u64>() {
                if sec > 0 {
                    return Some(sec);
                }
            }
        }
    }
    None
}

/// [Issue #3414] 统一构造带有 X-Mapped-Model、可选 X-Account-Email 以及 Retry-After 的 HeaderMap
pub fn build_token_error_headers<'a>(
    mapped_model: Option<&'a str>,
    account_email: Option<&'a str>,
    error_text: &str,
) -> axum::http::HeaderMap {
    use axum::http::header::{HeaderName, HeaderValue};
    let mut headers = axum::http::HeaderMap::new();

    if let Some(model) = mapped_model {
        if let Ok(val) = HeaderValue::from_str(model) {
            headers.insert(HeaderName::from_static("x-mapped-model"), val);
        }
    }
    if let Some(email) = account_email {
        if let Ok(val) = HeaderValue::from_str(email) {
            headers.insert(HeaderName::from_static("x-account-email"), val);
        }
    }
    if let Some(sec) = extract_retry_after_seconds(error_text) {
        if let Ok(val) = HeaderValue::from_str(&sec.to_string()) {
            headers.insert(axum::http::header::RETRY_AFTER, val);
        }
    }
    headers
}

/// 判断是否为模型不存在/不支持的错误
pub fn is_model_not_found_error(status: u16, body: &str) -> bool {
    if status == 404 {
        return true;
    }
    let lower = body.to_lowercase();
    lower.contains("model not found")
        || lower.contains("unknown model")
        || lower.contains("does not exist")
        || lower.contains("is not found")
        || lower.contains("unsupported model")
        || lower.contains("not found for api version")
        || lower.contains("publisher model")
        || lower.contains("model_not_found")
        || lower.contains("no such model")
        || lower.contains("invalid model")
        || lower.contains("model is not available")
}

/// 深度解析、剥离前缀与反转义上游错误，返回 (上游原始纯文本消息, 结构化解析对象)
pub fn parse_raw_upstream_error(error_text: &str) -> (String, serde_json::Value) {
    let trimmed = error_text.trim();
    // 1. 剥离可能附带的外层 HTTP 前缀，如 "HTTP 400: "、"HTTP 500: "、"All accounts exhausted. Last error: "
    let clean_str = if let Some(pos) = trimmed.find("HTTP ") {
        if let Some(colon_pos) = trimmed[pos..].find(": ") {
            trimmed[pos + colon_pos + 2..].trim()
        } else {
            trimmed
        }
    } else if let Some(stripped) = trimmed.strip_prefix("All accounts exhausted. Last error: ") {
        stripped.trim()
    } else {
        trimmed
    };

    // 2. 深度递归反转义解析 JSON
    let mut current_val: Option<serde_json::Value> = serde_json::from_str(clean_str).ok();
    // 如果解析出来的还是 string 且看起来像 JSON，尝试二次/三次解析（处理被双重转义的情况）
    for _ in 0..3 {
        if let Some(serde_json::Value::String(ref s)) = current_val {
            let s_trim = s.trim();
            if (s_trim.starts_with('{') && s_trim.ends_with('}'))
                || (s_trim.starts_with('[') && s_trim.ends_with(']'))
            {
                if let Ok(nested) = serde_json::from_str(s_trim) {
                    current_val = Some(nested);
                    continue;
                }
            }
        }
        break;
    }

    // 3. 如果当前值是个 Object，且它某个字段（例如 "response" 或 "raw"）又是嵌套 JSON 字符串，递归解析内部
    if let Some(mut parsed) = current_val {
        if let Some(obj) = parsed.as_object_mut() {
            for key in ["response", "raw", "error", "details", "message"] {
                if let Some(serde_json::Value::String(s)) = obj.get(key) {
                    let s_trim = s.trim();
                    if (s_trim.starts_with('{') && s_trim.ends_with('}'))
                        || (s_trim.starts_with('[') && s_trim.ends_with(']'))
                    {
                        if let Ok(nested) = serde_json::from_str::<serde_json::Value>(s_trim) {
                            obj.insert(key.to_string(), nested);
                        }
                    }
                }
            }
        }

        // 从结构化对象中提取原生的 message
        let raw_msg = if let Some(err_obj) = parsed.get("error") {
            if let Some(msg) = err_obj.get("message").and_then(|m| m.as_str()) {
                msg.to_string()
            } else if let Some(msg) = err_obj.as_str() {
                msg.to_string()
            } else {
                clean_str.to_string()
            }
        } else if let Some(msg) = parsed.get("message").and_then(|m| m.as_str()) {
            msg.to_string()
        } else if let Some(msg) = parsed.get("detail").and_then(|m| m.as_str()) {
            msg.to_string()
        } else {
            clean_str.to_string()
        };

        (raw_msg, parsed)
    } else {
        (
            clean_str.to_string(),
            serde_json::json!({ "raw": clean_str }),
        )
    }
}

pub(crate) fn map_status_code_to_gemini_status(status_code: u16) -> &'static str {
    match status_code {
        400 => "INVALID_ARGUMENT",
        401 => "UNAUTHENTICATED",
        403 => "PERMISSION_DENIED",
        404 => "NOT_FOUND",
        429 => "RESOURCE_EXHAUSTED",
        499 => "CANCELLED",
        500 => "INTERNAL",
        501 => "NOT_IMPLEMENTED",
        503 => "UNAVAILABLE",
        504 => "DEADLINE_EXCEEDED",
        _ => "UPSTREAM_ERROR",
    }
}

/// 格式化双轨制错误报文（四大标准协议原始字段散开 + 顶层/UI层不阉割双轨诊断）
pub fn build_dual_track_error(
    protocol: &str, // "claude", "openai", "responses", or "gemini"
    status_code: u16,
    model: &str,
    error_text: &str,
) -> serde_json::Value {
    let lower = error_text.to_lowercase();
    let is_internal_limited = lower.contains("all accounts limited")
        || lower.contains("no accounts available")
        || lower.contains("all accounts failed")
        || lower.contains("token pool is empty")
        || lower.contains("all accounts exhausted")
        || lower.contains("all accounts unhealthy");

    let is_not_found = is_model_not_found_error(status_code, error_text);

    // 1. 深度解析并反转义上游错误
    let (raw_upstream_msg, parsed_upstream) = parse_raw_upstream_error(error_text);

    let (readable_prefix, diagnosis, suggestion, err_type, err_code) = if is_internal_limited {
        (
            "【网关调度受限】".to_string(),
            format!(
                "网关本地账号池当前暂无可用账号或全部可用账号处于限流冷却中。调度详情: {}",
                error_text
            ),
            "请等待冷却结束（参考等待秒数），或在网关中添加更多正常账号。".to_string(),
            "rate_limit_error",
            "all_accounts_limited",
        )
    } else if is_not_found {
        (
                format!("【模型不存在】[{}]", model),
                format!("模型 [{}] 在上游端点不存在，或当前绑定的账号暂未开通该模型的访问权限。", model),
                "请核对模型名称，或在网关配置中的「自定义模型映射」将其重定向至可用模型（如 gemini-2.5-flash）。".to_string(),
                "invalid_request_error",
                "model_not_found",
            )
    } else if status_code == 429 || status_code == 529 {
        (
            format!("【上游限流 HTTP {}】", status_code),
            format!("模型 [{}] 触发上游配额耗尽或频率限制。", model),
            "请稍候自动恢复，或添加更多账号以分散并发请求。".to_string(),
            "rate_limit_error",
            "rate_limit_exceeded",
        )
    } else {
        (
            format!("【上游错误 HTTP {}】", status_code),
            format!("调用上游模型 [{}] 发生错误 (HTTP {})。", model, status_code),
            "请参考 upstream_error 中的详细字段排查原因。".to_string(),
            "api_error",
            "upstream_error",
        )
    };

    let readable_message = format!(
        "{} 网关诊断: {} 建议: {}",
        readable_prefix, diagnosis, suggestion
    );

    // 2. 确定散开到各自协议标准 message 字段的内容：
    //    若是网关自身限制（无上游参与），则使用网关诊断文本；
    //    若是上游发生的真实报错，100% 保持上游原始报错（raw_upstream_msg），绝不被网关硬编码覆盖！
    let effective_message = if is_internal_limited || raw_upstream_msg.trim().is_empty() {
        readable_message.clone()
    } else {
        raw_upstream_msg
    };

    let gateway_error_obj = serde_json::json!({
        "error_code": err_code,
        "model": model,
        "diagnosis": diagnosis,
        "suggestion": suggestion,
        "readable_summary": readable_message
    });

    let upstream_error_obj = serde_json::json!({
        "status": status_code,
        "response": parsed_upstream
    });

    match protocol {
        "claude" => serde_json::json!({
            "type": "error",
            "error": {
                "type": err_type,
                "code": err_code,
                "message": effective_message,
                "gateway_error": gateway_error_obj,
                "upstream_error": upstream_error_obj
            }
        }),
        "openai" => serde_json::json!({
            "error": {
                "message": effective_message,
                "type": err_type,
                "param": serde_json::Value::Null,
                "code": err_code,
                "gateway_error": gateway_error_obj,
                "upstream_error": upstream_error_obj
            }
        }),
        "responses" => serde_json::json!({
            "error": {
                "message": effective_message,
                "type": err_type,
                "code": err_code,
                "gateway_error": gateway_error_obj,
                "upstream_error": upstream_error_obj
            }
        }),
        "gemini" => {
            let upstream_status = parsed_upstream
                .get("error")
                .and_then(|e| e.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or_else(|| map_status_code_to_gemini_status(status_code));

            serde_json::json!({
                "error": {
                    "code": status_code,
                    "message": effective_message,
                    "status": upstream_status,
                    "gateway_error": gateway_error_obj,
                    "upstream_error": upstream_error_obj
                }
            })
        }
        _ => serde_json::json!({
            "error": {
                "message": effective_message,
                "type": err_type,
                "code": err_code,
                "gateway_error": gateway_error_obj,
                "upstream_error": upstream_error_obj
            }
        }),
    }
}

#[cfg(test)]
pub(crate) mod retry_after_tests {
    use super::*;

    #[test]
    fn test_extract_retry_after_seconds() {
        assert_eq!(
            extract_retry_after_seconds("All accounts limited. Wait 29s."),
            Some(29)
        );
        assert_eq!(
            extract_retry_after_seconds("Token error: All accounts limited. Wait 5s."),
            Some(5)
        );
        assert_eq!(extract_retry_after_seconds("Token pool is empty"), None);
        assert_eq!(
            extract_retry_after_seconds("All accounts failed or unhealthy."),
            None
        );
    }

    #[test]
    fn test_build_token_error_headers() {
        let headers = build_token_error_headers(
            Some("gemini-2.5-pro"),
            Some("test@example.com"),
            "All accounts limited. Wait 45s.",
        );
        assert_eq!(
            headers.get("x-mapped-model").unwrap().to_str().unwrap(),
            "gemini-2.5-pro"
        );
        assert_eq!(
            headers.get("x-account-email").unwrap().to_str().unwrap(),
            "test@example.com"
        );
        assert_eq!(headers.get("retry-after").unwrap().to_str().unwrap(), "45");

        let headers_no_wait = build_token_error_headers(
            Some("gemini-2.5-pro"),
            None,
            "All accounts failed or unhealthy.",
        );
        assert!(headers_no_wait.get("retry-after").is_none());
        assert_eq!(
            headers_no_wait
                .get("x-mapped-model")
                .unwrap()
                .to_str()
                .unwrap(),
            "gemini-2.5-pro"
        );
    }

    #[test]
    fn test_parse_raw_upstream_error_with_http_prefix_and_escapes() {
        let raw = r#"HTTP 400: {"error":{"code":400,"message":"API key not valid. Please pass a valid API key.","status":"INVALID_ARGUMENT"}}"#;
        let (msg, parsed) = parse_raw_upstream_error(raw);
        assert_eq!(msg, "API key not valid. Please pass a valid API key.");
        assert_eq!(parsed["error"]["code"], 400);
        assert_eq!(parsed["error"]["status"], "INVALID_ARGUMENT");
    }

    #[test]
    fn test_parse_raw_upstream_error_double_escaped() {
        let raw = r#""{\"error\":{\"code\":403,\"message\":\"User location not supported.\"}}""#;
        let (msg, parsed) = parse_raw_upstream_error(raw);
        assert_eq!(msg, "User location not supported.");
        assert_eq!(parsed["error"]["code"], 403);
    }

    #[test]
    fn test_build_dual_track_error_four_protocols() {
        let raw_err = r#"HTTP 404: {"error":{"code":404,"message":"models/gemini-not-exist is not found","status":"NOT_FOUND"}}"#;

        // 1. Claude
        let claude_res = build_dual_track_error("claude", 404, "gemini-not-exist", raw_err);
        assert_eq!(claude_res["type"], "error");
        // 标准协议底层字段散开为上游原始 message
        assert_eq!(
            claude_res["error"]["message"],
            "models/gemini-not-exist is not found"
        );
        assert_eq!(claude_res["error"]["code"], "model_not_found");
        // UI层双轨制诊断不阉割
        assert!(claude_res["error"]["gateway_error"]["diagnosis"].is_string());
        assert_eq!(claude_res["error"]["upstream_error"]["status"], 404);
        assert_eq!(
            claude_res["error"]["upstream_error"]["response"]["error"]["status"],
            "NOT_FOUND"
        );

        // 2. OpenAI
        let openai_res = build_dual_track_error("openai", 404, "gemini-not-exist", raw_err);
        assert_eq!(
            openai_res["error"]["message"],
            "models/gemini-not-exist is not found"
        );
        assert_eq!(openai_res["error"]["code"], "model_not_found");
        assert!(openai_res["error"]["gateway_error"].is_object());
        assert!(openai_res["error"]["upstream_error"].is_object());

        // 3. Responses
        let resp_res = build_dual_track_error("responses", 404, "gemini-not-exist", raw_err);
        assert_eq!(
            resp_res["error"]["message"],
            "models/gemini-not-exist is not found"
        );
        assert_eq!(resp_res["error"]["code"], "model_not_found");
        assert!(resp_res["error"]["gateway_error"].is_object());

        // 4. Gemini
        let gemini_res = build_dual_track_error("gemini", 404, "gemini-not-exist", raw_err);
        assert_eq!(gemini_res["error"]["code"], 404);
        assert_eq!(
            gemini_res["error"]["message"],
            "models/gemini-not-exist is not found"
        );
        assert_eq!(gemini_res["error"]["status"], "NOT_FOUND");
        assert!(gemini_res["error"]["gateway_error"].is_object());
        assert!(gemini_res["error"]["upstream_error"].is_object());
    }

    #[test]
    fn test_build_dual_track_error_internal_limited() {
        let raw_err = "All accounts limited. Wait 30s.";
        let claude_res = build_dual_track_error("claude", 429, "gemini-2.5-flash", raw_err);
        assert_eq!(claude_res["error"]["code"], "all_accounts_limited");
        assert!(claude_res["error"]["message"]
            .as_str()
            .unwrap()
            .contains("网关调度受限"));
    }
}
