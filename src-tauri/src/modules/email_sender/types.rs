use base64::prelude::*;

use super::*;

/// Result of an email delivery attempt
#[derive(Debug, Clone)]
pub struct SendResult {
    pub is_success: bool,
    pub used_account_id: String,
    pub used_account_email: String,
    pub attempts_count: usize,
    pub message: String,
}

/// Clean recipient string into RFC 5321 pure email address
pub fn clean_recipient_email(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find('<') {
        if let Some(end) = trimmed[start + 1..].find('>') {
            let inner = &trimmed[start + 1..start + 1 + end];
            return inner.trim().to_string();
        }
    }
    trimmed
        .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'')
        .trim()
        .to_string()
}

/// Returns the current machine's node alias and local IPv4 address
pub fn get_local_node_identity() -> (String, String) {
    let name = crate::modules::email_watcher::detect_machine_name();
    let ip = crate::modules::email_watcher::detect_local_ip();
    (name, ip)
}
