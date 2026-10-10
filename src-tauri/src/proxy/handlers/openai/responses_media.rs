// Responses-media helpers: stream-chunk inspection and content-part parsing.
use serde_json::{json, Value};

use crate::proxy::mappers::openai::{OpenAIContent, OpenAIContentBlock};

/// Return true only when a streamed chunk contains an actual error event.
///
/// Responses lifecycle envelopes legitimately contain `"error": null`, and
/// assistant text may also mention the word "error". A substring search would
/// misclassify both as failures and rotate otherwise healthy accounts.
pub(crate) fn stream_chunk_has_error_event(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(bytes).lines().any(|line| {
        let Some(data) = line.trim_start().strip_prefix("data:") else {
            return false;
        };
        let Ok(payload) = serde_json::from_str::<Value>(data.trim()) else {
            return false;
        };

        matches!(
            payload.get("type").and_then(Value::as_str),
            Some("error" | "response.failed")
        ) || payload.get("error").is_some_and(|error| !error.is_null())
    })
}

pub(crate) fn response_has_inline_image_data(value: &Value) -> bool {
    let response = value.get("response").unwrap_or(value);
    response
        .get("candidates")
        .and_then(Value::as_array)
        .is_some_and(|candidates| {
            candidates.iter().any(|candidate| {
                candidate
                    .get("content")
                    .and_then(|content| content.get("parts"))
                    .and_then(Value::as_array)
                    .is_some_and(|parts| {
                        parts.iter().any(|part| {
                            part.get("inlineData")
                                .or_else(|| part.get("inline_data"))
                                .and_then(|image| image.get("data"))
                                .and_then(Value::as_str)
                                .is_some_and(|data| !data.is_empty())
                        })
                    })
            })
        })
}

fn text_has_nonempty_image_data_url(text: &str) -> bool {
    let mut remaining = text;
    while let Some(start) = remaining.find("data:image/") {
        let candidate = &remaining[start..];
        if let Some((_, encoded)) = candidate.split_once(";base64,") {
            if encoded.chars().next().is_some_and(|first| {
                !first.is_whitespace() && !matches!(first, '"' | '\\' | ')' | ']' | '}')
            }) {
                return true;
            }
        }
        remaining = &candidate["data:image/".len()..];
    }
    false
}

fn value_has_nonempty_image_data_url(value: &Value) -> bool {
    match value {
        Value::String(text) => text_has_nonempty_image_data_url(text),
        Value::Array(values) => values.iter().any(value_has_nonempty_image_data_url),
        Value::Object(values) => values.values().any(value_has_nonempty_image_data_url),
        _ => false,
    }
}

pub(crate) fn stream_chunk_has_image_data(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(bytes).lines().any(|line| {
        let Some(data) = line.trim_start().strip_prefix("data:") else {
            return false;
        };
        serde_json::from_str::<Value>(data.trim())
            .ok()
            .is_some_and(|payload| value_has_nonempty_image_data_url(&payload))
    })
}

pub(crate) fn responses_input_item_type(item: &Value) -> &str {
    item.get("type")
        .and_then(Value::as_str)
        .or_else(|| item.get("role").and_then(Value::as_str).map(|_| "message"))
        .unwrap_or("")
}

fn push_responses_content_part(
    mut part: Value,
    text_parts: &mut Vec<String>,
    media_parts: &mut Vec<Value>,
) -> Result<(), Value> {
    let part_type = part
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if let Some(Value::String(text)) = part.as_object_mut().and_then(|obj| obj.remove("text")) {
        text_parts.push(text);
        Ok(())
    } else if part_type == "input_image" {
        if let Some(Value::String(image_url)) =
            part.as_object_mut().and_then(|obj| obj.remove("image_url"))
        {
            media_parts.push(json!({
                "type": "image_url",
                "image_url": { "url": image_url }
            }));
            Ok(())
        } else {
            Err(part)
        }
    } else if part_type == "image_url" {
        if let Some(image_url) = part.as_object_mut().and_then(|obj| obj.remove("image_url")) {
            media_parts.push(json!({
                "type": "image_url",
                "image_url": image_url
            }));
            Ok(())
        } else {
            Err(part)
        }
    } else if matches!(part_type.as_str(), "input_audio" | "audio") {
        if let Some(input_audio) = part
            .as_object_mut()
            .and_then(|obj| obj.remove("input_audio"))
        {
            media_parts.push(json!({
                "type": "input_audio",
                "input_audio": input_audio
            }));
            Ok(())
        } else {
            Err(part)
        }
    } else if part_type == "audio_url" {
        if let Some(audio_url) = part.as_object_mut().and_then(|obj| obj.remove("audio_url")) {
            media_parts.push(json!({
                "type": "audio_url",
                "audio_url": audio_url
            }));
            Ok(())
        } else {
            Err(part)
        }
    } else {
        Err(part)
    }
}

