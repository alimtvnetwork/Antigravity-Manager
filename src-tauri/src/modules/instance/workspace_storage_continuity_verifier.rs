//! Headless & CLI Profile Workspace Storage Continuity Verifier (`Imp-638`).
//!
//! Enforces AGENTS.md Headless & CLI Parity rules:
//! "Ensure GUI configurations and lifecycle operations maintain functional parity
//! across headless servers, CLI environments, and cross-platform environments.
//! Any profile/instance duplication or settings synchronization must pass through
//! identical underlying service routes (`copy_instance_with_options`), preserving
//! project continuity (`workspaceStorage`) and allowing deep-merge settings
//! synchronization from both GUI and CLI."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical shared service route required for all profile duplication and sync.
pub const CANONICAL_COPY_SERVICE_ROUTE: &str = "copy_instance_with_options";

/// Errors occurring during workspace storage continuity verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuityViolation {
    /// Operation bypassed the standard shared service route.
    BypassedStandardServiceRoute {
        route_used: String,
        expected: String,
    },
    /// Target instance is missing the workspaceStorage root directory.
    MissingWorkspaceStorage { target_path: String },
    /// Not all workspace IDs were transferred to the target profile.
    IncompleteWorkspaceTransfer { missing_ids: Vec<String> },
    /// State database for a specific workspace is invalid or corrupted.
    CorruptedWorkspaceStateDb {
        workspace_id: String,
        details: String,
    },
}

impl fmt::Display for ContinuityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BypassedStandardServiceRoute {
                route_used,
                expected,
            } => {
                write!(
                    f,
                    "Continuity violation: operation used route '{}', expected standard '{}'",
                    route_used, expected
                )
            }
            Self::MissingWorkspaceStorage { target_path } => {
                write!(
                    f,
                    "Continuity violation: missing workspaceStorage directory at '{}'",
                    target_path
                )
            }
            Self::IncompleteWorkspaceTransfer { missing_ids } => {
                write!(
                    f,
                    "Continuity violation: {} workspaces failed to transfer: {:?}",
                    missing_ids.len(),
                    missing_ids
                )
            }
            Self::CorruptedWorkspaceStateDb {
                workspace_id,
                details,
            } => {
                write!(
                    f,
                    "Continuity violation: corrupted state.vscdb for workspace '{}': {}",
                    workspace_id, details
                )
            }
        }
    }
}

impl std::error::Error for ContinuityViolation {}

/// Represents an inspected workspace storage entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    pub workspace_id: String,
    pub has_valid_state_db: bool,
    pub storage_size_bytes: u64,
}

impl WorkspaceEntry {
    /// Creates a new workspace entry representation.
    pub fn new(
        workspace_id: impl Into<String>,
        has_valid_state_db: bool,
        storage_size_bytes: u64,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            has_valid_state_db,
            storage_size_bytes,
        }
    }
}

/// Comprehensive report detailing workspace storage continuity across instance operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceContinuityReport {
    pub is_preserved: bool,
    pub source_workspace_count: usize,
    pub target_workspace_count: usize,
    pub missing_workspace_ids: Vec<String>,
    pub corrupted_workspaces: Vec<String>,
    pub service_route_used: String,
}

impl WorkspaceContinuityReport {
    /// Returns true if all project continuity requirements are satisfied.
    pub fn is_clean(&self) -> bool {
        self.is_preserved
            && self.missing_workspace_ids.is_empty()
            && self.corrupted_workspaces.is_empty()
    }
}

/// Verifies that the invocation used the mandatory shared service route.
pub fn verify_service_route(route: &str) -> Result<(), ContinuityViolation> {
    if route == CANONICAL_COPY_SERVICE_ROUTE {
        Ok(())
    } else {
        Err(ContinuityViolation::BypassedStandardServiceRoute {
            route_used: route.to_string(),
            expected: CANONICAL_COPY_SERVICE_ROUTE.to_string(),
        })
    }
}

/// Verifies workspace storage continuity from source to target profile.
pub fn verify_workspace_continuity(
    source_entries: &[WorkspaceEntry],
    target_entries: &[WorkspaceEntry],
    service_route: &str,
) -> Result<WorkspaceContinuityReport, ContinuityViolation> {
    verify_service_route(service_route)?;

    let mut missing_ids = Vec::new();
    let mut corrupted = Vec::new();

    for src in source_entries {
        match target_entries
            .iter()
            .find(|t| t.workspace_id == src.workspace_id)
        {
            Some(tgt) => {
                if !tgt.has_valid_state_db {
                    corrupted.push(tgt.workspace_id.clone());
                }
            }
            None => {
                missing_ids.push(src.workspace_id.clone());
            }
        }
    }

    let is_preserved = missing_ids.is_empty() && corrupted.is_empty();

    Ok(WorkspaceContinuityReport {
        is_preserved,
        source_workspace_count: source_entries.len(),
        target_workspace_count: target_entries.len(),
        missing_workspace_ids: missing_ids,
        corrupted_workspaces: corrupted,
        service_route_used: service_route.to_string(),
    })
}
