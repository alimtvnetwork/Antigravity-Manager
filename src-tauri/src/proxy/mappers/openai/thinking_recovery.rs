use serde_json::{json, Value};

/// Strip all blocks marked as thinking content (thought: true)
pub fn strip_all_thinking_blocks(contents: Vec<Value>) -> Vec<Value> {
    contents
        .into_iter()
        .map(|mut content| {
            if let Some(parts) = content.get_mut("parts").and_then(|v| v.as_array_mut()) {
                parts.retain(|part| {
                    !part
                        .get("thought")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                });
            }
            content
        })
        .filter(|msg| {
            !msg["parts"]
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(true)
        })
        .collect()
}

/// Close tool loop for thinking models
/// First strip thinking blocks, then inject synthetic Model confirmation and User continue instructions
#[allow(dead_code)]
pub fn close_tool_loop_for_thinking(contents: Vec<Value>) -> Vec<Value> {
    let mut stripped = strip_all_thinking_blocks(contents);

    // If no content left, return empty
    if stripped.is_empty() {
        return stripped;
    }

    // Synthesize model message: tool execution completed
    stripped.push(json!({
        "role": "model",
        "parts": [{"text": "[Tool execution completed.]"}]
    }));

    // Synthesize user message: prompt continue
    stripped.push(json!({
        "role": "user",
        "parts": [{"text": "[Continue]"}]
    }));

    stripped
}
