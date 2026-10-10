use super::*;

/// Injects cache_control ephemeral trigger to first message's content block if it's the XML summary
pub(crate) fn inject_cache_control_to_forked_summary(body: &mut serde_json::Value) {
    if let Some(messages) = body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        if !messages.is_empty() {
            let first_msg = &mut messages[0];
            if let Some(content) = first_msg.get_mut("content") {
                if let Some(content_arr) = content.as_array_mut() {
                    if !content_arr.is_empty() {
                        let is_summary = content_arr[0]
                            .get("text")
                            .and_then(|t| t.as_str())
                            .map(|s| s.contains("Context has been compressed"))
                            .unwrap_or(false);

                        if is_summary {
                            if let Some(obj) = content_arr[0].as_object_mut() {
                                obj.insert(
                                    "cache_control".to_string(),
                                    serde_json::json!({
                                        "type": "ephemeral"
                                    }),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
