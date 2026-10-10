use super::*;

/// Log the inbound Claude request summary + full debug dump.
/// Extracted from `handle_messages`. Pure function, no mutation, no logic changes.
pub(crate) fn log_request_summary(
    request: &crate::proxy::mappers::claude::models::ClaudeRequest,
    trace_id: &str,
) {
    // 获取最新一条“有意义”的消息内容（用于日志记录和后台任务检测）
    // 策略：反向遍历，首先筛选出所有角色为 "user" 的消息，然后从中找到第一条非 "Warmup" 且非空的文本消息
    // 获取最新一条“有意义”的消息内容（用于日志记录和后台任务检测）
    // 策略：反向遍历，首先筛选出所有和用户相关的消息 (role="user")
    // 然后提取其文本内容，跳过 "Warmup" 或系统预设的 reminder
    let meaningful_msg = request
        .messages
        .iter()
        .rev()
        .filter(|m| m.role == "user")
        .find_map(|m| {
            let content = match &m.content {
                crate::proxy::mappers::claude::models::MessageContent::String(s) => s.to_string(),
                crate::proxy::mappers::claude::models::MessageContent::Array(arr) => {
                    // 对于数组，提取所有 Text 块并拼接，忽略 ToolResult
                    arr.iter()
                        .filter_map(|block| match block {
                            crate::proxy::mappers::claude::models::ContentBlock::Text { text } => {
                                Some(text.as_str())
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            };

            // 过滤规则：
            // 1. 忽略空消息
            // 2. 忽略 "Warmup" 消息
            // 3. 忽略 <system-reminder> 标签的消息
            if content.trim().is_empty()
                || content.starts_with("Warmup")
                || content.contains("<system-reminder>")
            {
                None
            } else {
                Some(content)
            }
        });

    // 如果经过过滤还是找不到（例如纯工具调用），则回退到最后一条消息的原始展示
    let latest_msg = meaningful_msg.unwrap_or_else(|| {
        request
            .messages
            .last()
            .map(|m| match &m.content {
                crate::proxy::mappers::claude::models::MessageContent::String(s) => s.clone(),
                crate::proxy::mappers::claude::models::MessageContent::Array(_) => {
                    "[Complex/Tool Message]".to_string()
                }
            })
            .unwrap_or_else(|| "[No Messages]".to_string())
    });

    // INFO 级别: 简洁的一行摘要
    info!(
        "[{}] Claude Request | Model: {} | Stream: {} | Messages: {} | Tools: {}",
        trace_id,
        request.model,
        request.stream,
        request.messages.len(),
        request.tools.is_some()
    );

    // DEBUG 级别: 详细的调试信息
    debug!(
        "========== [{}] CLAUDE REQUEST DEBUG START ==========",
        trace_id
    );
    debug!("[{}] Model: {}", trace_id, request.model);
    debug!("[{}] Stream: {}", trace_id, request.stream);
    debug!("[{}] Max Tokens: {:?}", trace_id, request.max_tokens);
    debug!("[{}] Temperature: {:?}", trace_id, request.temperature);
    debug!("[{}] Message Count: {}", trace_id, request.messages.len());
    debug!("[{}] Has Tools: {}", trace_id, request.tools.is_some());
    debug!(
        "[{}] Has Thinking Config: {}",
        trace_id,
        request.thinking.is_some()
    );
    debug!("[{}] Content Preview: {:.100}...", trace_id, latest_msg);

    // 输出每一条消息的详细信息
    for (idx, msg) in request.messages.iter().enumerate() {
        let content_preview = match &msg.content {
            crate::proxy::mappers::claude::models::MessageContent::String(s) => {
                let char_count = s.chars().count();
                if char_count > 200 {
                    // 【修复】使用 chars().take() 安全截取，避免 UTF-8 字符边界 panic
                    let preview: String = s.chars().take(200).collect();
                    format!("{}... (total {} chars)", preview, char_count)
                } else {
                    s.clone()
                }
            }
            crate::proxy::mappers::claude::models::MessageContent::Array(arr) => {
                format!("[Array with {} blocks]", arr.len())
            }
        };
        debug!(
            "[{}] Message[{}] - Role: {}, Content: {}",
            trace_id, idx, msg.role, content_preview
        );
    }

    debug!(
        "[{}] Full Claude Request JSON: {}",
        trace_id,
        serde_json::to_string_pretty(&request).unwrap_or_default()
    );
    debug!(
        "========== [{}] CLAUDE REQUEST DEBUG END ==========",
        trace_id
    );
}
