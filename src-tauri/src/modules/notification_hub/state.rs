use once_cell::sync::Lazy;
use std::sync::Mutex;

use super::*;

pub(crate) static PREVIOUS_EMAIL_STATE: Lazy<Mutex<Option<String>>> =
    Lazy::new(|| Mutex::new(None));

pub(crate) static LAST_SWITCH_DISPATCH: Lazy<Mutex<(String, i64)>> =
    Lazy::new(|| Mutex::new((String::new(), 0)));

/// Record the previous account email prior to switching so telemetry can report old_email accurately
pub fn record_previous_email(email: &str) {
    let trimmed = email.trim();
    if !trimmed.is_empty() {
        if let Ok(mut guard) = PREVIOUS_EMAIL_STATE.lock() {
            *guard = Some(trimmed.to_string());
        }
    }
}

/// Extract up to max_words words from prompt text and return (snippet, word_count)
pub fn extract_words_preview(text: &str, max_words: usize) -> (String, usize) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (String::new(), 0);
    }
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let count = words.len();
    if count <= max_words {
        (trimmed.to_string(), count)
    } else {
        (format!("{}...", words[..max_words].join(" ")), count)
    }
}

/// Escape text for Telegram HTML parse_mode
pub fn escape_telegram_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Deduplicate project and resource names while preserving insertion order
pub fn deduplicate_names<I, S>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for it in items {
        let s = it.as_ref().trim();
        if !s.is_empty() && seen.insert(s.to_lowercase()) {
            out.push(s.to_string());
        }
    }
    out
}
