//! Claude Message Alternating Role Prefix Sanitizer
//!
//! Enforces strict alternating role sequences (`user` -> `assistant` -> `user`) in Claude
//! messages payloads. Detects adjacent turns with identical roles (e.g. consecutive `user` turns)
//! and merges them or inserts clean synthetic bridging turns to ensure Anthropic schema compliance.

use serde_json::Value;
use tracing::debug;

/// Returns the opposite canonical role for synthetic bridging turns.
pub fn opposite_role(role: &str) -> &'static str {
    if role.eq_ignore_ascii_case("user") {
        "assistant"
    } else {
        "user"
    }
}

/// Checks if two role identifiers are equivalent.
pub fn is_same_role(r1: &str, r2: &str) -> bool {
    r1.eq_ignore_ascii_case(r2)
}

/// Merges two string content blocks with standard double-newline spacing.
pub fn merge_text_contents(c1: &str, c2: &str) -> String {
    format!("{}\n\n{}", c1, c2)
}

/// Strips legacy conversational role prefixes (e.g., "Human: ", "Assistant: ", "User: ") from message content.
pub fn strip_leading_role_markers(text: &str) -> &str {
    const PREFIXES: &[&str] = &[
        "Human:",
        "human:",
        "Assistant:",
        "assistant:",
        "User:",
        "user:",
        "System:",
        "system:",
    ];

    let trimmed = text.trim_start();
    for &p in PREFIXES {
        if trimmed.starts_with(p) {
            return trimmed[p.len()..].trim_start();
        }
    }
    trimmed
}

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

        if !is_same_role(current_role, next_role) {
            i += 1;
            continue;
        }

        // Consecutive matching roles detected: merge content blocks if both are strings
        let current_content = messages[i].get("content").cloned();
        let next_content = messages[i + 1].get("content").cloned();

        if let (Some(Value::String(c1)), Some(Value::String(c2))) = (current_content, next_content) {
            let merged = merge_text_contents(&c1, &c2);
            if let Some(obj) = messages[i].as_object_mut() {
                obj.insert("content".to_string(), Value::String(merged));
            }
            messages.remove(i + 1);
            modifications += 1;
            continue; // Re-check from current index
        }

        // If structured or non-string, insert synthetic bridging turn
        let bridging = opposite_role(current_role);
        let bridging_msg = serde_json::json!({
            "role": bridging,
            "content": "..."
        });
        messages.insert(i + 1, bridging_msg);
        modifications += 1;
        i += 2;
    }

    if modifications > 0 {
        debug!(modifications = modifications, "Sanitized Claude message role alternation");
    }

    modifications
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opposite_role() {
        assert_eq!(opposite_role("user"), "assistant");
        assert_eq!(opposite_role("User"), "assistant");
        assert_eq!(opposite_role("assistant"), "user");
        assert_eq!(opposite_role("other"), "user");
    }

    #[test]
    fn test_is_same_role() {
        assert!(is_same_role("user", "user"));
        assert!(is_same_role("User", "USER"));
        assert!(!is_same_role("user", "assistant"));
    }

    #[test]
    fn test_strip_leading_role_markers() {
        assert_eq!(strip_leading_role_markers("Human: Hello world"), "Hello world");
        assert_eq!(strip_leading_role_markers("Assistant: I am here"), "I am here");
        assert_eq!(strip_leading_role_markers("Plain text without marker"), "Plain text without marker");
    }

    #[test]
    fn test_sanitize_consecutive_user_turns() {
        let mut messages = vec![
            serde_json::json!({"role": "user", "content": "Question 1"}),
            serde_json::json!({"role": "user", "content": "Question 2"}),
        ];

        let mods = sanitize_claude_role_alternation(&mut messages);
        assert_eq!(mods, 1);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["content"], "Question 1\n\nQuestion 2");
    }

    #[test]
    fn test_sanitize_alternating_already_clean() {
        let mut messages = vec![
            serde_json::json!({"role": "user", "content": "Hi"}),
            serde_json::json!({"role": "assistant", "content": "Hello"}),
        ];

        let mods = sanitize_claude_role_alternation(&mut messages);
        assert_eq!(mods, 0);
        assert_eq!(messages.len(), 2);
    }
}
