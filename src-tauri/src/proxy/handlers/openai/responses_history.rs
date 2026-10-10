// Responses history helpers: inline-media stripping, prefill rewrites, session saves.
use serde_json::{json, Value};
use tracing::debug;

fn historical_media_placeholder(value: &serde_json::Map<String, Value>) -> Option<Value> {
    match value.get("type").and_then(Value::as_str) {
        Some("input_image") | Some("image_url") => Some(json!({
            "type": "input_text",
            "text": "[historical image omitted]"
        })),
        Some("input_audio") | Some("audio") | Some("audio_url") => Some(json!({
            "type": "input_text",
            "text": "[historical audio omitted]"
        })),
        _ => None,
    }
}

pub(crate) fn history_without_inline_media(value: &Value) -> Value {
    fn clone_bounded(value: &Value) -> Option<Value> {
        match value {
            Value::String(text)
                if text.starts_with("data:image/") || text.starts_with("data:audio/") =>
            {
                None
            }
            Value::Array(values) => Some(Value::Array(
                values.iter().filter_map(clone_bounded).collect(),
            )),
            Value::Object(values) => historical_media_placeholder(values).or_else(|| {
                Some(Value::Object(
                    values
                        .iter()
                        .filter_map(|(key, value)| {
                            clone_bounded(value).map(|value| (key.clone(), value))
                        })
                        .collect(),
                ))
            }),
            _ => Some(value.clone()),
        }
    }

    clone_bounded(value).unwrap_or(Value::Null)
}

pub(crate) fn into_history_without_inline_media(value: Value) -> Option<Value> {
    match value {
        Value::String(text)
            if text.starts_with("data:image/") || text.starts_with("data:audio/") =>
        {
            None
        }
        Value::Array(values) => Some(Value::Array(
            values
                .into_iter()
                .filter_map(into_history_without_inline_media)
                .collect(),
        )),
        Value::Object(values) => {
            if let Some(placeholder) = historical_media_placeholder(&values) {
                Some(placeholder)
            } else {
                Some(Value::Object(
                    values
                        .into_iter()
                        .filter_map(|(key, value)| {
                            into_history_without_inline_media(value).map(|value| (key, value))
                        })
                        .collect(),
                ))
            }
        }
        other => Some(other),
    }
}

pub(crate) fn omit_media_before_latest_user_turn(items: &mut [Value]) {
    let Some(current_turn_start) = items.iter().rposition(|item| {
        item.get("role").and_then(Value::as_str) == Some("user")
            && matches!(
                item.get("type").and_then(Value::as_str),
                None | Some("message")
            )
    }) else {
        return;
    };

    for item in &mut items[..current_turn_start] {
        let historical = std::mem::take(item);
        *item = into_history_without_inline_media(historical).unwrap_or(Value::Null);
    }
}

pub(crate) async fn save_session_unless_response_cancelled<F>(
    mut ack_tx: tokio::sync::oneshot::Sender<()>,
    save: F,
) where
    F: std::future::Future<Output = ()>,
{
    let saved = tokio::select! {
        biased;
        _ = ack_tx.closed() => false,
        _ = save => true,
    };
    if saved {
        // Justification: oneshot::Sender::send returns Result<(), ()> — the unit error carries no information to log; a dropped receiver is benign here.
        let _ = ack_tx.send(());
    }
}

pub(crate) fn build_responses_tool_output_content(
    text: String,
    mut media_parts: Vec<Value>,
) -> Value {
    if media_parts.is_empty() {
        return Value::String(text);
    }

    let mut content = Vec::with_capacity(media_parts.len() + usize::from(!text.is_empty()));
    if !text.is_empty() {
        content.push(json!({ "type": "text", "text": text }));
    }
    content.append(&mut media_parts);
    Value::Array(content)
}

