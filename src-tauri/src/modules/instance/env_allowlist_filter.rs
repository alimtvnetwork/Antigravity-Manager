//! Sandboxed environment variable allowlist filter.
//! Scrubs sensitive master proxy credentials and leaking developer API keys
//! before launching sandboxed Antigravity IDE child processes.

use std::collections::HashSet;

pub struct EnvAllowlistFilter;

impl EnvAllowlistFilter {
    /// Sanitizes an environment variable collection, filtering out proxy secrets and credential tokens.
    pub fn sanitize_environment(
        env_vars: impl IntoIterator<Item = (String, String)>,
    ) -> Vec<(String, String)> {
        let blocked_exact: HashSet<&'static str> = [
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY",
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
            "SUPABASE_SERVICE_ROLE_KEY",
            "TELEGRAM_BOT_TOKEN",
            "AGM_MASTER_KEY",
            "AGM_ADMIN_SECRET",
        ]
        .into_iter()
        .collect();

        let mut sanitized = Vec::new();

        for (k, v) in env_vars {
            let upper = k.to_ascii_uppercase();

            // Block explicit secret keys
            if blocked_exact.contains(upper.as_str()) {
                continue;
            }

            // Block AGM internal session tokens
            if upper.starts_with("AGM_INTERNAL_") || upper.starts_with("AGM_SECRET_") {
                continue;
            }

            sanitized.push((k, v));
        }

        sanitized
    }
}
