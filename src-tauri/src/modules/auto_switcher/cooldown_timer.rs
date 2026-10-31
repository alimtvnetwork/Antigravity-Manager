//! Account cooldown countdown helper for auto-switch governance.
//! Calculates exact remaining cooldown durations and human-readable remaining time strings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownStatus {
    pub is_in_cooldown: bool,
    pub remaining_seconds: u64,
    pub human_remaining: String,
    pub expiry_timestamp: i64,
}

/// Evaluates whether an account is currently restricted by cooldown.
pub fn check_cooldown_status(
    last_switch_time: i64,
    cooldown_duration_secs: u64,
    now_sec: i64,
) -> CooldownStatus {
    if last_switch_time <= 0 || cooldown_duration_secs == 0 {
        return CooldownStatus {
            is_in_cooldown: false,
            remaining_seconds: 0,
            human_remaining: "Ready".to_string(),
            expiry_timestamp: 0,
        };
    }

    let expiry_timestamp = last_switch_time + (cooldown_duration_secs as i64);
    if now_sec >= expiry_timestamp {
        CooldownStatus {
            is_in_cooldown: false,
            remaining_seconds: 0,
            human_remaining: "Expired (Ready)".to_string(),
            expiry_timestamp,
        }
    } else {
        let remaining_seconds = (expiry_timestamp - now_sec) as u64;
        let human_remaining = format_cooldown_duration(remaining_seconds);
        CooldownStatus {
            is_in_cooldown: true,
            remaining_seconds,
            human_remaining,
            expiry_timestamp,
        }
    }
}

/// Formats remaining seconds into a concise human-readable duration (e.g. "45s", "3m 12s", "1h 15m").
pub fn format_cooldown_duration(seconds: u64) -> String {
    if seconds == 0 {
        return "0s".to_string();
    }
    if seconds < 60 {
        return format!("{}s", seconds);
    }
    let minutes = seconds / 60;
    let rem_secs = seconds % 60;
    if minutes < 60 {
        if rem_secs > 0 {
            format!("{}m {}s", minutes, rem_secs)
        } else {
            format!("{}m", minutes)
        }
    } else {
        let hours = minutes / 60;
        let rem_mins = minutes % 60;
        if rem_mins > 0 {
            format!("{}h {}m", hours, rem_mins)
        } else {
            format!("{}h", hours)
        }
    }
}
