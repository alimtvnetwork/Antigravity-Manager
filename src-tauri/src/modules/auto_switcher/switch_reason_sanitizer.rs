//! Switch Audit Reason Sanitizer
//! Scrubs sensitive account credentials, raw bearer tokens, and private email identifiers
//! from switch audit reasons prior to SQLite persistence and UI event emission.

use regex::Regex;
use std::sync::LazyLock;

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(ya29\.[a-zA-Z0-9_\-]+|bearer\s+[a-zA-Z0-9_\.\-]+)").unwrap()
});

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-zA-Z0-9_\.\-]+)@([a-zA-Z0-9_\.\-]+\.[a-zA-Z]{2,})").unwrap());

pub struct SwitchReasonSanitizer;

impl SwitchReasonSanitizer {
    /// Sanitizes an auto-switch reason string, masking sensitive tokens and email addresses.
    pub fn sanitize_reason(reason: &str) -> String {
        // 1. Redact OAuth tokens
        let no_tokens = TOKEN_REGEX.replace_all(reason, "[REDACTED_TOKEN]");

        // 2. Mask email addresses: keep first char, mask user part with ***
        let masked = EMAIL_REGEX.replace_all(&no_tokens, |caps: &regex::Captures| {
            let user = &caps[1];
            let domain = &caps[2];
            let first_char = user.chars().next().unwrap_or('*');
            format!("{}***@{}", first_char, domain)
        });

        masked.to_string()
    }
}
