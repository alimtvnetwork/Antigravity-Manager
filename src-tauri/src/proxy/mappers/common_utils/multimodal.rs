// Multimodal content helpers (split from common_utils.rs).
// Common utilities for request mapping across all protocols
// Provides unified grounding/networking logic

use serde_json::{json, Value};

/// Request configuration after grounding resolution

/// [FIX] Parse markdown base64 images from text and split into Gemini parts
pub fn parse_markdown_images_to_parts(text: &str) -> Vec<Value> {
    let mut parts = Vec::new();
    // Match ![...](data:image/...;base64,...)
    if let Ok(re) = regex::Regex::new(r"!\[.*?\]\(data:(image/[^;]+);base64,([a-zA-Z0-9+/=]+)\)") {
        let mut last_match = 0;

        for cap in re.captures_iter(text) {
            let m = cap.get(0).unwrap();

            // Add preceding text
            if m.start() > last_match {
                let preceding = &text[last_match..m.start()];
                if !preceding.trim().is_empty() {
                    parts.push(json!({"text": preceding}));
                }
            }

            // Add inlineData image
            let mime = cap.get(1).unwrap().as_str();
            let b64 = cap.get(2).unwrap().as_str();
            let part = create_gemini_inline_part(Some(mime), b64, "Markdown Image");
            parts.push(part);

            last_match = m.end();
        }

        // Add remaining text
        if last_match < text.len() {
            let remaining = &text[last_match..];
            if !remaining.trim().is_empty() {
                parts.push(json!({"text": remaining}));
            }
        }

        if parts.is_empty() && !text.trim().is_empty() {
            parts.push(json!({"text": text}));
        }

        return parts;
    }

    if !text.trim().is_empty() {
        parts.push(json!({"text": text}));
    }

    parts
}

/// 严格受支持的常见图片 MIME 白名单（Gemini 官方原生兼容），
/// 坚决排除非图片格式（音频、视频、PDF、文档、二进制文件等），避免上游报 400 不兼容。
pub const SUPPORTED_TOOL_IMAGE_MIMES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/webp",
    "image/gif",
];

#[inline]
pub fn is_supported_tool_image_mime(mime: &str) -> bool {
    let lower = mime.trim().to_ascii_lowercase();
    SUPPORTED_TOOL_IMAGE_MIMES.contains(&lower.as_str())
}

