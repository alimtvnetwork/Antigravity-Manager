// Response unwrapping (split from wrapper.rs).
use bytes::Bytes;
use serde_json::{json, Value};

/// Unpack the response (extract the response field)
pub fn unwrap_response(response: &Value) -> Value {
    response.get("response").unwrap_or(response).clone()
}

/// [NEW v3.3.18] Automatically inject Tool ID for Gemini response of Claude model
///
/// The purpose is to allow clients (such as OpenCode/Vercel AI SDK) to perceive the ID.
/// And bring it back as it is in the next round of dialogue, thereby satisfying Google v1internal’s verification of Claude’s model.
pub fn inject_ids_to_response(response: &mut Value, model_name: &str) {
    if !model_name.to_lowercase().contains("claude") {
        return;
    }

    if let Some(candidates) = response
        .get_mut("candidates")
        .and_then(|c| c.as_array_mut())
    {
        for candidate in candidates {
            if let Some(parts) = candidate
                .get_mut("content")
                .and_then(|c| c.get_mut("parts"))
                .and_then(|p| p.as_array_mut())
            {
                let mut name_counters: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                for part in parts {
                    if let Some(fc) = part.get_mut("functionCall").and_then(|f| f.as_object_mut()) {
                        if fc.get("id").is_none() {
                            let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                            let count = name_counters.entry(name.to_string()).or_insert(0);
                            let call_id = format!("call_{}_{}", name, count);
                            *count += 1;

                            fc.insert("id".to_string(), json!(call_id));
                            tracing::debug!("[Gemini-Wrap] Response stage: Injected synthetic call_id '{}' for client", call_id);
                        }
                    }
                }
            }
        }
    }
}
