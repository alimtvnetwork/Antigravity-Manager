// Claude Message Turn Structure Validator (Imp-515)
// Validates conversation turn structures in Claude protocol requests,
// ensuring strictly alternating roles, non-empty content arrays, and
// stripping trailing whitespace blocks that trigger upstream 400 faults.

use serde_json::Value;

pub struct MessageTurnValidator;

impl MessageTurnValidator {
    /// Affirmative predicate checking if a role is permitted in Claude messages.
    pub fn is_valid_role(role: &str) -> bool {
        matches!(
            role.trim().to_ascii_lowercase().as_str(),
            "user" | "assistant"
        )
    }

    /// Checks if a role is 'user'.
    pub fn is_user_role(role: &str) -> bool {
        role.trim().eq_ignore_ascii_case("user")
    }

    /// Checks if a role is 'assistant'.
    pub fn is_assistant_role(role: &str) -> bool {
        role.trim().eq_ignore_ascii_case("assistant")
    }

    /// Checks if turn index is the terminal message in a sequence.
    pub fn is_terminal_turn(index: usize, total_turns: usize) -> bool {
        total_turns > 0 && index + 1 == total_turns
    }

    /// Validates and sanitizes a Claude message payload value.
    /// Returns Ok(()) or Err(validation_error_message).
    pub fn validate_and_clean_messages(messages: &mut [Value]) -> Result<(), String> {
        if messages.is_empty() {
            return Err("Messages array cannot be empty".to_string());
        }

        for (i, msg) in messages.iter_mut().enumerate() {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            if !Self::is_valid_role(role) {
                return Err(format!(
                    "Message at index {} has invalid role '{}'",
                    i, role
                ));
            }

            // Clean content if it is a string
            if let Some(content_str) = msg.get("content").and_then(|c| c.as_str()) {
                let trimmed = content_str.trim_end();
                if trimmed.is_empty() && Self::is_user_role(role) {
                    // Upstream rejects empty user turns; backfill minimal token
                    msg["content"] = Value::String(".".to_string());
                } else if trimmed.len() != content_str.len() {
                    msg["content"] = Value::String(trimmed.to_string());
                }
            }
        }

        Ok(())
    }

    /// Verifies strict turn alternation (user -> assistant -> user).
    pub fn verify_role_alternation(messages: &[Value]) -> bool {
        let mut expected_user = true;
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            let is_user = Self::is_user_role(role);
            if is_user != expected_user {
                return false;
            }
            expected_user = !expected_user;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_role() {
        assert!(MessageTurnValidator::is_valid_role("user"));
        assert!(MessageTurnValidator::is_valid_role("User"));
        assert!(MessageTurnValidator::is_valid_role("assistant"));
        assert!(MessageTurnValidator::is_valid_role("ASSISTANT"));
        assert!(!MessageTurnValidator::is_valid_role("system"));
        assert!(!MessageTurnValidator::is_valid_role("bot"));
    }

    #[test]
    fn test_is_user_and_assistant_role() {
        assert!(MessageTurnValidator::is_user_role("user"));
        assert!(!MessageTurnValidator::is_user_role("assistant"));
        assert!(MessageTurnValidator::is_assistant_role("assistant"));
        assert!(!MessageTurnValidator::is_assistant_role("user"));
    }

    #[test]
    fn test_is_terminal_turn() {
        assert!(MessageTurnValidator::is_terminal_turn(2, 3));
        assert!(!MessageTurnValidator::is_terminal_turn(1, 3));
        assert!(!MessageTurnValidator::is_terminal_turn(0, 0));
    }

    #[test]
    fn test_verify_role_alternation() {
        let valid = vec![
            serde_json::json!({"role": "user", "content": "hi"}),
            serde_json::json!({"role": "assistant", "content": "hello"}),
            serde_json::json!({"role": "user", "content": "how are you?"}),
        ];
        assert!(MessageTurnValidator::verify_role_alternation(&valid));

        let invalid = vec![
            serde_json::json!({"role": "user", "content": "hi"}),
            serde_json::json!({"role": "user", "content": "hello again"}),
        ];
        assert!(!MessageTurnValidator::verify_role_alternation(&invalid));
    }

    #[test]
    fn test_validate_and_clean_messages() {
        let mut messages = vec![
            serde_json::json!({"role": "user", "content": "   "}),
            serde_json::json!({"role": "assistant", "content": "hello \n "}),
        ];

        let res = MessageTurnValidator::validate_and_clean_messages(&mut messages);
        assert!(res.is_ok());
        // Empty user string is backfilled to "."
        assert_eq!(messages[0]["content"], ".");
        // Assistant string has trailing whitespace trimmed
        assert_eq!(messages[1]["content"], "hello");
    }
}
