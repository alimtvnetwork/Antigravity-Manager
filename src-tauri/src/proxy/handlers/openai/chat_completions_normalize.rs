// Normalize a Responses-format body into Chat Completions shape (prelude phase).
use serde_json::{json, Value};
use tracing::debug;

/// Detect Responses API format (`instructions`/`input` without `messages`)
/// and convert it to Chat Completions format in place.
pub(crate) fn apply_responses_format_fallback(body: &mut Value) {
    // [NEW] Automatically detect and convert Responses format
    // If the request contains instructions or input but no messages, it is considered to be in Responses format
    let is_responses_format = !body.get("messages").is_some()
        && (body.get("instructions").is_some() || body.get("input").is_some());

    if is_responses_format {
        debug!("Detected Responses API format, converting to Chat Completions format");

        // Convert instructions to system message
        if let Some(instructions) = body.get("instructions").and_then(|v| v.as_str()) {
            if !instructions.is_empty() {
                let system_msg = json!({
                    "role": "system",
                    "content": instructions
                });

                // Initialize messages array
                if !body.get("messages").is_some() {
                    body["messages"] = json!([]);
                }

                // Insert system message at the beginning
                if let Some(messages) = body.get_mut("messages").and_then(|v| v.as_array_mut()) {
                    messages.insert(0, system_msg);
                }
            }
        }

        // Convert input to user message (if present)
        if let Some(input) = body.get("input") {
            let user_msg = if input.is_string() {
                json!({
                    "role": "user",
                    "content": input.as_str().unwrap_or("")
                })
            } else {
                // input is in array format, temporarily simplifying the processing
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