pub(crate) fn debug_value_without_inline_data(value: &Value) -> Value {
    match value {
        Value::String(text)
            if text.starts_with("data:image/") || text.starts_with("data:audio/") =>
        {
            Value::String(format!("[inline data omitted: {} chars]", text.len()))
        }
        Value::Array(values) => {
            Value::Array(values.iter().map(debug_value_without_inline_data).collect())
        }
        Value::Object(values) => {
            let is_inline_data = values.get("mimeType").and_then(Value::as_str).is_some()
                && values.get("data").and_then(Value::as_str).is_some();
            Value::Object(
                values
                    .iter()
                    .map(|(key, value)| {
                        let value = if is_inline_data && key == "data" {
                            Value::String(format!(
                                "[inline data omitted: {} chars]",
                                value.as_str().map(str::len).unwrap_or_default()
                            ))
                        } else {
                            debug_value_without_inline_data(value)
                        };
                        (key.clone(), value)
                    })
                    .collect(),
            )
        }
        _ => value.clone(),
    }
}

#[derive(Default)]
struct JsonByteCounter(usize);

impl std::io::Write for JsonByteCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 += bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn serialized_json_len(value: &Value) -> usize {
    let mut counter = JsonByteCounter::default();
    serde_json::to_writer(&mut counter, value)
        .map(|_| counter.0)
        .unwrap_or_default()
}

pub(crate) fn rewrite_terminal_assistant_prefill(messages: &mut [Value]) -> bool {
    let Some(last_message) = messages.last_mut() else {
        return false;
    };

    if last_message.get("role").and_then(Value::as_str) != Some("assistant") {
        return false;
    }

    let has_nonempty_plain_text = last_message
        .get("content")
        .and_then(Value::as_str)
        .is_some_and(|content| !content.trim().is_empty());
    if !has_nonempty_plain_text {
        return false;
    }

    let has_tool_calls = match last_message.get("tool_calls") {
        None | Some(Value::Null) => false,
        Some(Value::Array(tool_calls)) => !tool_calls.is_empty(),
        Some(_) => true,
    };
    if has_tool_calls {
        return false;
    }

    let Some(message) = last_message.as_object_mut() else {
        return false;
    };
    message.insert("role".to_string(), Value::String("user".to_string()));
    true
}

fn is_tool_history_message(message: &Value) -> bool {
    match message.get("role").and_then(Value::as_str) {
        Some("tool" | "function") => true,
        Some("assistant") => message
            .get("tool_calls")
            .and_then(Value::as_array)
            .is_some_and(|tool_calls| !tool_calls.is_empty()),
        _ => false,
    }
}

pub(crate) fn drop_leading_orphan_tool_history(messages: &mut Vec<Value>) -> usize {
    let conversation_start = messages
        .iter()
        .position(|message| message.get("role").and_then(Value::as_str) != Some("system"))
        .unwrap_or(messages.len());
    let orphan_count = messages[conversation_start..]
        .iter()
        .take_while(|message| is_tool_history_message(message))
        .count();

    if orphan_count > 0 {
        messages.drain(conversation_start..conversation_start + orphan_count);
    }
    orphan_count
}

#[cfg(test)]
mod tests {
    use super::super::codex_convert::convert_codex_to_openai_request;
    use super::super::responses_media::validate_responses_input_image_limits;
    use super::*;
    use serde_json::{json, Value};

    fn pure_image_tool_history_keeps_small_placeholder() {
        let history = json!({
            "type": "function_call_output",
            "call_id": "call_image",
            "output": [{
                "type": "input_image",
                "image_url": "data:image/png;base64,AQ=="
            }]
        });
        let expected = json!({
            "type": "function_call_output",
            "call_id": "call_image",
            "output": [{
                "type": "input_text",
                "text": "[historical image omitted]"
            }]
        });

        let borrowed = history_without_inline_media(&history);
        let moved = into_history_without_inline_media(history).expect("bounded history");
        assert_eq!(borrowed, expected);
        assert_eq!(moved, expected);
        assert!(!borrowed.to_string().contains("base64"));
    }

    fn responses_omits_old_images_before_validating_current_turn() {
        let images = |count| {
            (0..count)
                .map(|_| json!({"type": "input_image", "image_url": "data:image/png;base64,AQ=="}))
                .collect::<Vec<_>>()
        };
        let mut input = vec![
            json!({"type": "message", "role": "user", "content": images(16)}),
            json!({"type": "message", "role": "assistant", "content": "done"}),
            json!({"type": "message", "role": "user", "content": images(16)}),
        ];

        omit_media_before_latest_user_turn(&mut input);
        assert!(input[0].to_string().contains("[historical image omitted]"));
        assert!(!input[0].to_string().contains("data:image/"));
        assert!(validate_responses_input_image_limits(Some(&Value::Array(input.clone()))).is_ok());

        input[2]["content"] = Value::Array(images(17));
        assert!(validate_responses_input_image_limits(Some(&Value::Array(input))).is_err());
    }

