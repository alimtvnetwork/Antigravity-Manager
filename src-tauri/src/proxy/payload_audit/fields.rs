use super::*;

/// **Display order** of streamlined text fields: Sort by user/developer attention from high to low.
/// Unlisted fields are appended in their original relative order (depending on `preserve_order` of `serde_json`).
/// Goal: Model → Thinking Block → Context → Usage/Cache → Tool Call → Rest, the four protocols behave consistently.
const CORE_FIELD_ORDER: [&str; 22] = [
    // 1. Model identification
    "model",
    // 2. Think blocks: Switch/Intensity/Content/Signature
    "thinking",
    "reasoning_effort",
    "reasoning",
    "reasoning_content",
    "generationConfig",
    "generation_config",
    "thinking_signature",
    "thought_signature",
    "signature",
    "thoughtSignature",
    "_session_thinking_id",
    // 3. Context: system prompt words and dialogue
    "system",
    "systemInstruction",
    "instructions",
    "messages",
    "contents",
    "input",
    // 4. Usage and caching
    "usage",
    "usageMetadata",
    // 5. Tool calling (the second most concerned)
    "tool_calls",
    "tools",
];

/// Rearrange top-level fields by attention - **Only adjust the order, do not delete any fields**.
/// The four protocols are shared with debugging logs to ensure consistent display order; the `request` packaging layer is recursively rearranged and kept at the front.
pub fn reorder_payload_fields(value: &Value) -> Value {
    let Value::Object(map) = value else {
        return value.clone();
    };
    let mut ordered = Map::new();
    let mut rest: Map<String, Value> = Map::new();
    for (k, v) in map {
        if k == "request" {
            ordered.insert(k.clone(), reorder_payload_fields(v));
        } else {
            rest.insert(k.clone(), v.clone());
        }
    }
    for key in CORE_FIELD_ORDER {
        if let Some(v) = rest.remove(key) {
            ordered.insert(key.to_string(), v);
        }
    }
    for (k, v) in rest {
        ordered.insert(k, v);
    }
    Value::Object(ordered)
}
