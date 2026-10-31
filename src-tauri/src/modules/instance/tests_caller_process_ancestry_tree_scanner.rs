//! Unit tests for Caller Process Ancestry Tree Scanner (`Imp-688`).

use super::caller_process_ancestry_tree_scanner::*;

#[test]
fn test_detect_caller_inside_ide_hierarchy() {
    // Process chain: agm.exe (100) -> pwsh.exe (90) -> antigravity.exe (80)
    let table = vec![
        ProcessNode::new(100, 90, "agm.exe"),
        ProcessNode::new(90, 80, "pwsh.exe"),
        ProcessNode::new(80, 10, "antigravity.exe")
            .with_command_line("antigravity.exe --instance-id=inst-worker-1"),
        ProcessNode::new(10, 0, "explorer.exe"),
    ];

    let report = scan_process_ancestry_with_table(100, &table, None);
    assert!(report.is_caller_ide_detected);
    assert_eq!(
        report.caller_origin,
        CallerIdeOrigin::AntigravityIdeTerminal {
            instance_id: "inst-worker-1".to_string(),
            ide_pid: 80,
        }
    );
    assert_eq!(report.ide_pid(), Some(80));
    assert_eq!(
        detect_caller_instance_id_from_ancestry(&report),
        Some("inst-worker-1".to_string())
    );
    assert_eq!(report.process_chain.len(), 3);
}

#[test]
fn test_detect_caller_standalone_outside_ide() {
    // Process chain: agm.exe (200) -> cmd.exe (190) -> explorer.exe (10)
    let table = vec![
        ProcessNode::new(200, 190, "agm.exe"),
        ProcessNode::new(190, 10, "cmd.exe"),
        ProcessNode::new(10, 0, "explorer.exe"),
    ];

    let report = scan_process_ancestry_with_table(200, &table, None);
    assert!(!report.is_caller_ide_detected);
    assert_eq!(report.caller_origin, CallerIdeOrigin::StandaloneTerminal);
    assert_eq!(report.ide_pid(), None);
    assert_eq!(detect_caller_instance_id_from_ancestry(&report), None);
    assert_eq!(report.process_chain.len(), 3);
}

#[test]
fn test_detect_caller_with_env_override() {
    let table = vec![
        ProcessNode::new(300, 290, "agm.exe"),
        ProcessNode::new(290, 280, "pwsh.exe"),
        ProcessNode::new(280, 10, "antigravity.exe"),
    ];

    let env = vec![("ANTIGRAVITY_INSTANCE_ID", "env-instance-profile")];
    let report = scan_process_ancestry_with_table(300, &table, Some(&env));
    assert!(report.is_caller_ide_detected);
    assert_eq!(
        detect_caller_instance_id_from_ancestry(&report),
        Some("env-instance-profile".to_string())
    );
    assert_eq!(report.ide_pid(), Some(280));
}

#[test]
fn test_detect_caller_with_cloned_executable_name() {
    // Cloned instance binary: antigravity-worker-2.exe
    let table = vec![
        ProcessNode::new(400, 390, "agm.exe"),
        ProcessNode::new(390, 380, "pwsh.exe"),
        ProcessNode::new(380, 10, "antigravity-worker-2.exe"),
    ];

    let report = scan_process_ancestry_with_table(400, &table, None);
    assert!(report.is_caller_ide_detected);
    assert_eq!(
        detect_caller_instance_id_from_ancestry(&report),
        Some("worker-2".to_string())
    );
    assert_eq!(report.ide_pid(), Some(380));
}

#[test]
fn test_detect_caller_with_user_data_dir_arg() {
    // Standard binary with --user-data-dir
    let table = vec![
        ProcessNode::new(500, 490, "agm.exe"),
        ProcessNode::new(490, 480, "bash"),
        ProcessNode::new(480, 1, "antigravity").with_command_line(
            "antigravity --user-data-dir=\"/home/user/.antigravity_tools/instances/inst-alpha\"",
        ),
    ];

    let report = scan_process_ancestry_with_table(500, &table, None);
    assert!(report.is_caller_ide_detected);
    assert_eq!(
        detect_caller_instance_id_from_ancestry(&report),
        Some("inst-alpha".to_string())
    );
    assert_eq!(report.ide_pid(), Some(480));
}

#[test]
fn test_detect_caller_terminates_on_cycle() {
    // Broken cyclic parentage
    let table = vec![
        ProcessNode::new(600, 601, "proc1.exe"),
        ProcessNode::new(601, 600, "proc2.exe"), // Cycle
    ];

    let report = scan_process_ancestry_with_table(600, &table, None);
    assert!(!report.is_caller_ide_detected);
    assert!(report.process_chain.len() <= 2);
}

#[test]
fn test_detect_caller_terminates_on_root_zero_ppid() {
    let table = vec![ProcessNode::new(700, 0, "init")];

    let report = scan_process_ancestry_with_table(700, &table, None);
    assert!(!report.is_caller_ide_detected);
    assert_eq!(report.process_chain.len(), 1);
}

#[test]
fn test_detect_caller_unknown_when_table_empty() {
    let table: Vec<ProcessNode> = Vec::new();

    let report = scan_process_ancestry_with_table(800, &table, None);
    assert!(!report.is_caller_ide_detected);
    assert_eq!(report.caller_origin, CallerIdeOrigin::Unknown);
    assert!(report.process_chain.is_empty());
}

#[test]
fn test_manager_binary_is_not_treated_as_ide() {
    // agm.exe (900) -> antigravity-manager.exe (890) -> explorer.exe (10)
    let table = vec![
        ProcessNode::new(900, 890, "agm.exe"),
        ProcessNode::new(890, 10, "antigravity-manager.exe"),
        ProcessNode::new(10, 0, "explorer.exe"),
    ];

    let report = scan_process_ancestry_with_table(900, &table, None);
    assert!(!report.is_caller_ide_detected);
    assert_eq!(report.caller_origin, CallerIdeOrigin::StandaloneTerminal);
    assert_eq!(detect_caller_instance_id_from_ancestry(&report), None);
}

#[test]
fn test_display_formatting() {
    let origin_ide = CallerIdeOrigin::AntigravityIdeTerminal {
        instance_id: "inst-1".to_string(),
        ide_pid: 1234,
    };
    let origin_term = CallerIdeOrigin::StandaloneTerminal;
    let origin_unk = CallerIdeOrigin::Unknown;

    assert!(format!("{origin_ide}").contains("inst-1"));
    assert!(format!("{origin_term}").contains("Standalone Terminal"));
    assert!(format!("{origin_unk}").contains("Unknown Process Origin"));

    let report = AncestryScanReport {
        process_chain: vec![],
        caller_origin: origin_ide,
        is_caller_ide_detected: true,
    };
    assert!(format!("{report}").contains("ide_detected=true"));
}