    async fn cancelled_response_drops_pending_session_save() {
        let response_id = format!("resp-cancelled-{}", uuid::Uuid::new_v4());
        let save_response_id = response_id.clone();
        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
        let save_task = tokio::spawn(save_session_unless_response_cancelled(ack_tx, async move {
            entered_tx.send(()).expect("save future entered");
            std::future::pending::<()>().await;
            crate::proxy::http_session_store::save_session_delta(
                save_response_id,
                None,
                vec![json!({"id": "cancelled-user"})],
                Vec::new(),
                String::new(),
                "gemini-pro-agent".to_string(),
                "routing-cancelled".to_string(),
            )
            .await;
        }));

        entered_rx.await.expect("save future is waiting");
        drop(ack_rx);
        tokio::time::timeout(std::time::Duration::from_secs(1), save_task)
            .await
            .expect("cancelled save task exits")
            .expect("save task");
        assert!(crate::proxy::http_session_store::get_session(&response_id)
            .await
            .is_none());
    }

    fn responses_compat_rewrites_only_terminal_plain_text_assistant_prefill() {
        let mut plain_text = vec![json!({
            "role": "assistant",
            "content": "Choose the next action."
        })];
        assert!(rewrite_terminal_assistant_prefill(&mut plain_text));
        assert_eq!(plain_text[0]["role"], "user");

        let unchanged_messages = [
            json!({"role": "assistant", "content": "   "}),
            json!({
                "role": "assistant",
                "content": [{
                    "type": "image_url",
                    "image_url": {"url": "data:image/png;base64,AA=="}
                }]
            }),
            json!({
                "role": "assistant",
                "content": "Call a tool.",
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {"name": "query_memory", "arguments": "{}"}
                }]
            }),
        ];
        for original in unchanged_messages {
            let mut messages = vec![original.clone()];
            assert!(!rewrite_terminal_assistant_prefill(&mut messages));
            assert_eq!(messages[0], original);
        }

        let mut non_terminal = vec![
            json!({"role": "assistant", "content": "Earlier output."}),
            json!({"role": "user", "content": "Latest input."}),
        ];
        assert!(!rewrite_terminal_assistant_prefill(&mut non_terminal));
        assert_eq!(non_terminal[0]["role"], "assistant");

        let converted = convert_codex_to_openai_request(json!({
            "input": [{
                "role": "assistant",
                "content": "Choose the next action."
            }]
        }));
        assert_eq!(
            converted["messages"],
            json!([{"role": "user", "content": "Choose the next action."}])
        );
    }

    fn responses_compat_drops_only_leading_orphan_tool_history() {
        let mut messages = vec![
            json!({"role": "system", "content": "Plan safely."}),
            json!({
                "role": "assistant",
                "content": "",
                "tool_calls": [{
                    "id": "call_orphan",
                    "type": "function",
                    "function": {"name": "reply", "arguments": "{}"}
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_orphan",
                "content": "done"
            }),
            json!({"role": "user", "content": "Latest message"}),
            json!({
                "role": "assistant",
                "content": "",
                "tool_calls": [{
                    "id": "call_valid",
                    "type": "function",
                    "function": {"name": "query_memory", "arguments": "{}"}
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_valid",
                "content": "result"
            }),
        ];

        assert_eq!(drop_leading_orphan_tool_history(&mut messages), 2);
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[2]["tool_calls"][0]["id"], "call_valid");
        assert_eq!(messages[3]["tool_call_id"], "call_valid");

        let mut ordinary_history = vec![
            json!({"role": "system", "content": "Plan safely."}),
            json!({"role": "assistant", "content": "Earlier answer"}),
            json!({"role": "user", "content": "Latest message"}),
        ];
        let original = ordinary_history.clone();
        assert_eq!(drop_leading_orphan_tool_history(&mut ordinary_history), 0);
        assert_eq!(ordinary_history, original);
    }
}