fn responses_content_parts(content: Option<Value>) -> (Vec<String>, Vec<Value>, Vec<Value>) {
    let mut text_parts = Vec::new();
    let mut media_parts = Vec::new();
    let mut unhandled_parts = Vec::new();

    match content {
        Some(Value::String(text)) => text_parts.push(text),
        Some(Value::Array(parts)) => {
            for part in parts {
                if let Err(part) =
                    push_responses_content_part(part, &mut text_parts, &mut media_parts)
                {
                    unhandled_parts.push(part);
                }
            }
        }
        Some(part @ Value::Object(_)) => {
            if let Err(part) = push_responses_content_part(part, &mut text_parts, &mut media_parts)
            {
                unhandled_parts.push(part);
            }
        }
        Some(other) => unhandled_parts.push(other),
        None => {}
    }

    (text_parts, media_parts, unhandled_parts)
}

pub(crate) fn responses_message_parts(item: &mut Value) -> (Vec<String>, Vec<Value>) {
    let content = item.as_object_mut().and_then(|obj| obj.remove("content"));
    let (text, media, _) = responses_content_parts(content);
    (text, media)
}

pub(crate) fn responses_tool_output_parts(item: &mut Value) -> (String, Vec<Value>) {
    let Some(output) = item.as_object_mut().and_then(|obj| obj.remove("output")) else {
        return (String::new(), Vec::new());
    };
    let mut output = output;
    let content = output
        .as_object_mut()
        .and_then(|obj| obj.remove("content"))
        .unwrap_or(output);

    match content {
        Value::String(text) => (text, Vec::new()),
        content @ Value::Array(_) => {
            let (text_parts, media_parts, unhandled) = responses_content_parts(Some(content));
            if text_parts.is_empty() && media_parts.is_empty() {
                (Value::Array(unhandled).to_string(), Vec::new())
            } else {
                (text_parts.join("\n"), media_parts)
            }
        }
        content @ Value::Object(_) if content.get("type").is_some() => {
            let (text_parts, media_parts, unhandled) = responses_content_parts(Some(content));
            if text_parts.is_empty() && media_parts.is_empty() {
                (
                    unhandled
                        .into_iter()
                        .next()
                        .unwrap_or(Value::Null)
                        .to_string(),
                    Vec::new(),
                )
            } else {
                (text_parts.join("\n"), media_parts)
            }
        }
        _ => (content.to_string(), Vec::new()),
    }
}

fn decoded_base64_len(encoded: &str) -> Result<usize, String> {
    let mut decoder = base64::read::DecoderReader::new(
        encoded.as_bytes(),
        &base64::engine::general_purpose::STANDARD,
    );
    let mut sink = io::sink();
    usize::try_from(
        io::copy(&mut decoder, &mut sink)
            .map_err(|_| "Input image contains invalid base64 data".to_string())?,
    )
    .map_err(|_| "Input image is too large".to_string())
}

fn validate_responses_image_data_url(
    data_url: &str,
    image_count: usize,
    total_bytes: usize,
) -> Result<usize, String> {
    let (_, encoded) = parse_image_data_url_parts(data_url)?;
    validate_input_image_limits(image_count, 0, total_bytes)?;
    let max_encoded_len = MAX_INPUT_IMAGE_BYTES.div_ceil(3).saturating_mul(4);
    if encoded.len() > max_encoded_len {
        return Err(format!(
            "Input image is too large: maximum decoded size is {} bytes",
            MAX_INPUT_IMAGE_BYTES
        ));
    }
    let decoded_len = decoded_base64_len(encoded)?;
    validate_input_image_limits(
        image_count,
        decoded_len,
        total_bytes.saturating_add(decoded_len),
    )?;
    Ok(decoded_len)
}

