#[cfg(test)]
mod tests {
    use super::super::role_normalization_validator::{validate_turn_order, CanonicalRole};

    #[test]
    fn test_parse_valid_roles() {
        assert_eq!(CanonicalRole::parse("user"), Some(CanonicalRole::User));
        assert_eq!(CanonicalRole::parse("human"), Some(CanonicalRole::User));
        assert_eq!(CanonicalRole::parse("model"), Some(CanonicalRole::Model));
        assert_eq!(
            CanonicalRole::parse("assistant"),
            Some(CanonicalRole::Assistant)
        );
        assert_eq!(CanonicalRole::parse("bot"), Some(CanonicalRole::Assistant));
        assert_eq!(CanonicalRole::parse("system"), Some(CanonicalRole::System));
        assert_eq!(
            CanonicalRole::parse("developer"),
            Some(CanonicalRole::Developer)
        );
        assert_eq!(CanonicalRole::parse("tool"), Some(CanonicalRole::Tool));
        assert_eq!(CanonicalRole::parse("function"), Some(CanonicalRole::Tool));
    }

    #[test]
    fn test_parse_case_and_whitespace() {
        assert_eq!(CanonicalRole::parse("  USER \n"), Some(CanonicalRole::User));
        assert_eq!(
            CanonicalRole::parse("Assistant "),
            Some(CanonicalRole::Assistant)
        );
        assert_eq!(
            CanonicalRole::parse("DEVELOPER"),
            Some(CanonicalRole::Developer)
        );
    }

    #[test]
    fn test_parse_invalid_roles() {
        assert_eq!(CanonicalRole::parse(""), None);
        assert_eq!(CanonicalRole::parse("agent"), None);
        assert_eq!(CanonicalRole::parse("moderator"), None);
    }

    #[test]
    fn test_to_gemini_role_mapping() {
        assert_eq!(CanonicalRole::User.to_gemini_role(), "user");
        assert_eq!(CanonicalRole::Tool.to_gemini_role(), "user");
        assert_eq!(CanonicalRole::Model.to_gemini_role(), "model");
        assert_eq!(CanonicalRole::Assistant.to_gemini_role(), "model");
        assert_eq!(CanonicalRole::System.to_gemini_role(), "user");
        assert_eq!(CanonicalRole::Developer.to_gemini_role(), "user");
    }

    #[test]
    fn test_to_openai_role_mapping() {
        assert_eq!(CanonicalRole::User.to_openai_role(), "user");
        assert_eq!(CanonicalRole::Model.to_openai_role(), "assistant");
        assert_eq!(CanonicalRole::Assistant.to_openai_role(), "assistant");
        assert_eq!(CanonicalRole::System.to_openai_role(), "system");
        assert_eq!(CanonicalRole::Developer.to_openai_role(), "developer");
        assert_eq!(CanonicalRole::Tool.to_openai_role(), "tool");
    }

    #[test]
    fn test_to_claude_role_mapping() {
        assert_eq!(CanonicalRole::User.to_claude_role(), "user");
        assert_eq!(CanonicalRole::Model.to_claude_role(), "assistant");
        assert_eq!(CanonicalRole::Assistant.to_claude_role(), "assistant");
        assert_eq!(CanonicalRole::System.to_claude_role(), "user");
        assert_eq!(CanonicalRole::Developer.to_claude_role(), "user");
        assert_eq!(CanonicalRole::Tool.to_claude_role(), "user");
    }

    #[test]
    fn test_validate_turn_order() {
        assert!(!validate_turn_order(&[]));
        assert!(!validate_turn_order(&[CanonicalRole::Model]));
        assert!(validate_turn_order(&[CanonicalRole::User]));
        assert!(validate_turn_order(&[
            CanonicalRole::User,
            CanonicalRole::Model
        ]));
        assert!(validate_turn_order(&[
            CanonicalRole::System,
            CanonicalRole::Assistant
        ]));
    }
}
