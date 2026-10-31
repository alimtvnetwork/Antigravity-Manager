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

        // 2. Mask email addresses: keep 4 chars, mask user part with ***, hide @gmail
        let masked = EMAIL_REGEX.replace_all(&no_tokens, |caps: &regex::Captures| {
            let user = &caps[1];
            let domain = &caps[2];
            let prefix: String = user.chars().take(4).collect();
            let lower_domain = domain.to_ascii_lowercase();
            if lower_domain.contains("gmail") || lower_domain.contains("googlemail") {
                format!("{prefix}***")
            } else {
                format!("{prefix}***@{domain}")
            }
        });

        masked.to_string()
    }
}
