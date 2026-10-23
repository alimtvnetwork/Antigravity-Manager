//! Unit tests for Multi-Stage Launcher Job Breakaway Spawner (Imp-628).

use crate::modules::instance::job_breakaway_process_spawner::{
    classify_target_kind, is_windows_batch_file, resolve_spawn_plan, ProcessSpawnDecision,
    SpawnFallbackLadderStage, SpawnerTargetKind, CREATE_BREAKAWAY_FROM_JOB, ERROR_ACCESS_DENIED,
};

#[test]
fn test_standard_binary_spawn_with_breakaway_enabled() {
    let args = vec!["--app-dir".to_string(), "C:\\app".to_string()];
    let decision = resolve_spawn_plan(
        "C:\\Program Files\\Antigravity\\antigravity.exe",
        &args,
        None,
        true,
        None,
    );

    assert_eq!(
        decision.executable_path,
        "C:\\Program Files\\Antigravity\\antigravity.exe"
    );
    assert_eq!(decision.args, args);
    assert_eq!(decision.creation_flags, CREATE_BREAKAWAY_FROM_JOB);
    assert_eq!(decision.stage, SpawnFallbackLadderStage::BreakawayAttempt);
    assert!(!decision.is_shell_wrapped);
    assert_eq!(decision.target_kind, SpawnerTargetKind::DirectBinary);
    assert!(decision.has_breakaway_flag());
}

#[test]
fn test_error_5_fallback_removes_breakaway_flag() {
    let args = vec!["--port".to_string(), "9000".to_string()];
    let decision = resolve_spawn_plan(
        "antigravity.exe",
        &args,
        None,
        true,
        Some(ERROR_ACCESS_DENIED),
    );

    assert_eq!(decision.executable_path, "antigravity.exe");
    assert_eq!(decision.creation_flags, 0);
    assert_eq!(decision.stage, SpawnFallbackLadderStage::NoBreakawayFallback);
    assert!(!decision.has_breakaway_flag());
    assert!(!decision.is_shell_wrapped);
    assert_eq!(decision.target_kind, SpawnerTargetKind::DirectBinary);
}

#[test]
fn test_batch_file_fallback_to_base_binary() {
    let args = vec!["--profile".to_string(), "p1".to_string()];
    let detected_base = "C:\\Base\\antigravity.exe";

    let decision_bat = resolve_spawn_plan(
        "C:\\Scripts\\launcher.bat",
        &args,
        Some(detected_base),
        true,
        None,
    );
    assert_eq!(decision_bat.executable_path, detected_base);
    assert_eq!(decision_bat.args, args);
    assert_eq!(decision_bat.creation_flags, CREATE_BREAKAWAY_FROM_JOB);
    assert_eq!(decision_bat.stage, SpawnFallbackLadderStage::BreakawayAttempt);
    assert!(!decision_bat.is_shell_wrapped);
    assert_eq!(decision_bat.target_kind, SpawnerTargetKind::DirectBinary);

    let decision_cmd = resolve_spawn_plan(
        "C:\\Scripts\\launcher.cmd",
        &args,
        Some(detected_base),
        true,
        None,
    );
    assert_eq!(decision_cmd.executable_path, detected_base);
    assert_eq!(decision_cmd.target_kind, SpawnerTargetKind::DirectBinary);
}

#[test]
fn test_batch_file_fallback_to_windows_shell_dispatch() {
    let args = vec!["--headless".to_string(), "--verbose".to_string()];
    let decision = resolve_spawn_plan("C:\\tools\\run.bat", &args, None, true, None);

    assert_eq!(decision.executable_path, "cmd.exe");
    assert_eq!(decision.creation_flags, 0);
    assert_eq!(
        decision.stage,
        SpawnFallbackLadderStage::ShellDispatchFallback
    );
    assert!(decision.is_shell_wrapped);
    assert_eq!(decision.target_kind, SpawnerTargetKind::ShellFallback);

    assert_eq!(decision.args[0], "/c");
    assert_eq!(decision.args[1], "start");
    assert_eq!(decision.args[2], "C:\\tools\\run.bat");
    assert_eq!(decision.args[3], "--headless");
    assert_eq!(decision.args[4], "--verbose");
}

