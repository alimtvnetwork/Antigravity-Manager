//! Caller IDE Exit Code 2 Enforcer (`Imp-693`, Step 279).
//!
//! Enforces caller IDE self-preservation per AGENTS.md:
//! "Caller IDE Self-Preservation & Transient Test Sandboxing: Automated tests,
//! CLI commands, and auto-switcher daemons MUST detect the caller's active IDE
//! process (`detect_caller_instance_id()`) to prevent terminating the developer's
//! working IDE session. Commands targeting the caller instance (`agm instances stop`,
//! `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Non-negotiable process exit code when a destructive command targets the caller IDE session without `--force`.
pub const CALLER_PROTECTION_EXIT_CODE: i32 = 2;

/// Instance command types subject to caller IDE interception and protection checks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstanceCommandType {
    Stop,
    Remove,
    Switch,
    Restart,
    NonDestructive(String),
}

impl InstanceCommandType {
    /// Returns true if this command performs a destructive action (stop, kill, removal, switch, restart).
    pub fn is_destructive(&self) -> bool {
        match self {
            Self::Stop | Self::Remove | Self::Switch | Self::Restart => true,
            Self::NonDestructive(_) => false,
        }
    }

    /// Returns the canonical CLI subcommand token.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Stop => "stop",
            Self::Remove => "rm",
            Self::Switch => "switch",
            Self::Restart => "restart",
            Self::NonDestructive(ref name) => name.as_str(),
        }
    }
}

impl fmt::Display for InstanceCommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Contextual details of the caller instance initiating an instance operation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CallerInstanceContext {
    pub caller_instance_id: Option<String>,
    pub caller_pid: Option<u32>,
    pub caller_ide_name: Option<String>,
}

impl CallerInstanceContext {
    pub fn new(
        caller_instance_id: Option<String>,
        caller_pid: Option<u32>,
        caller_ide_name: Option<String>,
    ) -> Self {
        Self {
            caller_instance_id,
            caller_pid,
            caller_ide_name,
        }
    }

    pub fn from_caller_id(id: impl Into<String>) -> Self {
        Self {
            caller_instance_id: Some(id.into()),
            caller_pid: None,
            caller_ide_name: None,
        }
    }

    /// Checks if a given target instance matches this caller's session.
    pub fn is_caller(&self, target_id: &str) -> bool {
        let clean_target = target_id.trim();
        if clean_target.is_empty() {
            return false;
        }

        match &self.caller_instance_id {
            Some(caller_id) => {
                let clean_caller = caller_id.trim();
                !clean_caller.is_empty() && clean_target == clean_caller
            }
            None => false,
        }
    }
}

/// Structured interception result evaluating caller preservation gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandInterceptionResult {
    pub is_intercepted: bool,
    pub exit_code: i32,
    pub target_instance_id: String,
    pub is_caller_match: bool,
    pub was_force_provided: bool,
    pub reason: String,
    pub remediation_hint: String,
}

impl CommandInterceptionResult {
    /// Creates a permitted (non-intercepted) execution verdict with exit code 0.
    pub fn permitted(
        target_id: &str,
        is_caller_match: bool,
        was_force_provided: bool,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            is_intercepted: false,
            exit_code: 0,
            target_instance_id: target_id.trim().to_string(),
            is_caller_match,
            was_force_provided,
            reason: reason.into(),
            remediation_hint: String::new(),
        }
    }

    /// Creates an intercepted failure verdict with exit code 2 and explicit remediation hint.
    pub fn blocked(
        target_id: &str,
        command: &InstanceCommandType,
        context: &CallerInstanceContext,
    ) -> Self {
        let clean_target = target_id.trim();
        let pid_info = context
            .caller_pid
            .map(|p| format!(" (PID {})", p))
            .unwrap_or_default();
        let ide_info = context
            .caller_ide_name
            .as_deref()
            .map(|ide| format!(" [{}]", ide))
            .unwrap_or_default();

        let reason = format!(
            "Commands targeting the caller instance ('{}'{}{}) must fail-fast with exit code 2 to prevent terminating your working IDE session.",
            clean_target, pid_info, ide_info
        );
        let remediation_hint = format!(
            "Re-run the command with '--force' to proceed: agm instances {} {} --force",
            command.as_str(),
            clean_target
        );

        Self {
            is_intercepted: true,
            exit_code: CALLER_PROTECTION_EXIT_CODE,
            target_instance_id: clean_target.to_string(),
            is_caller_match: true,
            was_force_provided: false,
            reason,
            remediation_hint,
        }
    }

    /// Formats user-facing diagnostic text for terminal display.
    pub fn format_terminal_error(&self) -> String {
        if self.is_intercepted {
            format!(
                "Error [Exit Code {}]: {}\nRemediation: {}",
                self.exit_code, self.reason, self.remediation_hint
            )
        } else {
            self.reason.clone()
        }
    }
}

