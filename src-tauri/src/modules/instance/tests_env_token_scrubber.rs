use super::*;

#[cfg(test)]
mod tests {
    use super::env_token_scrubber::{EnvTokenScrubber, SENSITIVE_TOKEN_KEYS};

    #[test]
    fn test_is_sensitive_key_exact_match() {
        for key in SENSITIVE_TOKEN_KEYS {
            assert!(EnvTokenScrubber::is_sensitive_key(key));
        }
    }

    #[test]
    fn test_is_sensitive_key_case_insensitive() {
        assert!(EnvTokenScrubber::is_sensitive_key("github_token"));
        assert!(EnvTokenScrubber::is_sensitive_key("openai_api_key"));
        assert!(EnvTokenScrubber::is_sensitive_key("Gemini_Api_Key"));
        assert!(EnvTokenScrubber::is_sensitive_key("cLaUdE_aPi_KeY"));
    }

    #[test]
    fn test_is_sensitive_key_benign_keys() {
        assert!(!EnvTokenScrubber::is_sensitive_key("PATH"));
        assert!(!EnvTokenScrubber::is_sensitive_key("HOME"));
        assert!(!EnvTokenScrubber::is_sensitive_key("USER"));
        assert!(!EnvTokenScrubber::is_sensitive_key("APP_ENV"));
    }

    #[test]
    fn test_scrub_environment_filters_sensitive_tokens() {
        let env_vars = vec![
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("GITHUB_TOKEN".to_string(), "ghp_secret123".to_string()),
            ("NODE_ENV".to_string(), "production".to_string()),
            ("OPENAI_API_KEY".to_string(), "sk-live-xyz".to_string()),
            ("GEMINI_API_KEY".to_string(), "AIzaSySecret".to_string()),
            ("EDITOR".to_string(), "vim".to_string()),
        ];

        let (filtered, scrubbed_count) = EnvTokenScrubber::scrub_environment(env_vars);
        assert_eq!(scrubbed_count, 3);
        assert_eq!(filtered.len(), 3);

        let keys: Vec<&str> = filtered.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"PATH"));
        assert!(keys.contains(&"NODE_ENV"));
        assert!(keys.contains(&"EDITOR"));
        assert!(!keys.contains(&"GITHUB_TOKEN"));
        assert!(!keys.contains(&"OPENAI_API_KEY"));
        assert!(!keys.contains(&"GEMINI_API_KEY"));
    }

    #[test]
    fn test_scrub_environment_empty_input() {
        let (filtered, scrubbed_count) = EnvTokenScrubber::scrub_environment(vec![]);
        assert_eq!(scrubbed_count, 0);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_scrub_environment_all_clean() {
        let env_vars = vec![
            ("USER".to_string(), "developer".to_string()),
            ("SHELL".to_string(), "/bin/bash".to_string()),
        ];
        let (filtered, scrubbed_count) = EnvTokenScrubber::scrub_environment(env_vars);
        assert_eq!(scrubbed_count, 0);
        assert_eq!(filtered.len(), 2);
    }
}
