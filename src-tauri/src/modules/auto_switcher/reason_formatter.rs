//! Account Switch Reason Audit Code Formatter.
//! Formats and parses machine-readable switch reasons for the audit trail and task history ledger.

use serde::{Deserialize, Serialize};

/// Standardized machine-readable switch reason codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchReasonCode {
    LowQuotaPrimary,
    CooldownFallback,
    ManualOverride,
    ProactiveStartup,
    QuotaDepleted,
    ModelSwitch,
    SiblingLeaseCollision,
    Unknown,
}

impl SwitchReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LowQuotaPrimary => "LOW_QUOTA_PRIMARY",
            Self::CooldownFallback => "COOLDOWN_FALLBACK",
            Self::ManualOverride => "MANUAL_OVERRIDE",
            Self::ProactiveStartup => "PROACTIVE_STARTUP",
            Self::QuotaDepleted => "QUOTA_DEPLETED",
            Self::ModelSwitch => "MODEL_SWITCH",
            Self::SiblingLeaseCollision => "SIBLING_LEASE_COLLISION",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub fn from_str_prefix(s: &str) -> Self {
        let upper = s.trim().to_uppercase();
        if upper.starts_with("LOW_QUOTA_PRIMARY") {
            Self::LowQuotaPrimary
        } else if upper.starts_with("COOLDOWN_FALLBACK") {
            Self::CooldownFallback
        } else if upper.starts_with("MANUAL_OVERRIDE") {
            Self::ManualOverride
        } else if upper.starts_with("PROACTIVE_STARTUP") {
            Self::ProactiveStartup
        } else if upper.starts_with("QUOTA_DEPLETED") {
            Self::QuotaDepleted
        } else if upper.starts_with("MODEL_SWITCH") {
            Self::ModelSwitch
        } else if upper.starts_with("SIBLING_LEASE_COLLISION") {
            Self::SiblingLeaseCollision
        } else {
            Self::Unknown
        }
    }
}

/// Formats a standardized switch reason string: `CODE[: details]`.
pub fn format_switch_reason(code: SwitchReasonCode, detail: Option<&str>) -> String {
    match detail {
        Some(d) if !d.trim().is_empty() => format!("{}: {}", code.as_str(), d.trim()),
        _ => code.as_str().to_string(),
    }
}

/// Parses a raw reason string into a `SwitchReasonCode` and optional detail message.
pub fn parse_switch_reason(raw: &str) -> (SwitchReasonCode, Option<String>) {
    let code = SwitchReasonCode::from_str_prefix(raw);
    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    let detail = if parts.len() > 1 {
        let d = parts[1].trim();
        if !d.is_empty() {
            Some(d.to_string())
        } else {
            None
        }
    } else {
        None
    };

    (code, detail)
}