impl fmt::Display for CommandInterceptionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_intercepted {
            write!(
                f,
                "[BLOCKED - exit code {}] {}. Hint: {}",
                self.exit_code, self.reason, self.remediation_hint
            )
        } else {
            write!(f, "[PERMITTED] {}", self.reason)
        }
    }
}

/// Evaluates an instance command against caller self-preservation policies.
///
/// Returns:
/// - When `target_id` matches caller and `command` is destructive and `!force`:
///   `is_intercepted = true`, `exit_code = 2`, actionable remediation instructions.
/// - When `force == true` or `target_id != caller`:
///   `is_intercepted = false`, `exit_code = 0`.
/// - When `command` is non-destructive (e.g. status, logs):
///   `is_intercepted = false`, `exit_code = 0`.
pub fn evaluate_instance_command(
    target_id: &str,
    command: InstanceCommandType,
    force: bool,
    context: &CallerInstanceContext,
) -> CommandInterceptionResult {
    let clean_target = target_id.trim();
    let is_match = context.is_caller(clean_target);

    if !is_match {
        return CommandInterceptionResult::permitted(
            clean_target,
            false,
            force,
            format!(
                "Target instance '{}' is safe: does not match caller IDE session.",
                clean_target
            ),
        );
    }

    if !command.is_destructive() {
        return CommandInterceptionResult::permitted(
            clean_target,
            true,
            force,
            format!(
                "Command '{}' is non-destructive and permitted on caller instance '{}'.",
                command, clean_target
            ),
        );
    }

    if force {
        return CommandInterceptionResult::permitted(
            clean_target,
            true,
            true,
            format!(
                "Bypassing caller self-preservation for instance '{}' via explicit --force flag.",
                clean_target
            ),
        );
    }

    CommandInterceptionResult::blocked(clean_target, &command, context)
}

// =========================================================================
// Legacy compatibility layer for TargetInstanceCommand & CallerProtectionPolicy
// =========================================================================

/// Legacy enum for instance lifecycle actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetInstanceCommand {
    Stop,
    Remove,
    Switch,
    Restart,
    Kill,
}

impl TargetInstanceCommand {
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

impl From<TargetInstanceCommand> for InstanceCommandType {
    fn from(cmd: TargetInstanceCommand) -> Self {
        match cmd {
            TargetInstanceCommand::Stop => InstanceCommandType::Stop,
            TargetInstanceCommand::Remove => InstanceCommandType::Remove,
            TargetInstanceCommand::Switch => InstanceCommandType::Switch,
            TargetInstanceCommand::Restart => InstanceCommandType::Restart,
            TargetInstanceCommand::Kill => InstanceCommandType::Stop,
        }
    }
}

/// Legacy policy governing caller protection.
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

/// Legacy verdict for caller protection.
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
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            Self::AllowedSafeTarget | Self::AllowedForceBypass { .. }
        )
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::BlockedCallerProtectionFailFast { .. })
    }

    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::BlockedCallerProtectionFailFast { exit_code, .. } => Some(*exit_code),
            _ => None,
        }
    }

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
