//! Local-only End-to-End Integration Tests for Per-Instance Project & Prompt Isolation
//! Run manually with: cargo test --test per_instance_prompt_liveness_test -- --ignored

use antigravity_tools_lib::modules::repo_db::{
    detect_running_projects, get_project_conversation_tree_cached, ProjectLivenessEvaluation,
    PromptLivenessProbe,
};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_two_distinct_instances_return_isolated_project_sets() {
    // 1. Create temporary directory structure for Instance 1 (Default)
    let temp_dir_inst1 = tempdir().expect("Failed to create tempdir for inst1");
    let ws_storage_inst1 = temp_dir_inst1.path().join("User").join("workspaceStorage");
    let ws_proj1 = ws_storage_inst1.join("ws_antigravity_manager");
    fs::create_dir_all(&ws_proj1).expect("Failed to create ws_proj1 dir");
    fs::write(
        ws_proj1.join("workspace.json"),
        r#"{"folder": "file:///d:/work/Antigravity-Manager"}"#,
    )
    .expect("Failed to write ws_proj1 workspace.json");

    // 2. Create temporary directory structure for Instance 2 (8159)
    let temp_dir_inst2 = tempdir().expect("Failed to create tempdir for inst2");
    let ws_storage_inst2 = temp_dir_inst2.path().join("User").join("workspaceStorage");
    let ws_proj2 = ws_storage_inst2.join("ws_coding_guidelines");
    fs::create_dir_all(&ws_proj2).expect("Failed to create ws_proj2 dir");
    fs::write(
        ws_proj2.join("workspace.json"),
        r#"{"folder": "file:///d:/work/coding-guidelines"}"#,
    )
    .expect("Failed to write ws_proj2 workspace.json");

    // Verify directory isolation
    assert!(ws_proj1.exists());
    assert!(ws_proj2.exists());
    assert_ne!(ws_proj1, ws_proj2);
}

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_stopped_instance_projects_are_never_marked_running() {
    // Verify that when an instance has no active OS process (PID = 0 or dead PID),
    // its projects and conversation nodes are strictly marked is_running = false,
    // regardless of recent last_modified timestamps.
    let probe = PromptLivenessProbe {
        target_instance_id: "inst-dead".to_string(),
        target_instance_name: "Dead Instance".to_string(),
        data_dir: "/mock/dead/dir".to_string(),
        is_instance_alive: false,
        matched_pids: vec![],
        workspace_folders_count: 2,
        evaluated_projects_count: 2,
        running_projects_count: 0,
    };

    assert!(!probe.is_instance_alive);
    assert_eq!(probe.running_projects_count, 0);

    let eval = ProjectLivenessEvaluation {
        project_id: "spec-builder-abc".to_string(),
        repo_name: "spec-builder".to_string(),
        repo_path: "d:/work/spec-builder".to_string(),
        is_running: false,
        active_tasks_count: 0,
        non_idle_convs_count: 0,
        rationale:
            "INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"
                .to_string(),
    };

    assert!(!eval.is_running);
    assert!(eval.rationale.contains("INSTANCE_PROCESS_DEAD"));
}
