// Request identity and payload guards (split from common_utils.rs).
// Common utilities for request mapping across all protocols
// Provides unified grounding/networking logic

use serde_json::{json, Value};

/// Request configuration after grounding resolution

/// Check if two model strings are compatible (same family)
pub fn is_model_compatible(cached: &str, target: &str) -> bool {
    let c = cached.to_lowercase();
    let t = target.to_lowercase();

    if c == t {
        return true;
    }

    // Claude 全系列通用兼容：凡是同属 Claude 家族模型，直接判定签名兼容（面向未来任何 Claude 5/新模型及变体）
    if c.contains("claude") && t.contains("claude") {
        return true;
    }

    // Gemini models: strict family match required for signatures
    if c.contains("gemini-1.5-pro") && t.contains("gemini-1.5-pro") {
        return true;
    }
    if c.contains("gemini-1.5-flash") && t.contains("gemini-1.5-flash") {
        return true;
    }
    if c.contains("gemini-2.0-flash") && t.contains("gemini-2.0-flash") {
        return true;
    }
    if c.contains("gemini-2.0-pro") && t.contains("gemini-2.0-pro") {
        return true;
    }
    if c.contains("gemini-3") && t.contains("gemini-3") {
        let c_flash = c.contains("flash");
        let t_flash = t.contains("flash");
        let c_pro = c.contains("pro");
        let t_pro = t.contains("pro");
        if c_flash == t_flash && c_pro == t_pro {
            return true;
        }
        if c_flash && t_flash {
            return true;
        }
        if c_pro && t_pro {
            return true;
        }
    }
    if c.contains("gemini-3.7") && t.contains("gemini-3.7") {
        return true;
    }

    false
}

/// 解析官方客户端指纹，返回 `(userAgent, ideType)`。
///
/// 官方 Go Worker 中**企业 / GCP 账号**（非 `@gmail.com` / `@googlemail.com` 邮箱）
/// 使用 `jetski` 指纹，其余使用 `antigravity`。
///
/// **三个适配器必须共用本函数** —— 否则同一账号经不同协议入口会产出不同指纹。
/// 历史缺陷：jetski 仿真只实现在 Gemini 路径，Claude / OpenAI 路径硬编码
/// `"antigravity"`，导致企业账号发生指纹漂移。
pub fn resolve_official_fingerprint(
    token: Option<&crate::proxy::token_manager::ProxyToken>,
) -> (&'static str, &'static str) {
    let is_enterprise = token
        .map(|t| !t.email.ends_with("@gmail.com") && !t.email.ends_with("@googlemail.com"))
        .unwrap_or(false);
    if is_enterprise {
        ("jetski", "JETSKI")
    } else {
        ("antigravity", "ANTIGRAVITY")
    }
}

/// 构造官方形态的 requestId：`agent/{conversationId}/{unixMs}/{trajectoryId}/{step}`。
///
/// 官方样本（3 份报文逐字核对），例如：
///
/// ```text
/// agent/a89a2006-72b4-470d-8282-3f1e4c88dc29/1790410048596/d3a2e3d2-21f9-411b-967f-53811898c2cd/14
/// ```
///
/// **第 2 段 `unixMs` 每次请求都不同 → 天然的幂等隔离**，避免重试命中上一次的
/// 429 / 旧缓存。历史缺陷：Claude 路径曾用 `agent/antigravity/{session[:8]}/{count}`，
/// **不含时间戳** —— 同一会话同一轮次重试会拿到完全相同的 ID。
///
/// **三适配器必须共用本函数** —— 否则同一对话经不同入口会产出不同 ID 形态。
///
/// ## 与 `session_id` 的关系（重要）
///
/// 本函数**只读 `session_id`，绝不写它**，也**不影响**它的唯一性：
/// 防主子 agent 并发串话的机制是 `thinking_store::derive_blended_session_id`
/// 的 SHA256 多维正交哈希（tenant + 会话语义头 + query sid + body sid + anchor），
/// 它决定的是 thinking store / signature cache / prefix cache 的 key，
/// 与出站 requestId 是**两条独立通路**（全仓无任何代码从 requestId 反推 session）。
///
/// 会话段使用 `session_id` 的**单向哈希派生**而非原文：
/// - 同一会话稳定（贴近官方 conversationId 语义）；
/// - 不把网关内部 blended session_id 的原文暴露给上游；
/// - 不同 agent / 会话的 session_id 不同 → 派生值不同，隔离性保持。
pub fn build_official_request_id(session_id: &str, step: u64) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    let sanitized = crate::proxy::thinking_store::sanitize_session_id(session_id);
    let conversation = if sanitized.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
        sha2::Digest::update(&mut hasher, sanitized.as_bytes());
        let digest = sha2::Digest::finalize(hasher);
        let hex = format!("{:x}", digest);
        hex[..16].to_string()
    };
    // 轨迹段每请求唯一，与 unixMs 共同保证幂等隔离
    let trajectory = &uuid::Uuid::new_v4().simple().to_string()[..8];
    format!("agent/{}/{}/{}/{}", conversation, ts, trajectory, step)
}

