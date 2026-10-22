//! Claude Message Alternating Role Prefix Sanitizer
//!
//! Enforces strict alternating role sequences (`user` -> `assistant` -> `user`) in Claude
//! messages payloads. Detects adjacent turns with identical roles (e.g. consecutive `user` turns)
//! and merges them or inserts clean synthetic bridging turns to ensure Anthropic schema compliance.

use serde_json::Value;
use tracing::debug;

/// Sanitizes a Claude messages JSON array to ensure strict user/assistant alternation.
pub fn sanitize_claude_role_alternation(messages: &mut Vec<Value>) -> usize {
    if messages.is_empty() {
        return 0;
    }

    let mut modifications = 0;
    let mut i = 0;

    while i + 1 < messages.len() {
        let current_role = messages[i].get("role").and_then(|r| r.as_str()).unwrap_or("user");
        let next_role = messages[i + 1].get("role").and_then(|r| r.as_str()).unwrap_or("user");

        if current_role == next_role {
            // Consecutive matching roles detected: merge content blocks if both are strings or arrays
            let current_content = messages[i].get("content").cloned();
            let next_content = messages[i + 1].get("content").cloned();

            if let (Some(Value::String(c1)), Some(Value::String(c2))) = (current_content, next_content) {
                let merged = format!("{}\n\n{}", c1, c2);
                if let Some(obj) = messages[i].as_object_mut() {
                    obj.insert("content".to_string(), Value::String(merged));
                }
                messages.remove(i + 1);
                modifications += 1;
                continue; // Re-check from current index
            } else {
                // If structured, insert synthetic bridging turn
                let bridging_role = if current_role == "user" { "assistant" } else { "user" };
                let bridging_msg = serde_json::json!({
                    "role": bridging_role,
                    "content": "..."
                });
                messages.insert(i + 1, bridging_msg);
                modifications += 1;
                i += 2;
                continue;
            }
        }

        i += 1;
    }

    if modifications > 0 {
        debug!(modifications = modifications, "Sanitized Claude message role alternation");
    }

    modifications
}
