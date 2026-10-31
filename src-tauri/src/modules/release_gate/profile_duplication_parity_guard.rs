//! Profile Duplication GUI/CLI Parity & Workspace Continuity Guard.
//!
//! Enforces lifecycle operation parity per AGENTS.md:
//! Ensure GUI configurations and lifecycle operations maintain functional parity across
//! headless servers, CLI environments, and cross-platform environments. Any profile/instance
//! duplication or settings synchronization must pass through identical underlying service
//! routes (`copy_instance_with_options`), preserving project continuity (`workspaceStorage`)
//! and allowing deep-merge settings synchronization from both GUI and CLI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceCopyOptions {
    pub source_id: String,
    pub target_id: String,
    pub preserve_workspace_storage: bool,
    pub sync_settings: bool,
    pub is_shared_service_route: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DuplicationParityReport {
    pub is_parity_compliant: bool,
    pub preserves_workspace_storage: bool,
    pub is_shared_service_route: bool,
    pub violation_reasons: Vec<String>,
}

/// Validates that an instance duplication request complies with headless, CLI, and GUI parity rules.
pub fn validate_duplication_parity(options: &InstanceCopyOptions) -> DuplicationParityReport {
    let mut violation_reasons = Vec::new();

    let clean_source = options.source_id.trim();
    let clean_target = options.target_id.trim();

    if clean_source.is_empty() {
        violation_reasons.push("Source instance ID cannot be empty".to_string());
    }
    if clean_target.is_empty() {
        violation_reasons.push("Target instance ID cannot be empty".to_string());
    }
    if clean_source == clean_target && !clean_source.is_empty() {
        violation_reasons.push(format!(
            "Target instance ID '{}' cannot be identical to source instance ID",
            clean_target
        ));
    }
    if !options.is_shared_service_route {
        violation_reasons.push(
            "Duplication must route through the unified 'copy_instance_with_options' core service"
                .to_string(),
        );
    }
    if !options.preserve_workspace_storage {
        violation_reasons.push(
            "Project continuity violation: 'preserve_workspace_storage' must be enabled to protect conversation state"
                .to_string(),
        );
    }

    let is_parity_compliant = violation_reasons.is_empty();

    DuplicationParityReport {
        is_parity_compliant,
        preserves_workspace_storage: options.preserve_workspace_storage,
        is_shared_service_route: options.is_shared_service_route,
        violation_reasons,
    }
}