#[test]
fn test_cmd_file_fallback_to_windows_shell_dispatch() {
    let args = vec!["arg1".to_string()];
    let decision = resolve_spawn_plan("test.cmd", &args, None, false, None);

    assert_eq!(decision.executable_path, "cmd.exe");
    assert_eq!(
        decision.stage,
        SpawnFallbackLadderStage::ShellDispatchFallback
    );
    assert!(decision.is_shell_wrapped);
    assert_eq!(decision.target_kind, SpawnerTargetKind::ShellFallback);
    assert_eq!(decision.args, vec!["/c", "start", "test.cmd", "arg1"]);
}

#[test]
fn test_non_batch_binary_preserves_direct_invocation() {
    let args = vec!["--check".to_string()];
    let decision = resolve_spawn_plan("custom_binary.exe", &args, None, false, None);

    assert_eq!(decision.executable_path, "custom_binary.exe");
    assert_eq!(decision.args, args);
    assert_eq!(decision.creation_flags, 0);
    assert_eq!(decision.stage, SpawnFallbackLadderStage::NoBreakawayFallback);
    assert!(!decision.is_shell_wrapped);
    assert_eq!(decision.target_kind, SpawnerTargetKind::DirectBinary);
}

#[test]
fn test_batch_file_detection() {
    assert!(is_windows_batch_file("run.bat"));
    assert!(is_windows_batch_file("RUN.BAT"));
    assert!(is_windows_batch_file("run.cmd"));
    assert!(is_windows_batch_file("RUN.CMD"));
    assert!(is_windows_batch_file("C:\\path\\to\\file.bat"));
    assert!(is_windows_batch_file("  spaced.cmd  "));

    assert!(!is_windows_batch_file("run.exe"));
    assert!(!is_windows_batch_file("run.sh"));
    assert!(!is_windows_batch_file("run.ps1"));
    assert!(!is_windows_batch_file("bat.txt"));
}

#[test]
fn test_classify_target_kind() {
    assert_eq!(
        classify_target_kind("launcher.bat"),
        SpawnerTargetKind::WindowsBatchScript
    );
    assert_eq!(
        classify_target_kind("run.cmd"),
        SpawnerTargetKind::WindowsBatchScript
    );
    assert_eq!(
        classify_target_kind("antigravity.exe"),
        SpawnerTargetKind::DirectBinary
    );
    assert_eq!(
        classify_target_kind("cmd.exe"),
        SpawnerTargetKind::ShellFallback
    );
    assert_eq!(
        classify_target_kind("C:\\Windows\\System32\\cmd.exe"),
        SpawnerTargetKind::ShellFallback
    );
}

#[test]
fn test_access_denied_with_base_binary_clears_breakaway() {
    let args = vec![];
    let detected_base = "C:\\bin\\app.exe";
    let decision = resolve_spawn_plan(
        "launch.bat",
        &args,
        Some(detected_base),
        true,
        Some(ERROR_ACCESS_DENIED),
    );

    assert_eq!(decision.executable_path, detected_base);
    assert_eq!(decision.creation_flags, 0);
    assert_eq!(decision.stage, SpawnFallbackLadderStage::NoBreakawayFallback);
    assert_eq!(decision.target_kind, SpawnerTargetKind::DirectBinary);
}

#[test]
fn test_serde_roundtrip() {
    let decision = ProcessSpawnDecision {
        executable_path: "cmd.exe".to_string(),
        args: vec!["/c".to_string(), "start".to_string(), "run.bat".to_string()],
        creation_flags: 0,
        stage: SpawnFallbackLadderStage::ShellDispatchFallback,
        is_shell_wrapped: true,
        target_kind: SpawnerTargetKind::ShellFallback,
    };

    let serialized = serde_json::to_string(&decision).expect("serialize decision");
    let deserialized: ProcessSpawnDecision =
        serde_json::from_str(&serialized).expect("deserialize decision");
    assert_eq!(decision, deserialized);

    let stage = SpawnFallbackLadderStage::BreakawayAttempt;
    let s_stage = serde_json::to_string(&stage).expect("serialize stage");
    let d_stage: SpawnFallbackLadderStage =
        serde_json::from_str(&s_stage).expect("deserialize stage");
    assert_eq!(stage, d_stage);

    let kind = SpawnerTargetKind::WindowsBatchScript;
    let s_kind = serde_json::to_string(&kind).expect("serialize kind");
    let d_kind: SpawnerTargetKind =
        serde_json::from_str(&s_kind).expect("deserialize kind");
    assert_eq!(kind, d_kind);
}
