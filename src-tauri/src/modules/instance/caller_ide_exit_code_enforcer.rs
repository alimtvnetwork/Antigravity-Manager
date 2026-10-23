//! Caller IDE Exit Code 2 Enforcer (`Imp-641`).
//!
//! Enforces caller IDE self-preservation per AGENTS.md:
//! "Caller IDE Self-Preservation & Transient Test Sandboxing: Automated tests,
//! CLI commands, and auto-switcher daemons MUST detect the caller's active IDE
//! process (`detect_caller_instance_id()`) to prevent terminating the developer's
//! working IDE session. Commands targeting the caller instance (`agm instances stop`,
//! `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Non-negotiable process exit code when an instance command targets the caller IDE session.
pub const CALLER_PROTECTION_EXIT_CODE: i32 = 2;

/// Lifecycle and management commands subject to caller IDE self-preservation protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetInstanceCommand {
    Stop,
    Remove,
    Switch,
    Restart,
    Kill,
}

impl TargetInstanceCommand {
    /// Returns the command name string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Remove => "rm",
            Self::Switch => "switch",
            Self::Restart => "restart",
            Self::Kill => "kill",
        }
    }
}

impl fmt::Display for TargetInstanceCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Policy governing caller IDE protection thresholds and overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallerProtectionPolicy {
    pub fail_fast_exit_code: i32,
    pub allow_force: bool,
}

impl Default for CallerProtectionPolicy {
    fn default() -> Self {
        Self {
            fail_fast_exit_code: CALLER_PROTECTION_EXIT_CODE,
            allow_force: true,
        }
    }
}

impl CallerProtectionPolicy {
    pub fn new(fail_fast_exit_code: i32, allow_force: bool) -> Self {
        Self {
            fail_fast_exit_code,
            allow_force,
        }
    }

    pub fn evaluate(
        &self,
        target_instance: &str,
        caller_instance: Option<&str>,
        is_forced: bool,
        command: TargetInstanceCommand,
    ) -> CallerProtectionVerdict {
        evaluate_instance_action_with_policy(
            target_instance,
            caller_instance,
            is_forced,
            command,
            self,
        )
    }
}

/// Final verdict on whether an instance operation is permitted or blocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallerProtectionVerdict {
    AllowedSafeTarget,
    AllowedForceBypass {
        caller_id: String,
    },
    BlockedCallerProtectionFailFast {
        caller_id: String,
        command: TargetInstanceCommand,
        exit_code: i32,
    },
}

impl CallerProtectionVerdict {
    /// Returns true if the command is permitted to execute.
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            Self::AllowedSafeTarget | Self::AllowedForceBypass { .. }
        )
    }

    /// Returns true if the command is blocked by caller self-preservation.
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::BlockedCallerProtectionFailFast { .. })
    }

    /// Returns the non-zero exit code if blocked, or None if allowed.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::BlockedCallerProtectionFailFast { exit_code, .. } => Some(*exit_code),
            _ => None,
        }
    }

    /// Human-readable explanation of why the action was blocked or bypassed.
    pub fn guidance_message(&self) -> Option<String> {
        match self {
            Self::BlockedCallerProtectionFailFast {
                caller_id,
                command,
                exit_code,
            } => Some(format!(
                "Command '{}' targeting active caller IDE session '{}' is blocked (exit code {}). Pass --force to override.",
                command, caller_id, exit_code
            )),
            Self::AllowedForceBypass { caller_id } => Some(format!(
                "Bypassing caller self-preservation for instance '{}' via explicit --force flag.",
                caller_id
            )),
            Self::AllowedSafeTarget => None,
        }
    }
}

/// Checks whether the target instance corresponds to the caller's active instance ID.
pub fn is_caller_target(target_instance: &str, caller_instance: Option<&str>) -> bool {
    let clean_target = target_instance.trim();
    if clean_target.is_empty() {
        return false;
    }

    match caller_instance {
        Some(caller) => {
            let clean_caller = caller.trim();
            !clean_caller.is_empty() && clean_target == clean_caller
        }
        None => false,
    }
}

/// Evaluates whether an instance lifecycle action is allowed under the standard caller protection policy.
pub fn evaluate_instance_action(
    target_instance: &str,
    caller_instance: Option<&str>,
    is_forced: bool,
    command: TargetInstanceCommand,
) -> CallerProtectionVerdict {
    let default_policy = CallerProtectionPolicy::default();
    evaluate_instance_action_with_policy(
        target_instance,
        caller_instance,
        is_forced,
        command,
        &default_policy,
    )
}

/// Evaluates whether an instance lifecycle action is allowed under a custom caller protection policy.
pub fn evaluate_instance_action_with_policy(
    target_instance: &str,
    caller_instance: Option<&str>,
    is_forced: bool,
    command: TargetInstanceCommand,
    policy: &CallerProtectionPolicy,
) -> CallerProtectionVerdict {
    if !is_caller_target(target_instance, caller_instance) {
        return CallerProtectionVerdict::AllowedSafeTarget;
    }

    let caller_id = caller_instance
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if is_forced && policy.allow_force {
        CallerProtectionVerdict::AllowedForceBypass { caller_id }
    } else {
        CallerProtectionVerdict::BlockedCallerProtectionFailFast {
            caller_id,
            command,
            exit_code: policy.fail_fast_exit_code,
        }
    }
}
