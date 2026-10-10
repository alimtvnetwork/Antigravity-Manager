use super::*;

fn truncate_chars(s: &str, max: usize) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max {
            out.push('…');
            return out;
        }
        out.push(ch);
    }
    out
}

fn simplify_part(part: &Value) -> Value {
    let mut obj = Map::new();
    if let Some(thought) = part.get("thought") {
        obj.insert("thought".into(), thought.clone());
    }
    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
        obj.insert("text".into(), json!(text));
    }
    for sig_key in [
        "thoughtSignature",
        "thought_signature",
        "signature",
        "thinking_signature",
    ] {
        if let Some(sig) = part.get(sig_key) {
            obj.insert(sig_key.to_string(), sig.clone());
        }
    }
    if let Some(fc) = part.get("functionCall") {
        let mut fc_out = Map::new();
        if let Some(name) = fc.get("name") {
            fc_out.insert("name".into(), name.clone());
        }
        if let Some(id) = fc.get("id") {
            fc_out.insert("id".into(), id.clone());
        }
        if let Some(args) = fc.get("args") {
            fc_out.insert("args".into(), args.clone());
        }
        obj.insert("functionCall".into(), Value::Object(fc_out));
    }
    if let Some(fr) = part.get("functionResponse") {
        let mut fr_out = Map::new();
        if let Some(name) = fr.get("name") {
            fr_out.insert("name".into(), name.clone());
        }
        if let Some(id) = fr.get("id") {
            fr_out.insert("id".into(), id.clone());
        }
        if let Some(resp) = fr.get("response") {
            fr_out.insert("response".into(), resp.clone());
        }
        obj.insert("functionResponse".into(), Value::Object(fr_out));
    }
    if let Some(inline) = part.get("inlineData").or_else(|| part.get("inline_data")) {
        let mime = inline
            .get("mimeType")
            .or_else(|| inline.get("mime_type"))
            .cloned()
            .unwrap_or(json!("unknown"));
        let data_len = inline
            .get("data")
            .and_then(|d| d.as_str())
            .map(|s| s.len())
            .unwrap_or(0);
        obj.insert(
            "inlineData".into(),
            json!({
                "mimeType": mime,
                "data": format!("[base64 image: {} bytes]", data_len),
            }),
        );
    }
    if obj.is_empty() {
        return part.clone();
    }
    Value::Object(obj)
}