/// 智能解析并提取工具输出中的多模态图像数据（全协议共享：OpenAI / Claude / Gemini / Responses）。
/// 支持：
/// 1. Markdown 格式图片：`![alt](data:image/...;base64,...)`
/// 2. 文本中内嵌或直接传递的 Data URL：`data:image/...;base64,...`
/// 3. JSON 格式工具输出中的图片字段：`{"image": "data:image/...", ...}` 或 `{"screenshot": "...", ...}`
///
/// 安全约束：
/// - 仅严格识别并放行常见白名单图片格式（png, jpeg, webp, gif）；
/// - 绝对不处理音频、视频、PDF、文本或二进制文件，保证非图片数据原样透传，杜绝破坏兼容性；
/// - 自动将提取出的 Base64 图像转化为规范的 Gemini `inlineData` part，追加到 `extra_parts` 中；
/// - 将原工具响应字符串中冗长庞大的 Base64 替换为精炼的摘要标记（如 `[Image: forwarded to visual input (image/png)]`），
///   既避免了 `functionResponse` JSON 负载膨胀，又让底层视觉模型能够原汁原味地进行视觉感知。
pub fn extract_multimodal_from_tool_text(raw_text: &str, extra_parts: &mut Vec<Value>) -> String {
    if !raw_text.contains("data:image/") {
        return raw_text.to_string();
    }

    // 1. 如果 raw_text 是 JSON 字符串，尝试解析并提取其中的图片字段
    if let Ok(mut val) = serde_json::from_str::<Value>(raw_text) {
        let mut extracted_any = false;
        if let Some(obj) = val.as_object_mut() {
            let candidate_keys = ["image", "screenshot", "data", "image_url", "picture"];
            for key in candidate_keys {
                if let Some(v) = obj.get_mut(key) {
                    if let Some(s) = v.as_str() {
                        if s.starts_with("data:image/") {
                            if let Some(pos) = s.find(',') {
                                let mime_part = &s[5..pos];
                                let mime_type = mime_part.split(';').next().unwrap_or("image/png");
                                // 严格限制：只允许白名单中的常见图片格式，排除任何音频、文档或非标图片
                                if is_supported_tool_image_mime(mime_type) {
                                    let b64_data = &s[pos + 1..];
                                    if let Some((valid_mime, valid_b64)) =
                                        validate_and_sanitize_inline_data(Some(mime_type), b64_data)
                                    {
                                        extra_parts.push(create_gemini_inline_part(
                                            Some(&valid_mime),
                                            &valid_b64,
                                            "Tool Result Image",
                                        ));
                                        *v = json!(format!(
                                            "[Image: forwarded to visual input ({})]",
                                            valid_mime
                                        ));
                                        extracted_any = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if extracted_any {
            return val.to_string();
        }
    }

    // 2. 检测 Markdown 图片格式：![alt](data:image/...) 或文本内嵌的 data:image/
    let mut clean_text = String::new();
    let mut rest = raw_text;
    let mut found_image = false;

    while let Some(start_idx) = rest.find("data:image/") {
        clean_text.push_str(&rest[..start_idx]);
        let data_slice = &rest[start_idx..];

        if let Some(comma_idx) = data_slice.find(',') {
            let mime_part = &data_slice[5..comma_idx];
            let mime_type = mime_part.split(';').next().unwrap_or("image/png");

            // 严格白名单校验：非白名单图片（如 svg/tiff/未知）或伪装格式不予解构，直接作为普通文本保留
            if !is_supported_tool_image_mime(mime_type) {
                clean_text.push_str("data:image/");
                rest = &data_slice["data:image/".len()..];
                continue;
            }

            let b64_start = comma_idx + 1;
            let b64_end = data_slice[b64_start..]
                .find(|c: char| c.is_whitespace() || c == ')' || c == '"' || c == '\'' || c == '`')
                .map(|idx| b64_start + idx)
                .unwrap_or(data_slice.len());

            let b64_data = &data_slice[b64_start..b64_end];
            if let Some((valid_mime, valid_b64)) =
                validate_and_sanitize_inline_data(Some(mime_type), b64_data)
            {
                extra_parts.push(create_gemini_inline_part(
                    Some(&valid_mime),
                    &valid_b64,
                    "Tool Result Image",
                ));
                clean_text.push_str(&format!(
                    "[Image: forwarded to visual input ({})]",
                    valid_mime
                ));
                found_image = true;
            } else {
                clean_text.push_str(&data_slice[..b64_end]);
            }
            rest = &data_slice[b64_end..];
        } else {
            clean_text.push_str("data:image/");
            rest = &data_slice["data:image/".len()..];
        }
    }
    clean_text.push_str(rest);

    if found_image {
        clean_text
    } else {
        raw_text.to_string()
    }
}

/// [FIX] Inject explicit tool mapping instructions for Gemini to read SKILL.md
pub fn enhance_gemini_skills_prompt(text: &str) -> String {
    let mut enhanced = text.to_string();
    let warning_note = "\n\n**[CRITICAL INSTRUCTION FOR GEMINI - HOW TO READ SKILL.md]**\nYou do NOT have a direct `view_file` or `read_file` tool.\nTo \"open and read its SKILL.md completely\" as instructed above, you MUST use the `shell_command` tool.\nFor example, run the following command in PowerShell:\n`Get-Content -Raw -Path \"C:\\Users\\...\\SKILL.md\"`\nDo NOT guess other non-existent reading tools. You must use `shell_command`!\n\n";

    // Inject before </skills_instructions> or </skills>
    if enhanced.contains("</skills_instructions>") {
        enhanced = enhanced.replace(
            "</skills_instructions>",
            &format!("{}</skills_instructions>", warning_note),
        );
    } else if enhanced.contains("</skills>") {
        enhanced = enhanced.replace("</skills>", &format!("{}</skills>", warning_note));
    }

    enhanced
}

/// Detect common MIME types from magic bytes
pub fn detect_mime_from_bytes(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(b"%PDF-") {
        Some("application/pdf")
    } else if bytes.len() >= 12
        && (&bytes[4..12] == b"ftypheic"
            || &bytes[4..12] == b"ftypmif1"
            || &bytes[4..12] == b"ftypheix")
    {
        Some("image/heic")
    } else {
        None
    }
}

/// Validates and sanitizes inline base64 data (images/documents) for Gemini upstream.
/// Returns `Some((mime_type, sanitized_b64))` if valid, or `None` if corrupt/empty/too small.
pub fn validate_and_sanitize_inline_data(
    mime_type: Option<&str>,
    b64_data: &str,
) -> Option<(String, String)> {
    let clean_b64 = b64_data.trim();
    if clean_b64.is_empty() {
        return None;
    }

    use base64::{engine::general_purpose::STANDARD, Engine as _};

    // Try decoding base64 to check validity and magic bytes
    let decoded_bytes = match STANDARD.decode(clean_b64) {
        Ok(bytes) => bytes,
        Err(_) => {
            use base64::engine::general_purpose::URL_SAFE;
            match URL_SAFE.decode(clean_b64) {
                Ok(bytes) => bytes,
                Err(_) => return None,
            }
        }
    };

    if decoded_bytes.is_empty() {
        return None;
    }

    let declared_mime = mime_type.map(str::trim).filter(|m| !m.is_empty());

    let is_audio_or_video = declared_mime
        .map(|m| m.starts_with("video/") || m.starts_with("audio/"))
        .unwrap_or(false);

    if !is_audio_or_video {
        // For images/documents, require at least 5 decoded bytes and 8 base64 chars
        if clean_b64.len() < 8 || decoded_bytes.len() < 5 {
            return None;
        }
    }

    // Detect MIME from magic bytes if possible
    let inferred_mime = detect_mime_from_bytes(&decoded_bytes);

    let final_mime = match (mime_type.map(str::trim), inferred_mime) {
        (Some(m), _)
            if !m.is_empty()
                && (m.starts_with("image/")
                    || m.starts_with("application/")
                    || m.starts_with("audio/")
                    || m.starts_with("video/")) =>
        {
            m.to_string()
        }
        (_, Some(inferred)) => inferred.to_string(),
        (Some(m), _) if !m.is_empty() => m.to_string(),
        _ => "image/jpeg".to_string(), // fallback default
    };

    Some((final_mime, clean_b64.to_string()))
}

/// Helper to create a Gemini inlineData part or fallback text if invalid
pub fn create_gemini_inline_part(
    mime_type: Option<&str>,
    b64_data: &str,
    fallback_label: &str,
) -> Value {
    if let Some((valid_mime, valid_data)) = validate_and_sanitize_inline_data(mime_type, b64_data) {
        json!({
            "inlineData": {
                "mimeType": valid_mime,
                "data": valid_data
            }
        })
    } else {
        tracing::warn!(
            "[Image-Defense] Omitted invalid or corrupt base64 data (len: {}, mime: {:?})",
            b64_data.len(),
            mime_type
        );
        json!({
            "text": format!("[{}: invalid or corrupted data omitted]", fallback_label)
        })
    }
}

/// Sanitizes any inlineData in an entire Gemini JSON request payload in-place.
/// Replaces invalid inlineData / inline_data parts with placeholder text parts.
pub fn sanitize_gemini_payload_inline_data(body: &mut Value) -> usize {
    let mut total_sanitized = 0;

    let mut sanitize_parts = |parts: &mut Vec<Value>| {
        for part in parts.iter_mut() {
            if let Some(obj) = part.as_object_mut() {
                let inline_key = if obj.contains_key("inlineData") {
                    Some("inlineData")
                } else if obj.contains_key("inline_data") {
                    Some("inline_data")
                } else {
                    None
                };

                if let Some(key) = inline_key {
                    let inline_obj = obj.get(key).and_then(Value::as_object);
                    let mime = inline_obj
                        .and_then(|o| o.get("mimeType").or_else(|| o.get("mime_type")))
                        .and_then(Value::as_str);
                    let data = inline_obj
                        .and_then(|o| o.get("data"))
                        .and_then(Value::as_str)
                        .unwrap_or_default();

                    if let Some((valid_mime, valid_data)) =
                        validate_and_sanitize_inline_data(mime, data)
                    {
                        // Ensure mimeType and data are normalized
                        obj.insert(
                            "inlineData".to_string(),
                            json!({
                                "mimeType": valid_mime,
                                "data": valid_data
                            }),
                        );
                        if key == "inline_data" {
                            obj.remove("inline_data");
                        }
                    } else {
                        total_sanitized += 1;
                        tracing::warn!(
                            "[Payload-Defense] Sanitized invalid inlineData part (len: {}, mime: {:?}) into text placeholder",
                            data.len(),
                            mime
                        );
                        *part = json!({
                            "text": "[Image/Data: invalid or corrupted inline payload omitted]"
                        });
                    }
                }
            }
        }
    };

    if let Some(contents) = body.get_mut("contents").and_then(Value::as_array_mut) {
        for content in contents.iter_mut() {
            if let Some(parts) = content.get_mut("parts").and_then(Value::as_array_mut) {
                sanitize_parts(parts);
            }
        }
    }

    if let Some(sys) = body
        .get_mut("systemInstruction")
        .and_then(Value::as_object_mut)
    {
        if let Some(parts) = sys.get_mut("parts").and_then(Value::as_array_mut) {
            sanitize_parts(parts);
        }
    }

    total_sanitized
}