/// [JEIKCODE SYNTHETIC USER REMINDER]
/// 将对话中途动态插入的系统消息就地包装为 `<system-reminder>` 标签块。
/// 提示词采用英文，明确告知模型：本内容为系统层注入的背景提醒，并非本轮用户输入，
/// 从而保证用户原始 query 完整透传，同时全局顶层 systemInstruction 保持绝对冻结以稳定 KV Cache。
pub fn wrap_in_system_reminder(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with("<system-reminder>") && trimmed.ends_with("</system-reminder>") {
        return trimmed.to_string();
    }
    format!(
        "<system-reminder>\nBefore the user's request for this turn, the system provides the following reminder for your awareness. Please note that this is from prior system messages, not spoken by the user:\n{}\n</system-reminder>",
        trimmed
    )
}

/// [DEFENSE] 通用中转报文保底文本（温和提示继续分析，避免触发 Agent 误进入修改阶段）
pub const TRANSIT_DEFENSE_FALLBACK_TEXT: &str = "Please continue your analysis.";

/// [DEFENSE] 通用中转报文保底防御节点（协议无关性）
/// 确保发给 Google Gemini 的报文末尾轮次严格符合规范：
/// 1. 自动兼容平铺 payload 或包含 "request" 包装的 payload；
/// 2. 若 contents 为空，追加 {"role": "user", "parts": [{"text": TRANSIT_DEFENSE_FALLBACK_TEXT}]}；
/// 3. 若末尾轮次为 "model"（缺失用户轮次），追加 {"role": "user", "parts": [{"text": TRANSIT_DEFENSE_FALLBACK_TEXT}]}；
///    例外：末尾 model 轮若携带 functionCall / functionResponse（模型主动发起的工具轮），
///    视为合法中间态，不注入（否则会与 normalize_function_response_roles 的 fr@model 对齐
///    打架，把官方合法报文误判为缺用户轮，注入"Please continue your analysis."造成工具死循环）；
/// 4. 若末尾轮次为 "user" 且其 parts 为空、或仅含有空文本 / "(no content)" / "·" 且无工具/图片，规范化填充为 [{"text": TRANSIT_DEFENSE_FALLBACK_TEXT}]；
/// 5. 修复中间轮次中 parts 为空的情况，防止 Google 返回 400 "parts must not be empty"。
pub fn ensure_gemini_payload_ends_with_user(body: &mut Value) -> bool {
    let contents = if let Some(contents) = body
        .get_mut("request")
        .and_then(|r| r.get_mut("contents"))
        .and_then(|c| c.as_array_mut())
    {
        contents
    } else if let Some(contents) = body.get_mut("contents").and_then(|c| c.as_array_mut()) {
        contents
    } else {
        return false;
    };

    let mut modified = false;

    // 防御 1: contents 整体为空
    if contents.is_empty() {
        tracing::warn!("[Defense] Gemini contents array is empty, appending fallback user turn");
        contents.push(json!({
            "role": "user",
            "parts": [{ "text": TRANSIT_DEFENSE_FALLBACK_TEXT }]
        }));
        return true;
    }

    // 防御 2: 修复历史/中间轮次中可能存在的 parts 为空
    for turn in contents.iter_mut() {
        let is_model = turn
            .get("role")
            .and_then(|r| r.as_str())
            .map(|r| r == "model" || r == "assistant")
            .unwrap_or(false);
        if let Some(parts) = turn.get_mut("parts").and_then(|p| p.as_array_mut()) {
            if parts.is_empty() {
                modified = true;
                if is_model {
                    parts.push(json!({ "text": "..." }));
                } else {
                    parts.push(json!({ "text": TRANSIT_DEFENSE_FALLBACK_TEXT }));
                }
            }
        }
    }

    // 防御 3: 检查末尾轮次。
    // 注意：InboundThinkingPipeline::normalize_function_response_roles 会把纯回执轮对齐为
    // role=model（官方 Antigravity 报文约定 fr 恒在 model 轮）。因此「末尾为 model 轮」
    // 并不代表报文不完整——若该轮携带 functionCall（模型主动发起工具轮，等待回执，
    // 属于合法的中间态），绝不能注入假用户话术，否则会放大成 Agent 工具死循环。
    let need_append_user = if let Some(last_turn) = contents.last_mut() {
        let role = last_turn.get("role").and_then(|r| r.as_str()).unwrap_or("");
        if role == "model" || role == "assistant" {
            // 工具轮合法：末尾 model 轮含 functionCall 或 functionResponse 时不注入
            let is_tool_turn = last_turn
                .get("parts")
                .and_then(|p| p.as_array())
                .map(|parts| {
                    parts.iter().any(|part| {
                        part.get("functionCall").is_some() || part.get("functionResponse").is_some()
                    })
                })
                .unwrap_or(false);
            !is_tool_turn
        } else {
            if let Some(parts) = last_turn.get_mut("parts").and_then(|p| p.as_array_mut()) {
                let has_substantive_part = parts.iter().any(|part| {
                    if part.get("functionCall").is_some()
                        || part.get("functionResponse").is_some()
                        || part.get("inlineData").is_some()
                        || part.get("fileData").is_some()
                    {
                        return true;
                    }
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        let t = text.trim();
                        !t.is_empty() && t != "(no content)" && t != "·"
                    } else {
                        false
                    }
                });

                if !has_substantive_part {
                    tracing::warn!(
                        "[Defense] Last user turn has no substantive content, normalizing to '{}'",
                        TRANSIT_DEFENSE_FALLBACK_TEXT
                    );
                    *parts = vec![json!({ "text": TRANSIT_DEFENSE_FALLBACK_TEXT })];
                    modified = true;
                }
            }
            false
        }
    } else {
        false
    };

    if need_append_user {
        tracing::warn!(
            "[Defense] Gemini payload ended with model turn, appending user turn with '{}'",
            TRANSIT_DEFENSE_FALLBACK_TEXT
        );
        contents.push(json!({
            "role": "user",
            "parts": [{ "text": TRANSIT_DEFENSE_FALLBACK_TEXT }]
        }));
        modified = true;
    }

    modified
}

/// 安全地按最大字节数截断字符串切片，保证切片边界严格对齐在 UTF-8 字符边界上。
/// 若 max_bytes 恰好落在多字节字符中间，会自动向左回退到最近的合法字符边界。
pub fn safe_truncate_str(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// 安全地按最大字符数 (Unicode 标量值) 截断字符串切片。
/// 如果字符总数超过 max_chars，截取前 max_chars 个字符对应的有效切片。
pub fn safe_truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => &s[..byte_idx],
        None => s,
    }
}
