//! Unit tests for Headless & CLI Profile Workspace Storage Continuity Verifier (`Imp-638`).

use super::workspace_storage_continuity_verifier::{
    verify_service_route, verify_workspace_continuity, ContinuityViolation, WorkspaceEntry,
    CANONICAL_COPY_SERVICE_ROUTE,
};

#[test]
fn test_canonical_service_route_passes() {
    let result = verify_service_route(CANONICAL_COPY_SERVICE_ROUTE);
    assert!(result.is_ok());
}

#[test]
fn test_bypassed_service_route_returns_violation() {
    let result = verify_service_route("custom_direct_fs_copy");
    match result {
        Err(ContinuityViolation::BypassedStandardServiceRoute { route_used, expected }) => {
            assert_eq!(route_used, "custom_direct_fs_copy");
            assert_eq!(expected, CANONICAL_COPY_SERVICE_ROUTE);
        }
        _ => panic!("Expected BypassedStandardServiceRoute error"),
    }
}

#[test]
fn test_clean_workspace_continuity_passes() {
    let source = vec![
        WorkspaceEntry::new("ws-alpha-101", true, 4096),
        WorkspaceEntry::new("ws-beta-202", true, 8192),
    ];
    let target = vec![
        WorkspaceEntry::new("ws-alpha-101", true, 4096),
        WorkspaceEntry::new("ws-beta-202", true, 8192),
    ];

    let report = verify_workspace_continuity(&source, &target, CANONICAL_COPY_SERVICE_ROUTE)
        .expect("continuity verification should succeed");

    assert!(report.is_preserved);
    assert!(report.is_clean());
    assert_eq!(report.source_workspace_count, 2);
    assert_eq!(report.target_workspace_count, 2);
    assert!(report.missing_workspace_ids.is_empty());
    assert!(report.corrupted_workspaces.is_empty());
}

#[test]
fn test_missing_workspace_ids_detected() {
    let source = vec![
        WorkspaceEntry::new("ws-alpha-101", true, 4096),
        WorkspaceEntry::new("ws-beta-202", true, 8192),
    ];
    let target = vec![
        WorkspaceEntry::new("ws-alpha-101", true, 4096),
    ];

    let report = verify_workspace_continuity(&source, &target, CANONICAL_COPY_SERVICE_ROUTE)
        .expect("continuity verification should succeed");

    assert!(!report.is_preserved);
    assert!(!report.is_clean());
    assert_eq!(report.missing_workspace_ids, vec!["ws-beta-202"]);
}

#[test]
fn test_corrupted_state_db_detected() {
    let source = vec![
        WorkspaceEntry::new("ws-gamma-303", true, 2048),
    ];
    let target = vec![
        WorkspaceEntry::new("ws-gamma-303", false, 0),
    ];

    let report = verify_workspace_continuity(&source, &target, CANONICAL_COPY_SERVICE_ROUTE)
        .expect("continuity verification should succeed");

    assert!(!report.is_preserved);
    assert!(!report.is_clean());
    assert_eq!(report.corrupted_workspaces, vec!["ws-gamma-303"]);
}

#[test]
fn test_violation_display_formatting() {
    let v1 = ContinuityViolation::BypassedStandardServiceRoute {
        route_used: "raw_cp".into(),
        expected: CANONICAL_COPY_SERVICE_ROUTE.into(),
    };
    assert!(v1.to_string().contains("raw_cp"));

    let v2 = ContinuityViolation::MissingWorkspaceStorage {
        target_path: "/tmp/profile/workspaceStorage".into(),
    };
    assert!(v2.to_string().contains("/tmp/profile/workspaceStorage"));

    let v3 = ContinuityViolation::IncompleteWorkspaceTransfer {
        missing_ids: vec!["ws-1".into()],
    };
    assert!(v3.to_string().contains("1 workspaces failed"));

    let v4 = ContinuityViolation::CorruptedWorkspaceStateDb {
        workspace_id: "ws-2".into(),
        details: "malformed header".into(),
    };
    assert!(v4.to_string().contains("malformed header"));
}
