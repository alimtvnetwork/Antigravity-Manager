#[cfg(test)]
mod tests {
    use super::super::agent_prompt_sanitizer_guard::{
        audit_sanitization_rule, detect_destructive_keyword_replacement, evaluate_prompt_integrity,
        sanitize_prompt_payload, sanitize_prompt_text, DestructiveViolationType, PromptPayload,
        RedactionKind, REDACTED_AUTH, REDACTED_SECRET,
    };

    #[test]
    fn test_sanitize_openai_and_anthropic_keys() {
        let text = "Here is my OpenAI key sk-abcdef1234567890abcdef123456 and Anthropic key sk-ant-api03-abcdef1234567890.";
        let report = sanitize_prompt_text(text);

        assert!(report.has_redactions);
        assert_eq!(report.redacted_count, 2);
        assert!(!report.sanitized_text.contains("sk-abcdef1234567890abcdef123456"));
        assert!(!report.sanitized_text.contains("sk-ant-api03"));
        assert!(report.sanitized_text.contains(REDACTED_SECRET));
    }

    #[test]
    fn test_sanitize_gemini_key() {
        let text = "Using Gemini key AIzaSyD1234567890abcdef1234567890abcdef for model calls.";
        let report = sanitize_prompt_text(text);

        assert!(report.has_redactions);
        assert_eq!(report.redacted_count, 1);
        assert!(!report.sanitized_text.contains("AIzaSyD1234567890"));
        assert_eq!(
            report.sanitized_text,
            format!("Using Gemini key {REDACTED_SECRET} for model calls.")
        );
    }

