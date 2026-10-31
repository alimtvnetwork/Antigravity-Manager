// Environment Variable Host Token Scrubber (Imp-499)
// Scrubs sensitive host credentials and cloud tokens from child instance environment
// vectors, ensuring sandboxed IDE processes cannot leak operator API keys.

use std::collections::HashSet;

pub const SENSITIVE_TOKEN_KEYS: &[&str] = &[
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "AWS_SECRET_ACCESS_KEY",
    "GEMINI_API_KEY",
    "CLAUDE_API_KEY",
    "SUPABASE_SERVICE_ROLE_KEY",
    "STRIPE_API_KEY",
];

pub struct EnvTokenScrubber;

impl EnvTokenScrubber {
    /// Filters out key-value pairs whose variable names match known sensitive credentials.
    pub fn scrub_environment(env_vars: Vec<(String, String)>) -> (Vec<(String, String)>, usize) {
        let sensitive_set: HashSet<String> = SENSITIVE_TOKEN_KEYS
            .iter()
            .map(|k| k.to_uppercase())
            .collect();

        let mut scrubbed_count = 0;
        let filtered: Vec<(String, String)> = env_vars
            .into_iter()
            .filter(|(k, _)| {
                let upper = k.to_uppercase();
                if sensitive_set.contains(&upper) {
                    scrubbed_count += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        (filtered, scrubbed_count)
    }

    /// Checks if a single environment variable name is considered sensitive.
    pub fn is_sensitive_key(key: &str) -> bool {
        let upper = key.to_uppercase();
        SENSITIVE_TOKEN_KEYS.iter().any(|&s| s == upper)
    }
}
