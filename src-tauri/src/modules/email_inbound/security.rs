use base64::prelude::*;

use super::*;
use crate::modules::email_vault_db;

/// Check sliding 10-second debounce stack
/// Returns true if execution should proceed, false if throttled
pub fn check_debounce_rate_limit(sender: &str, command: &str, target: &str, now: i64) -> bool {
    let key = format!(
        "{}:{}:{}",
        sender.trim().to_lowercase(),
        command.trim().to_lowercase(),
        target.trim().to_lowercase()
    );
    let mut stack = DEBOUNCE_STACK.lock().unwrap();

    if let Some(record) = stack.get_mut(&key) {
        if now - record.first_seen < 10 {
            record.count += 1;
            // Within 10s window: allow at most 2 calls through (initial + 1 retry)
            return record.count <= 2;
        } else {
            *record = DebounceRecord {
                first_seen: now,
                count: 1,
                has_acked: true,
                has_completed: false,
            };
            true
        }
    } else {
        stack.insert(
            key,
            DebounceRecord {
                first_seen: now,
                count: 1,
                has_acked: true,
                has_completed: false,
            },
        );
        true
    }
}

/// Extract clean email from RFC From header (e.g. "User <user@example.com>")
pub fn extract_email_address(raw_from: &str) -> String {
    let raw = raw_from.trim();
    if let Some(start) = raw.find('<') {
        if let Some(end) = raw.find('>') {
            if end > start {
                return raw[start + 1..end].trim().to_string();
            }
        }
    }
    raw.to_string()
}

/// Verify sender email against active notify recipients and vault accounts
pub fn is_authorized_notifier(sender_raw: &str) -> bool {
    let clean = extract_email_address(sender_raw).to_lowercase();
    if clean.is_empty() {
        return false;
    }

    if let Ok(recipients) = email_vault_db::list_notify_recipients() {
        for r in recipients {
            if r.is_active && r.email.trim().eq_ignore_ascii_case(&clean) {
                return true;
            }
        }
    }

    if let Ok(accounts) = email_vault_db::list_email_accounts() {
        for acc in accounts {
            if acc.is_active && acc.email.trim().eq_ignore_ascii_case(&clean) {
                return true;
            }
        }
    }

    false
}