pub fn simplify_message(msg: &Value) -> Value {
    let mut out = Map::new();
    if let Some(role) = msg.get("role") {
        out.insert("role".into(), role.clone());
    }
    if let Some(name) = msg.get("name") {
        out.insert("name".into(), name.clone());
    }
    if let Some(tool_call_id) = msg.get("tool_call_id") {
        out.insert("tool_call_id".into(), tool_call_id.clone());
    }
    if let Some(parts) = msg.get("parts").and_then(|p| p.as_array()) {
        out.insert(
            "parts".into(),
            Value::Array(parts.iter().map(simplify_part).collect()),
        );
    }
    if let Some(content) = msg.get("content") {
        out.insert("content".into(), simplify_content(content));
    }
    if let Some(thinking) = msg.get("thinking") {
        out.insert("thinking".into(), thinking.clone());
    }
    if let Some(rc) = msg.get("reasoning_content") {
        out.insert("reasoning_content".into(), rc.clone());
    }
    for sig_key in [
        "thinking_signature",
        "thought_signature",
        "signature",
        "thoughtSignature",
    ] {
        if let Some(sig) = msg.get(sig_key) {
            out.insert(sig_key.to_string(), sig.clone());
        }
    }
    if let Some(tool_calls) = msg.get("tool_calls").and_then(|t| t.as_array()) {
        out.insert(
            "tool_calls".into(),
            Value::Array(
                tool_calls
                    .iter()
                    .map(|tc| {
                        if let Some(tc_obj) = tc.as_object() {
                            let mut tc_out = tc_obj.clone();
                            if let Some(func) = tc_obj.get("function") {
                                tc_out.insert("function".into(), func.clone());
                            }
                            Value::Object(tc_out)
                        } else {
                            tc.clone()
                        }
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(out)
}

/// OpenAI Responses 的 `input` 项结构与 chat message 不同（`type` / `call_id` / `arguments` /
/// `output` 等），单独裁剪：沿用 `simplify_message` 的体积控制，再补回调试必需的字段。
fn simplify_responses_input_item(item: &Value) -> Value {
    let mut out = simplify_message(item);
    if let Some(obj) = out.as_object_mut() {
        for key in ["type", "role", "call_id", "arguments", "output"] {
            if let Some(v) = item.get(key) {
                obj.insert(key.to_string(), v.clone());
            }
        }
    }
    out
}

fn simplify_content(content: &Value) -> Value {
    match content {
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => Value::Array(
            arr.iter()
                .map(|block| {
                    if let Some(obj) = block.as_object() {
                        let mut slim = Map::new();
                        if let Some(t) = obj.get("type") {
                            slim.insert("type".into(), t.clone());
                        }
                        if let Some(text) = obj.get("text") {
                            slim.insert("text".into(), text.clone());
                        }
                        if let Some(thinking) = obj.get("thinking") {
                            slim.insert("thinking".into(), thinking.clone());
                        }
                        for sig_key in [
                            "signature",
                            "thoughtSignature",
                            "thinking_signature",
                            "thought_signature",
                        ] {
                            if let Some(sig) = obj.get(sig_key) {
                                slim.insert(sig_key.to_string(), sig.clone());
                            }
                        }
                        if let Some(id) = obj.get("id") {
                            slim.insert("id".into(), id.clone());
                        }
                        if let Some(name) = obj.get("name") {
                            slim.insert("name".into(), name.clone());
                        }
                        if let Some(input) = obj.get("input") {
                            slim.insert("input".into(), input.clone());
                        }
                        if let Some(tool_use_id) = obj.get("tool_use_id") {
                            slim.insert("tool_use_id".into(), tool_use_id.clone());
                        }
                        if let Some(is_error) = obj.get("is_error") {
                            slim.insert("is_error".into(), is_error.clone());
                        }
                        if let Some(source) = obj.get("source") {
                            if source.get("type").and_then(|t| t.as_str()) == Some("base64") {
                                let media_type = source
                                    .get("media_type")
                                    .cloned()
                                    .unwrap_or(json!("unknown"));
                                let len = source
                                    .get("data")
                                    .and_then(|d| d.as_str())
                                    .map(|s| s.len())
                                    .unwrap_or(0);
                                slim.insert(
                                    "source".into(),
                                    json!({
                                        "type": "base64",
                                        "media_type": media_type,
                                        "data": format!("[base64 image: {} bytes]", len)
                                    }),
                                );
                            } else {
                                slim.insert("source".into(), source.clone());
                            }
                        }
                        if let Some(img_url) = obj.get("image_url") {
                            if let Some(url_str) = img_url.get("url").and_then(|u| u.as_str()) {
                                if url_str.starts_with("data:") {
                                    let comma_pos = url_str.find(',').unwrap_or(0);
                                    let header = &url_str[..comma_pos];
                                    let len = url_str.len() - comma_pos;
                                    slim.insert("image_url".into(), json!({
                                        "url": format!("{},[base64 image: {} bytes]", header, len)
                                    }));
                                } else {
                                    slim.insert("image_url".into(), img_url.clone());
                                }
                            } else {
                                slim.insert("image_url".into(), img_url.clone());
                            }
                        }
                        if slim.is_empty() {
                            Value::Object(obj.clone())
                        } else {
                            Value::Object(slim)
                        }
                    } else {
                        block.clone()
                    }
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Fully preserves tools Schema (functionDeclarations, parameters, googleSearch, etc.) for debugging
fn simplify_tools(tools: &Value) -> Value {
    tools.clone()
}