pub(crate) fn validate_responses_input_image_limits(input: Option<&Value>) -> Result<(), String> {
    fn visit(value: &Value, count: &mut usize, total: &mut usize) -> Result<(), String> {
        match value {
            Value::Array(values) => {
                for value in values {
                    visit(value, count, total)?;
                }
            }
            Value::Object(values) => {
                if matches!(
                    values.get("type").and_then(Value::as_str),
                    Some("input_image") | Some("image_url")
                ) {
                    let image_url = values.get("image_url").and_then(|value| {
                        value
                            .as_str()
                            .or_else(|| value.get("url").and_then(Value::as_str))
                    });
                    if let Some(data_url) = image_url {
                        if !data_url.starts_with("data:") {
                            return Ok(());
                        }
                        *count = count.saturating_add(1);
                        let decoded_len =
                            validate_responses_image_data_url(data_url, *count, *total)?;
                        *total = total.saturating_add(decoded_len);
                    }
                    return Ok(());
                }
                for value in values.values() {
                    visit(value, count, total)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    let mut count = 0;
    let mut total = 0;
    if let Some(input) = input {
        visit(input, &mut count, &mut total)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::codex_convert::convert_codex_to_openai_request;
    use super::super::image_input::{
        validate_input_image_limits, MAX_INPUT_IMAGES, MAX_INPUT_IMAGE_BYTES,
        MAX_TOTAL_INPUT_IMAGE_BYTES,
    };
    use super::*;
    use crate::proxy::mappers::openai::{transform_openai_request, OpenAIRequest};
    use serde_json::{json, Value};

    fn responses_created_with_null_error_is_not_an_error_event() {
        let chunk = br#"event: response.created
data: {"type":"response.created","response":{"status":"in_progress","error":null}}

"#;
        assert!(!stream_chunk_has_error_event(chunk));
    }

    fn response_failed_is_an_error_event() {
        let chunk = br#"event: response.failed
data: {"type":"response.failed","response":{"status":"failed","error":{"code":"upstream_error"}}}

"#;
        assert!(stream_chunk_has_error_event(chunk));
    }

    fn legacy_top_level_error_is_an_error_event() {
        let chunk = br#"data: {"error":{"message":"quota exceeded"}}

"#;
        assert!(stream_chunk_has_error_event(chunk));
    }

    fn normal_text_containing_error_is_not_an_error_event() {
        let chunk = br#"data: {"type":"response.output_text.delta","delta":"The JSON key is called \"error\"."}

"#;
        assert!(!stream_chunk_has_error_event(chunk));
    }

    fn task_image_success_requires_nonempty_payload() {
        let empty = json!({
            "response": {"candidates": [{"content": {"parts": [{"inlineData": {"data": ""}}]}}]}
        });
        let image = json!({
            "response": {"candidates": [{"content": {"parts": [{"inlineData": {"data": "AQ=="}}]}}]}
        });
        assert!(!response_has_inline_image_data(&empty));
        assert!(response_has_inline_image_data(&image));

        let empty_chunk =
            br#"data: {"choices":[{"delta":{"content":"![image](data:image/png;base64,)"}}]}

"#;
        let image_chunk =
            br#"data: {"choices":[{"delta":{"content":"![image](data:image/png;base64,AQ==)"}}]}

"#;
        assert!(!stream_chunk_has_image_data(empty_chunk));
        assert!(stream_chunk_has_image_data(image_chunk));
    }

    fn responses_compat_accepts_optional_type_and_string_or_array_content() {
        let string_message = json!({
            "role": "system",
            "content": "Follow the system instructions."
        });
        let array_message = json!({
            "role": "user",
            "content": [{"type": "input_text", "text": "Continue planning."}]
        });

        assert_eq!(responses_input_item_type(&string_message), "message");
        assert_eq!(
            responses_message_parts(&mut string_message.clone()).0,
            vec!["Follow the system instructions."]
        );
        assert_eq!(
            responses_message_parts(&mut array_message.clone()).0,
            vec!["Continue planning."]
        );

        let converted = convert_codex_to_openai_request(json!({
            "input": [
                string_message,
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "output_text", "text": "Previous output."}]
                },
                array_message
            ]
        }));

        assert_eq!(
            converted["messages"],
            json!([
                {"role": "system", "content": "Follow the system instructions."},
                {"role": "assistant", "content": "Previous output."},
                {"role": "user", "content": "Continue planning."}
            ])
        );
    }

    fn responses_data_url_metadata_and_byte_limits() {
        assert_eq!(
            validate_responses_image_data_url("data:image/png;BASE64,AQ==", 1, 0)
                .expect("uppercase base64 token"),
            1
        );
        let encoded = "AAAA".repeat(MAX_INPUT_IMAGE_BYTES / 3 + 1);
        assert_eq!(
            validate_responses_image_data_url(&format!("data:image/png;BASE64,{encoded}"), 1, 0)
                .unwrap_err(),
            format!(
                "Input image is too large: maximum decoded size is {} bytes",
                MAX_INPUT_IMAGE_BYTES
            )
        );
        assert!(validate_responses_image_data_url(
            "data:image/png;BASE64,AQ==",
            1,
            MAX_TOTAL_INPUT_IMAGE_BYTES
        )
        .unwrap_err()
        .starts_with("Total input image data is too large"));
        assert!(validate_input_image_limits(
            MAX_INPUT_IMAGES,
            2 * 1024 * 1024,
            MAX_TOTAL_INPUT_IMAGE_BYTES
        )
        .is_ok());
        assert!(validate_input_image_limits(
            MAX_INPUT_IMAGES,
            2 * 1024 * 1024,
            MAX_TOTAL_INPUT_IMAGE_BYTES + 1
        )
        .is_err());
    }
}