    #[test]
    fn test_sanitize_bearer_and_auth_headers() {
        let text = "Request Header: Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.abcdef\nAlso Basic dXNlcjpwYXNzd29yZDEyMzQ1Ng==";
        let report = sanitize_prompt_text(text);

        assert!(report.has_redactions);
        assert!(report.sanitized_text.contains(REDACTED_AUTH));
        assert!(!report.sanitized_text.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(!report.sanitized_text.contains("dXNlcjpwYXNzd29yZDEyMzQ1Ng=="));
    }

    #[test]
    fn test_sanitize_database_uri_and_password() {
        let text = r#"db_conn = "postgres://admin:SuperSecretPass123@db.prod.internal:5432/main"
password = "ExtremelySecretPassword987!""#;
        let report = sanitize_prompt_text(text);

        assert!(report.has_redactions);
        assert!(!report.sanitized_text.contains("SuperSecretPass123"));
        assert!(!report.sanitized_text.contains("ExtremelySecretPassword987!"));
        assert!(report.sanitized_text.contains("postgres://admin:REDACTED_SECRET@db.prod.internal:5432/main"));
        assert!(report.sanitized_text.contains(r#"password = "REDACTED_SECRET""#));
    }

    #[test]
    fn test_sanitize_pem_private_key() {
        let pem = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0Y123456789abcdef\n-----END RSA PRIVATE KEY-----";
        let report = sanitize_prompt_text(pem);

        assert!(report.has_redactions);
        assert!(!report.sanitized_text.contains("MIIEowIBAAKCAQEA0Y123456789abcdef"));
        assert!(report.sanitized_text.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(report.sanitized_text.contains(REDACTED_SECRET));
        assert!(report.sanitized_text.contains("-----END PRIVATE KEY-----"));
    }

    #[test]
    fn test_benign_text_with_keywords_preserved() {
        let text = "The primary key index accelerates keyboard navigation and tokenizes phrases without requiring a password prompt.";
        let report = sanitize_prompt_text(text);

        // Benign text must remain 100% untouched
        assert!(!report.has_redactions);
        assert_eq!(report.redacted_count, 0);
        assert_eq!(report.sanitized_text, text);
    }

    #[test]
    fn test_prompt_payload_sanitization_and_integrity_success() {
        let original = PromptPayload {
            system_prompt: Some("You are an assistant. Do not leak credentials.".to_string()),
            user_query: "Here is my secret key sk-12345678901234567890123456. Please inspect it.".to_string(),
            context_messages: vec!["Hello assistant".to_string()],
        };

        let (sanitized, integrity) = sanitize_prompt_payload(&original);

        assert!(integrity.is_intact);
        assert!(integrity.system_prompt_preserved);
        assert!(integrity.user_query_preserved);
        assert!(integrity.context_messages_preserved);
        assert!(integrity.violations.is_empty());

        assert_eq!(
            sanitized.system_prompt.unwrap(),
            "You are an assistant. Do not leak credentials."
        );
        assert!(sanitized.user_query.contains(REDACTED_SECRET));
        assert!(!sanitized.user_query.contains("sk-12345678901234567890123456"));
    }

    #[test]
    fn test_evaluate_prompt_integrity_detects_system_wipeout() {
        let original = PromptPayload {
            system_prompt: Some("Essential system instructions that guide agent behavior across conversations.".to_string()),
            user_query: "What is 2 + 2?".to_string(),
            context_messages: vec![],
        };

        let corrupted = PromptPayload {
            system_prompt: Some("".to_string()), // Wiped out
            user_query: "What is 2 + 2?".to_string(),
            context_messages: vec![],
        };

        let integrity = evaluate_prompt_integrity(&original, &corrupted);
        assert!(!integrity.is_intact);
        assert!(!integrity.system_prompt_preserved);
        assert!(integrity.violations.iter().any(|v| v.contains("wiped out to empty")));
    }

    #[test]
    fn test_evaluate_prompt_integrity_detects_user_query_wipeout() {
        let original = PromptPayload {
            system_prompt: Some("System prompt".to_string()),
            user_query: "Long user instruction with question details and background context.".to_string(),
            context_messages: vec![],
        };

        let corrupted = PromptPayload {
            system_prompt: Some("System prompt".to_string()),
            user_query: "".to_string(), // Wiped out
            context_messages: vec![],
        };

        let integrity = evaluate_prompt_integrity(&original, &corrupted);
        assert!(!integrity.is_intact);
        assert!(!integrity.user_query_preserved);
        assert!(integrity.violations.iter().any(|v| v.contains("User query was wiped out")));
    }

    #[test]
    fn test_detect_destructive_keyword_replacement_mangled_words() {
        let original = "Use the keyboard shortcuts to navigate and tokenize the output.";
        let naive_mangled = "Use the REDACTED_SECRETboard shortcuts to navigate and REDACTED_SECRETize the output.";

        let violations = detect_destructive_keyword_replacement(original, naive_mangled);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.violation_type == DestructiveViolationType::MangledBenignWord));
        assert!(violations.iter().any(|v| v.keyword == "REDACTED_SECRETboard"));
    }

    #[test]
    fn test_detect_destructive_keyword_replacement_prompt_wipeout() {
        let original = "This is a detailed 50 character user instruction query.";
        let wiped = "";

        let violations = detect_destructive_keyword_replacement(original, wiped);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.violation_type == DestructiveViolationType::PromptWipeout));
    }

    #[test]
    fn test_audit_sanitization_rule_rejects_bare_keywords() {
        assert!(audit_sanitization_rule("key").is_err());
        assert!(audit_sanitization_rule("token").is_err());
        assert!(audit_sanitization_rule("auth").is_err());
        assert!(audit_sanitization_rule("password").is_err());
        assert!(audit_sanitization_rule("pwd").is_err());
    }

    #[test]
    fn test_audit_sanitization_rule_accepts_valid_regex() {
        assert!(audit_sanitization_rule(r"\bsk-[A-Za-z0-9]{20,}\b").is_ok());
        assert!(audit_sanitization_rule(r"(?i)\b(bearer\s+)[A-Za-z0-9_\-\.]{16,}").is_ok());
    }
}
