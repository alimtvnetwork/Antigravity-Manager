// Normalize a Responses-format body into Chat Completions shape (prelude phase).
use serde_json::{json, Value};
use tracing::debug;

/// Detect Responses API format (`instructions`/`input` without `messages`)
/// and convert it to Chat Completions format in place.
pub(crate) fn apply_responses_format_fallback(body: &mut Value) {
    // [NEW] 自动检测并转换 Responses 格式
    // 如果请求包含 instructions 或 input 但没有 messages，则认为是 Responses 格式
    let is_responses_format = !body.get("messages").is_some()
        && (body.get("instructions").is_some() || body.get("input").is_some());

    if is_responses_format {
        debug!("Detected Responses API format, converting to Chat Completions format");

        // 转换 instructions 为 system message
        if let Some(instructions) = body.get("instructions").and_then(|v| v.as_str()) {
            if !instructions.is_empty() {
                let system_msg = json!({
                    "role": "system",
                    "content": instructions
                });

                // 初始化 messages 数组
                if !body.get("messages").is_some() {
                    body["messages"] = json!([]);
                }

                // 将 system message 插入到开头
                if let Some(messages) = body.get_mut("messages").and_then(|v| v.as_array_mut()) {
                    messages.insert(0, system_msg);
                }
            }
        }

        // 转换 input 为 user message（如果存在）
        if let Some(input) = body.get("input") {
            let user_msg = if input.is_string() {
                json!({
                    "role": "user",
                    "content": input.as_str().unwrap_or("")
                })
            } else {
                // input 是数组格式，暂时简化处理
                json!({
                    "role": "user",
                    "content": input.to_string()
                })
            };

            if let Some(messages) = body.get_mut("messages").and_then(|v| v.as_array_mut()) {
                messages.push(user_msg);
            }
        }

        if let Some(obj) = body.as_object_mut() {
            obj.remove("instructions");
        }
    }
}
