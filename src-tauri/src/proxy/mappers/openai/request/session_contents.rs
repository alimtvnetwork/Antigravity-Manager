// Contents building phase (split from request.rs).
// OpenAI → Gemini 请求转换
use super::super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

pub(crate) fn phase_contents(
    request: &super::super::models::OpenAIRequest,
    mapped_model: &str,
    is_responses_api: bool,
    actual_include_thinking: bool,
    tool_id_to_name: &std::collections::HashMap<String, String>,
    tool_name_to_schema: &std::collections::HashMap<String, serde_json::Value>,
    session_id: &str,
) -> Vec<serde_json::Value> {
    // 2. 构建 Gemini contents (过滤掉已作为 leading system 的指令，中途 system 消息就地转为 user 保持前缀)
    let leading_system_count = request
        .messages
        .iter()
        .take_while(|m| m.role == "system" || m.role == "developer")
        .count();

    // 找出 messages 中最后一个 assistant 角色的下标 (绝对索引)
    let _last_assistant_msg_idx = request
        .messages
        .iter()
        .enumerate()
        .rposition(|(_, m)| m.role == "assistant");

    let contents: Vec<Value> = request
        .messages
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx >= leading_system_count)
        .map(|(msg_index, msg)| {
            let role = match msg.role.as_str() {
                "assistant" => "model",
                "tool" | "function" => "user",
                "system" | "developer" => "user",
                _ => &msg.role,
            };

            let mut parts = Vec::new();

            let client_reasoning = msg
                .reasoning_content
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty());

            if role == "model" {
                // [2026-09-27] 占位/缺失 reasoning 不再写 "..." 思考块。
                // 官方样本 9/24 轮是「无思考块 + 锚点带签名」，空/占位思考不出站；
                // 签名归位由流水线终审 place_turn_signature 按锚点规则处理。
                if actual_include_thinking {
                    if let Some(rc) = client_reasoning {
                        if !crate::proxy::thinking_store::is_placeholder_thought(rc) {
                            // 纯净线缆透传：客户端若自带签名则无损透传；缺失签名全权委托流水线统一对齐与回填
                            let mut thought_part = json!({
                                "text": rc,
                                "thought": true,
                            });
                            if let Some(ref sig) = msg.signature {
                                thought_part["thoughtSignature"] = json!(sig);
                            }
                            parts.push(thought_part);
                        }
                    }
                } else if let Some(rc) = client_reasoning {
                    // 思考关闭时，非占位思考降级为普通文本；占位/空直接跳过
                    if !crate::proxy::thinking_store::is_placeholder_thought(rc) {
                        parts.push(json!({ "text": rc }));
                    }
                }
            }

            // Handle content (multimodal or text)
            // [FIX] Skip standard content mapping for tool/function roles to avoid duplicate parts
            // These are handled below in the "Handle tool response" section.
            let is_tool_role = msg.role == "tool" || msg.role == "function";
            let is_mid_system_role = msg.role == "system" || msg.role == "developer";
            if let (Some(content), false) = (&msg.content, is_tool_role) {
                if is_mid_system_role {
                    // [JEIKCODE SYNTHETIC USER] 中途系统消息，转换为用户态下的 <system-reminder>，不破坏全局 systemInstruction 前缀
                    let sys_text = match content {
                        OpenAIContent::String(s) => s.clone(),
                        OpenAIContent::Array(blocks) => {
                            let mut joined = String::new();
                            for b in blocks {
                                if let OpenAIContentBlock::Text { text } = b {
                                    if !joined.is_empty() {
                                        joined.push('\n');
                                    }
                                    joined.push_str(text);
                                }
                            }
                            joined
                        }
                    };
                    let wrapped_reminder = crate::proxy::mappers::common_utils::wrap_in_system_reminder(&sys_text);
                    if !wrapped_reminder.is_empty() {
                        parts.push(json!({
                            "text": wrapped_reminder
                        }));
                    }
                } else {
                    match content {
                        OpenAIContent::String(s) => {
                            if !s.is_empty() {
                                parts.extend(crate::proxy::mappers::common_utils::parse_markdown_images_to_parts(s));
                            }
                        }
                        OpenAIContent::Array(blocks) => {
                            for block in blocks {
                                match block {
                                    OpenAIContentBlock::Text { text } => {
                                        parts.extend(crate::proxy::mappers::common_utils::parse_markdown_images_to_parts(text));
                                    }
                                OpenAIContentBlock::ImageUrl { image_url } => {
                                    if image_url.url.starts_with("data:") {
                                        if let Some(pos) = image_url.url.find(",") {
                                            let mime_part = &image_url.url[5..pos];
                                            let mime_type = mime_part.split(';').next().unwrap_or("image/jpeg");
                                            let data = &image_url.url[pos + 1..];

                                            parts.push(crate::proxy::mappers::common_utils::create_gemini_inline_part(
                                                Some(mime_type),
                                                data,
                                                "Image",
                                            ));
                                        } else {
                                            parts.push(json!({"text": "[Image: invalid data URL omitted]"}));
                                        }
                                    } else if image_url.url.starts_with("http") {
                                        parts.push(json!({
                                            "fileData": { "fileUri": &image_url.url, "mimeType": "image/jpeg" }
                                        }));
                                    } else {
                                        // [NEW] 处理本地文件路径 (file:// 或 Windows/Unix 路径)
                                        let file_path = if image_url.url.starts_with("file://") {
                                            // 移除 file:// 前缀
                                            #[cfg(target_os = "windows")]
                                            { image_url.url.trim_start_matches("file:///").replace('/', "\\") }
                                            #[cfg(not(target_os = "windows"))]
                                            { image_url.url.trim_start_matches("file://").to_string() }
                                        } else {
                                            image_url.url.clone()
                                        };

                                        tracing::debug!("[OpenAI-Request] Reading local image: {}", file_path);

                                        // 读取文件并转换为 base64
                                        if let Ok(file_bytes) = std::fs::read(&file_path) {
                                            use base64::Engine as _;
                                            let b64 = base64::engine::general_purpose::STANDARD.encode(&file_bytes);

                                            // 根据文件扩展名推断 MIME 类型
                                            let mime_type = if file_path.to_lowercase().ends_with(".png") {
                                                "image/png"
                                            } else if file_path.to_lowercase().ends_with(".gif") {
                                                "image/gif"
                                            } else if file_path.to_lowercase().ends_with(".webp") {
                                                "image/webp"
                                            } else {
                                                "image/jpeg"
                                            };

                                            parts.push(crate::proxy::mappers::common_utils::create_gemini_inline_part(
                                                Some(mime_type),
                                                &b64,
                                                "Image",
                                            ));
                                            tracing::debug!("[OpenAI-Request] Successfully loaded image: {} ({} bytes)", file_path, file_bytes.len());
                                        } else {
                                            tracing::debug!("[OpenAI-Request] Failed to read local image: {}", file_path);
                                        }
                                    }
                                }
                                OpenAIContentBlock::AudioUrl { audio_url } => {
                                    // [NEW] audio_url -> Gemini inlineData / fileData
                                    match crate::proxy::audio::audio_part_from_source(
                                        &audio_url.url,
                                        audio_url.mime_type.as_deref(),
                                    ) {
                                        Some(part) => {
                                            tracing::debug!("[OpenAI-Request] Mapped audio_url to Gemini part");
                                            parts.push(part);
                                        }
                                        None => {
                                            tracing::warn!("[OpenAI-Request] Dropped unreadable audio_url part");
                                        }
                                    }
                                }
                                OpenAIContentBlock::InputAudio { input_audio } => {
                                    // [NEW] OpenAI 官方 input_audio (base64 + format) -> Gemini inlineData
                                    let mime = input_audio.mime_type();
                                    match crate::proxy::audio::audio_part_from_source(
                                        &input_audio.data,
                                        Some(&mime),
                                    ) {
                                        Some(part) => {
                                            tracing::debug!("[OpenAI-Request] Mapped input_audio ({}) to Gemini part", mime);
                                            parts.push(part);
                                        }
                                        None => {
                                            tracing::warn!("[OpenAI-Request] Dropped empty input_audio part");
                                        }
                                    }
                                }
                                OpenAIContentBlock::VideoUrl { video_url } => {
                                    // [NEW #3381] video_url -> Gemini inlineData / fileData
                                    match crate::proxy::video::video_part_from_source(
                                        &video_url.url,
                                        video_url.mime_type.as_deref(),
                                    ) {
                                        Some(part) => {
                                            tracing::debug!("[OpenAI-Request] Mapped video_url to Gemini part");
                                            parts.push(part);
                                        }
                                        None => {
                                            tracing::warn!("[OpenAI-Request] Dropped unreadable video_url part: {}", video_url.url);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            }

            // Handle tool calls (assistant message)
            if let Some(tool_calls) = &msg.tool_calls {
                for (_index, tc) in tool_calls.iter().enumerate() {
                    /* 暂时移除：防止 Codex CLI 界面碎片化
                    if index == 0 && parts.is_empty() {
                         if mapped_model.contains("gemini-3") {
                              parts.push(json!({"text": "Thinking Process: Determining necessary tool actions."}));
                         }
                    }
                    */

                    let mut args_str = String::new();
                    let mut func_name = String::new();

                    if let Some(func) = &tc.function {
                        args_str = func.arguments.clone();
                        func_name = func.name.clone();
                    } else if let Some(op) = &tc.operation {
                        func_name = "apply_patch".to_string();
                        args_str = serde_json::to_string(op).unwrap_or_else(|_| "{}".to_string());
                    } else {
                        continue;
                    }

                    let mut args = serde_json::from_str::<Value>(&args_str).unwrap_or(json!({}));

                    // [New] 利用通用引擎修正参数类型 (替代以前硬编码的 shell 工具修复逻辑)
                    if let Some(original_schema) = tool_name_to_schema.get(&func_name) {
                        crate::proxy::common::json_schema::fix_tool_call_args(&mut args, original_schema);
                    }

                    let mut func_call_part = json!({
                        "functionCall": {
                            "name": if func_name == "local_shell_call" { "shell" } else { func_name.as_str() },
                            "args": args,
                            "id": &tc.id,
                        }
                    });

                    // 纯净线缆透传：客户端若自带签名则原样透传，未带则留空，全权委托进站流水线统一对齐与回填
                    if let Some(ref sig) = tc.signature {
                        func_call_part["thoughtSignature"] = json!(sig);
                    }

                    parts.push(func_call_part);
                }
            }

            // Handle tool response
            if msg.role == "tool" || msg.role == "function" {
                let name = msg.name.as_deref().unwrap_or("unknown");
                // 优先从紧邻的前置 assistant 消息中查找匹配该 tool_call_id 的工具名称 (精准杜绝长会话 ID 碰撞时全局 Map 覆盖错误)
                let matched_preceding_name = if let Some(ref target_id) = msg.tool_call_id {
                    let mut found = None;
                    for prev_idx in (0..msg_index).rev() {
                        if let Some(prev_msg) = request.messages.get(prev_idx) {
                            if prev_msg.role == "assistant" {
                                if let Some(ref calls) = prev_msg.tool_calls {
                                    for call in calls {
                                        if call.id == *target_id {
                                            found = if let Some(ref func) = call.function {
                                                Some(if func.name == "local_shell_call" { "shell".to_string() } else { func.name.clone() })
                                            } else if call.operation.is_some() || call.r#type == "apply_patch_call" {
                                                Some("apply_patch".to_string())
                                            } else {
                                                None
                                            };
                                            break;
                                        }
                                    }
                                }
                                break;
                            } else if prev_msg.role != "tool" && prev_msg.role != "function" {
                                break;
                            }
                        }
                    }
                    found
                } else {
                    None
                };

                let final_name = if let Some(ref p_name) = matched_preceding_name {
                    p_name.as_str()
                } else if name == "local_shell_call" {
                    "shell"
                } else if let Some(id) = &msg.tool_call_id {
                    tool_id_to_name.get(id).map(|s| s.as_str()).unwrap_or(name)
                } else {
                    name
                };

                let mut extra_parts = Vec::new();

                let content_val = match &msg.content {
                    Some(OpenAIContent::String(s)) => {
                        crate::proxy::mappers::common_utils::extract_multimodal_from_tool_text(
                            s,
                            &mut extra_parts,
                        )
                    }
                    Some(OpenAIContent::Array(blocks)) => {
                        let mut texts = Vec::new();
                        for block in blocks {
                            match block {
                                OpenAIContentBlock::Text { text } => texts.push(text.clone()),
                                OpenAIContentBlock::ImageUrl { image_url } => {
                                    if image_url.url.starts_with("data:") {
                                        if let Some(pos) = image_url.url.find(',') {
                                            let mime_part = &image_url.url[5..pos];
                                            let mime_type = mime_part.split(';').next().unwrap_or("image/jpeg");
                                            let data = &image_url.url[pos + 1..];

                                            extra_parts.push(crate::proxy::mappers::common_utils::create_gemini_inline_part(
                                                Some(mime_type),
                                                data,
                                                "Tool Result Image",
                                            ));
                                        }
                                    } else {
                                        texts.push("[image link]".to_string());
                                    }
                                }
                                OpenAIContentBlock::AudioUrl { audio_url } => {
                                    match crate::proxy::audio::audio_part_from_source(
                                        &audio_url.url,
                                        audio_url.mime_type.as_deref(),
                                    ) {
                                        Some(part) => extra_parts.push(part),
                                        None => texts.push("[audio]".to_string()),
                                    }
                                }
                                OpenAIContentBlock::InputAudio { input_audio } => {
                                    let mime = input_audio.mime_type();
                                    match crate::proxy::audio::audio_part_from_source(
                                        &input_audio.data,
                                        Some(&mime),
                                    ) {
                                        Some(part) => extra_parts.push(part),
                                        None => texts.push("[audio]".to_string()),
                                    }
                                }
                                OpenAIContentBlock::VideoUrl { video_url } => {
                                    match crate::proxy::video::video_part_from_source(
                                        &video_url.url,
                                        video_url.mime_type.as_deref(),
                                    ) {
                                        Some(part) => extra_parts.push(part),
                                        None => texts.push("[video]".to_string()),
                                    }
                                }
                            }
                        }
                        texts.join("\n")
                    },
                    None => "".to_string()
                };

                // [优化] 如果结果为空，注入显式确认信号，防止模型幻觉与 Gemini 400 校验异常
                let final_content = if content_val.trim().is_empty() {
                    "Command executed successfully.".to_string()
                } else {
                    content_val
                };

                let mut fr_part = json!({
                    "functionResponse": {
                       "name": final_name,
                       "response": { "result": final_content },
                       "id": msg.tool_call_id.clone().unwrap_or_default()
                    }
                });
                // 危险测试分支法则：tool 响应 (functionResponse) 绝不携带签名
                if let Some(obj) = fr_part.as_object_mut() {
                    obj.remove("thoughtSignature");
                    obj.remove("thought_signature");
                }
                parts.push(fr_part);

                for extra in extra_parts {
                    parts.push(extra);
                }
            }

            // Ensure user role message is not dropped if parts is empty, preserving role rotation
            if role == "user" && parts.is_empty() {
                parts.push(json!({ "text": " " }));
            }

            json!({ "role": role, "parts": parts })
        })
        .filter(|msg| !msg["parts"].as_array().map(|a| a.is_empty()).unwrap_or(true))
        .collect();

    // 连续相同角色的消息**保持独立**（对齐官方形态）。
    //
    // 历史实现会合并它们，理由是 "Gemini 强制要求 user/model 交替"。但官方
    // Antigravity 报文里连续 user 轮与连续 model 轮都是常态，v1internal 上游
    // 并不要求严格交替；实测（2026-09-26，`gemini-3.8-flash-tiered` @ daily）
    // 两种形态均 200 且上下文理解一致。
    let mut merged_contents = contents;
    let protocol = if is_responses_api {
        crate::proxy::pipeline::ProxyProtocol::OpenAIResponses
    } else {
        crate::proxy::pipeline::ProxyProtocol::OpenAIChat
    };
    crate::proxy::pipeline::InboundThinkingPipeline::process_contents(
        &mut merged_contents,
        protocol,
        mapped_model,
        actual_include_thinking,
        Some(&thinking_store_key),
        false,
    );
    let mut contents = merged_contents;

    // Gemini requires conversations to start with a user turn, and functionCall turns
    // must immediately follow a user turn or a functionResponse turn.
    // If the conversation starts with a model turn (e.g. autonomous agent loops starting with tool calls),
    // inject a lightweight user primer to prevent 400 INVALID_ARGUMENT error.
    if contents.is_empty() {
        contents.push(json!({
            "role": "user",
            "parts": [{ "text": "Continue" }]
        }));
    } else if contents
        .first()
        .and_then(|f| f.get("role"))
        .and_then(|r| r.as_str())
        == Some("model")
    {
        contents.insert(
            0,
            json!({
                "role": "user",
                "parts": [{ "text": "Continue the task." }]
            }),
        );
    }
}
