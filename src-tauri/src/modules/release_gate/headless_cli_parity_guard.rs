//! Headless & CLI Parity Guard.
//!
//! Enforces GUI and CLI functional parity per AGENTS.md:
//! "Headless & CLI Parity: Ensure GUI configurations and lifecycle operations maintain
//! functional parity across headless servers, CLI environments, and cross-platform
//! environments. Any profile/instance duplication or settings synchronization must pass
//! through identical underlying service routes (`copy_instance_with_options`),
//! preserving project continuity (`workspaceStorage`) and allowing deep-merge settings
//! synchronization from both GUI and CLI."

use serde::{Deserialize, Serialize};

/// Represents an operational capability and its presence across GUI and CLI interfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationEntry {
    pub operation_name: String,
    pub has_gui_entry: bool,
    pub has_cli_entry: bool,
    pub service_route: Option<String>,
}

impl OperationEntry {
    /// Constructs a basic operation entry.
    pub fn new(operation_name: impl Into<String>) -> Self {
        Self {
            operation_name: operation_name.into(),
            has_gui_entry: false,
            has_cli_entry: false,
            service_route: None,
        }
    }

    /// Constructs a fully paired operation entry sharing a unified backend service route.
    pub fn paired(operation_name: impl Into<String>, service_route: impl Into<String>) -> Self {
        Self {
            operation_name: operation_name.into(),
            has_gui_entry: true,
            has_cli_entry: true,
            service_route: Some(service_route.into()),
        }
    }

    /// Constructs an operation entry existing only in GUI.
    pub fn gui_only(operation_name: impl Into<String>, service_route: Option<String>) -> Self {
        Self {
            operation_name: operation_name.into(),
            has_gui_entry: true,
            has_cli_entry: false,
            service_route,
        }
    }

    /// Constructs an operation entry existing only in CLI.
    pub fn cli_only(operation_name: impl Into<String>, service_route: Option<String>) -> Self {
        Self {
            operation_name: operation_name.into(),
            has_gui_entry: false,
            has_cli_entry: true,
            service_route,
        }
    }
}

/// Parity violations detected between GUI and CLI capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParityViolation {
    GuiOnlyOperation { operation: String },
    CliOnlyOperation { operation: String },
    MissingSharedServiceRoute { operation: String },
    DivergentServiceRoute { operation: String, reason: String },
}

/// Summary report of GUI/CLI operational parity verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParityReport {
    pub is_clean: bool,
    pub paired_operations_count: usize,
    pub violations: Vec<ParityViolation>,
}

/// Evaluates a collection of operations for GUI and CLI parity and shared backend service routes.
pub fn evaluate_parity(entries: &[OperationEntry]) -> ParityReport {
    let mut violations = Vec::new();
    let mut paired_operations_count = 0;

    for entry in entries {
        let op_name = entry.operation_name.trim();
        let mut entry_has_violation = false;

        if entry.has_gui_entry && !entry.has_cli_entry {
            violations.push(ParityViolation::GuiOnlyOperation {
                operation: op_name.to_string(),
            });
            entry_has_violation = true;
        } else if !entry.has_gui_entry && entry.has_cli_entry {
            violations.push(ParityViolation::CliOnlyOperation {
                operation: op_name.to_string(),
            });
            entry_has_violation = true;
        } else if !entry.has_gui_entry && !entry.has_cli_entry {
            violations.push(ParityViolation::MissingSharedServiceRoute {
                operation: op_name.to_string(),
            });
            entry_has_violation = true;
        }

        match &entry.service_route {
            None => {
                violations.push(ParityViolation::MissingSharedServiceRoute {
                    operation: op_name.to_string(),
                });
                entry_has_violation = true;
            }
            Some(route) if route.trim().is_empty() => {
                violations.push(ParityViolation::MissingSharedServiceRoute {
                    operation: op_name.to_string(),
                });
                entry_has_violation = true;
            }
            Some(route) => {
                let r = route.trim();
                if r.contains("gui_only")
                    || r.contains("cli_only")
                    || r.contains("divergent")
                    || r.contains("unshared")
                {
                    violations.push(ParityViolation::DivergentServiceRoute {
                        operation: op_name.to_string(),
                        reason: format!(
                            "Service route '{}' indicates divergent non-shared implementation",
                            r
                        ),
                    });
                    entry_has_violation = true;
                }
            }
        }

        if entry.has_gui_entry && entry.has_cli_entry && !entry_has_violation {
            paired_operations_count += 1;
        }
    }

    let is_clean = violations.is_empty();
    ParityReport {
        is_clean,
        paired_operations_count,
        violations,
    }
}

/// Canonical registry of standard lifecycle operations per AGENTS.md.
pub fn canonical_lifecycle_registry() -> Vec<OperationEntry> {
    vec![
        OperationEntry::paired("instance_duplication", "copy_instance_with_options"),
        OperationEntry::paired("settings_synchronization", "sync_settings_deep_merge"),
        OperationEntry::paired("instance_stop", "stop_instance"),
        OperationEntry::paired("instance_start", "start_instance"),
        OperationEntry::paired("instance_restart", "restart_instance"),
        OperationEntry::paired("instance_remove", "remove_instance"),
        OperationEntry::paired("instance_switch", "switch_instance"),
        OperationEntry::paired("sandbox_create", "test_create_sandbox"),
        OperationEntry::paired("sandbox_clean", "test_clean_sandbox"),
    ]
}
