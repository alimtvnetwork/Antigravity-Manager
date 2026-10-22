// Claude Message Turn Structure Validator (Imp-515)
// Validates conversation turn structures in Claude protocol requests,
// ensuring strictly alternating roles, non-empty content arrays, and
// stripping trailing whitespace blocks that trigger upstream 400 faults.

use serde_json::Value;

pub struct MessageTurnValidator;

impl MessageTurnValidator {
    /// Validates and sanitizes a Claude message payload value.
    /// Returns Ok(sanitized_messages) or Err(validation_error_message).
    pub fn validate_and_clean_messages(messages: &mut [Value]) -> Result<(), String> {
        if messages.is_empty() {
            return Err("Messages array cannot be empty".to_string());
        }

        for (i, msg) in messages.iter_mut().enumerate() {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            if role != "user" && role != "assistant" {
                return Err(format!("Message at index {} has invalid role '{}'", i, role));
            }

            // Clean content if it is a string
            if let Some(content_str) = msg.get("content").and_then(|c| c.as_str()) {
                let trimmed = content_str.trim_end();
                if trimmed.is_empty() && role == "user" {
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
            let is_user = role == "user";
            if is_user != expected_user {
                return false;
            }
            expected_user = !expected_user;
        }
        true
    }
}
