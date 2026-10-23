// Unit Test Suite for Claude Message Turn Validator & Role Prefix Sanitizer (Imp-515 & Imp-484)
// Validates message structure, empty message rejection, trailing whitespace stripping,
// empty user content dot-backfilling, turn alternation, and consecutive role merging.

#[cfg(test)]
mod tests {
    use super::super::message_turn_validator::MessageTurnValidator;
    use super::super::role_prefix_sanitizer::sanitize_claude_role_alternation;
    use serde_json::{json, Value};

    #[test]
    fn test_validate_empty_messages_returns_err() {
        let mut msgs: Vec<Value> = vec![];
        let result = MessageTurnValidator::validate_and_clean_messages(&mut msgs);
        assert_eq!(result, Err("Messages array cannot be empty".to_string()));
    }

    #[test]
    fn test_validate_invalid_roles_fail() {
        let mut system_msg = vec![json!({"role": "system", "content": "You are an assistant"})];
        let res1 = MessageTurnValidator::validate_and_clean_messages(&mut system_msg);
        assert_eq!(
            res1,
            Err("Message at index 0 has invalid role 'system'".to_string())
        );

        let mut tool_msg = vec![
            json!({"role": "user", "content": "calc"}),
            json!({"role": "tool", "content": "42"}),
        ];
        let res2 = MessageTurnValidator::validate_and_clean_messages(&mut tool_msg);
        assert_eq!(
            res2,
            Err("Message at index 1 has invalid role 'tool'".to_string())
        );
    }

    #[test]
    fn test_validate_trims_trailing_whitespace_from_content() {
        let mut msgs = vec![
            json!({"role": "user", "content": "  hello world   \n\t"}),
            json!({"role": "assistant", "content": "greeting response  "}),
        ];

        let result = MessageTurnValidator::validate_and_clean_messages(&mut msgs);
        assert!(result.is_ok());

        assert_eq!(msgs[0]["content"], "  hello world");
        assert_eq!(msgs[1]["content"], "greeting response");
    }

    #[test]
    fn test_validate_backfills_empty_user_content_with_dot() {
        let mut msgs = vec![
            json!({"role": "user", "content": "   \n\t"}),
            json!({"role": "assistant", "content": "response"}),
        ];

        let result = MessageTurnValidator::validate_and_clean_messages(&mut msgs);
        assert!(result.is_ok());
        assert_eq!(msgs[0]["content"], ".");
    }

    #[test]
    fn test_validate_structured_array_content_untouched() {
        let mut msgs = vec![json!({
            "role": "user",
            "content": [{"type": "text", "text": "structured"}]
        })];

        let result = MessageTurnValidator::validate_and_clean_messages(&mut msgs);
        assert!(result.is_ok());
        assert!(msgs[0]["content"].is_array());
        assert_eq!(msgs[0]["content"][0]["text"], "structured");
    }

    #[test]
    fn test_verify_role_alternation_success() {
        let valid = vec![
            json!({"role": "user", "content": "1"}),
            json!({"role": "assistant", "content": "2"}),
            json!({"role": "user", "content": "3"}),
        ];
        assert!(MessageTurnValidator::verify_role_alternation(&valid));

        // Single user message is valid
        assert!(MessageTurnValidator::verify_role_alternation(&[
            json!({"role": "user", "content": "1"})
        ]));

        // Empty is trivially valid
        assert!(MessageTurnValidator::verify_role_alternation(&[]));
    }

    #[test]
    fn test_verify_role_alternation_failures() {
        let start_assistant = vec![
            json!({"role": "assistant", "content": "hello"}),
            json!({"role": "user", "content": "hi"}),
        ];
        assert!(!MessageTurnValidator::verify_role_alternation(
            &start_assistant
        ));

        let consecutive_users = vec![
            json!({"role": "user", "content": "1"}),
            json!({"role": "user", "content": "2"}),
        ];
        assert!(!MessageTurnValidator::verify_role_alternation(
            &consecutive_users
        ));

        let consecutive_assistants = vec![
            json!({"role": "user", "content": "1"}),
            json!({"role": "assistant", "content": "2"}),
            json!({"role": "assistant", "content": "3"}),
        ];
        assert!(!MessageTurnValidator::verify_role_alternation(
            &consecutive_assistants
        ));
    }

    #[test]
    fn test_sanitize_empty_and_single_turn() {
        let mut empty: Vec<Value> = vec![];
        assert_eq!(sanitize_claude_role_alternation(&mut empty), 0);
        assert!(empty.is_empty());

        let mut single = vec![json!({"role": "user", "content": "hello"})];
        assert_eq!(sanitize_claude_role_alternation(&mut single), 0);
        assert_eq!(single.len(), 1);
    }

    #[test]
    fn test_sanitize_already_alternating_messages() {
        let mut msgs = vec![
            json!({"role": "user", "content": "prompt 1"}),
            json!({"role": "assistant", "content": "reply 1"}),
            json!({"role": "user", "content": "prompt 2"}),
        ];

        let modifications = sanitize_claude_role_alternation(&mut msgs);
        assert_eq!(modifications, 0);
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn test_sanitize_consecutive_user_string_messages_merged() {
        let mut msgs = vec![
            json!({"role": "user", "content": "Hello"}),
            json!({"role": "user", "content": "World"}),
        ];

        let mods = sanitize_claude_role_alternation(&mut msgs);
        assert_eq!(mods, 1);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "Hello\n\nWorld");
    }

    #[test]
    fn test_sanitize_consecutive_assistant_string_messages_merged() {
        let mut msgs = vec![
            json!({"role": "user", "content": "query"}),
            json!({"role": "assistant", "content": "thought 1"}),
            json!({"role": "assistant", "content": "thought 2"}),
        ];

        let mods = sanitize_claude_role_alternation(&mut msgs);
        assert_eq!(mods, 1);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[1]["role"], "assistant");
        assert_eq!(msgs[1]["content"], "thought 1\n\nthought 2");
    }

    #[test]
    fn test_interop_sanitize_produces_valid_alternating_sequence() {
        let mut msgs = vec![
            json!({"role": "user", "content": "A"}),
            json!({"role": "user", "content": "B"}),
            json!({"role": "assistant", "content": "C"}),
            json!({"role": "assistant", "content": "D"}),
            json!({"role": "user", "content": "E"}),
        ];

        let mods = sanitize_claude_role_alternation(&mut msgs);
        assert_eq!(mods, 2);
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0]["content"], "A\n\nB");
        assert_eq!(msgs[1]["content"], "C\n\nD");
        assert_eq!(msgs[2]["content"], "E");

        assert!(MessageTurnValidator::verify_role_alternation(&msgs));
    }
}
