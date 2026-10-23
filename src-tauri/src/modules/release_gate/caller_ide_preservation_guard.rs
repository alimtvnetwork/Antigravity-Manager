//! Caller IDE Self-Preservation & Termination Safety Guard.
//!
//! Enforces caller preservation per AGENTS.md:
//! Automated tests, CLI commands, and auto-switcher daemons MUST detect the caller's active
//! IDE process (`detect_caller_instance_id()`) to prevent terminating the developer's working
//! IDE session. Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`)
//! must fail-fast with exit code 2 unless `--force` is explicitly provided.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationSafetyViolation {
    CallerSelfPreservationBlocked {
        target_id: String,
        caller_id: String,
        exit_code: i32,
        guidance: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminationSafetyReport {
    pub is_permitted: bool,
    pub is_caller_instance: bool,
    pub target_id: String,
    pub caller_id: Option<String>,
    pub violation: Option<TerminationSafetyViolation>,
}

/// Evaluates whether an instance termination or destructive operation is safe to proceed.
pub fn validate_termination_safety(
    target_id: &str,
    detected_caller_id: Option<&str>,
    is_force: bool,
) -> TerminationSafetyReport {
    let clean_target = target_id.trim();
    let clean_caller = detected_caller_id.map(|s| s.trim());

    let is_caller = match clean_caller {
        Some(caller) if !caller.is_empty() => clean_target == caller,
        _ => false,
    };

    if is_caller && !is_force {
        let caller_str = clean_caller.unwrap_or("").to_string();
        let violation = TerminationSafetyViolation::CallerSelfPreservationBlocked {
            target_id: clean_target.to_string(),
            caller_id: caller_str,
            exit_code: 2,
            guidance: format!(
                "Target instance '{}' is your currently active caller IDE session. Terminating it is blocked to prevent workspace loss. Pass --force to override.",
                clean_target
            ),
        };
        TerminationSafetyReport {
            is_permitted: false,
            is_caller_instance: true,
            target_id: clean_target.to_string(),
            caller_id: clean_caller.map(|s| s.to_string()),
            violation: Some(violation),
        }
    } else {
        TerminationSafetyReport {
            is_permitted: true,
            is_caller_instance: is_caller,
            target_id: clean_target.to_string(),
            caller_id: clean_caller.map(|s| s.to_string()),
            violation: None,
        }
    }
}
