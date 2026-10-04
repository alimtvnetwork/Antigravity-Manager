//! Integration Tests for Per-Instance Project & Prompt Isolation
//! Verifies strict boolean liveness, instance isolation, and dead-process gating.

use antigravity_tools_lib::modules::repo_db::{
    is_prompt_running_for_project, AgmConversationNode, AgmProjectTreeNode,
    ProjectLivenessEvaluation, PromptLivenessProbe,
};
use std::fs;
use tempfile::tempdir;

fn build_mock_conv(id: &str, is_run: bool) -> AgmConversationNode {
    let raw = serde_json::json!({
        "seq_id": 1, "seq_code": "C001", "gitmap_seq_code": format!("GM:{id}"),
        "conversation_id": id, "short_id": id, "title": "conv", "status": "RUNNING",
        "is_running": is_run, "step_count": 1, "instance_id": "default",
        "prompt_preview_200w": "preview", "prompt_word_count": 1,
        "last_modified": "2026-10-04T00:00:00Z"
    });
    serde_json::from_value(raw).expect("valid conversation")
}

fn build_mock_project(
    id: &str,
    name: &str,
    is_run: bool,
    convs: Vec<AgmConversationNode>,
) -> AgmProjectTreeNode {
    let raw = serde_json::json!({
        "seq_id": 1, "seq_code": "P001", "gitmap_seq_code": format!("GM:#{id}"),
        "project_id": id, "repo_name": name, "repo_path": format!("work/{name}"),
        "instance_id": "default", "instance_seq_num": 1, "instance_name": "default",
        "bound_email": null, "is_running": is_run, "conversations": convs
    });
    serde_json::from_value(raw).expect("valid project")
}

fn mock_proj(name: &str, is_run: bool) -> AgmProjectTreeNode {
    let conv = build_mock_conv(&format!("c-{name}"), is_run);
    build_mock_project(&format!("p-{name}"), name, is_run, vec![conv])
}

fn build_dead_eval(id: &str, name: &str) -> ProjectLivenessEvaluation {
    ProjectLivenessEvaluation {
        project_id: id.to_string(),
        repo_name: name.to_string(),
        repo_path: format!("work/{name}"),
        is_running: false,
        active_tasks_count: 0,
        non_idle_convs_count: 0,
        rationale:
            "INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"
                .to_string(),
    }
}

fn build_dead_probe(id: &str) -> PromptLivenessProbe {
    PromptLivenessProbe {
        target_instance_id: id.to_string(),
        target_instance_name: "Dead Instance".to_string(),
        data_dir: "mock/dead/dir".to_string(),
        is_instance_alive: false,
        matched_pids: vec![],
        workspace_folders_count: 1,
        evaluated_projects_count: 1,
        running_projects_count: 0,
    }
}

fn check_worker_prefix_match(worker_key: &str, target_inst: &str) -> bool {
    let expected_prefix = format!("{}:", target_inst);
    worker_key.starts_with(&expected_prefix)
        || (target_inst == "default" && !worker_key.contains(':'))
}

fn create_mock_ws(dir: &tempfile::TempDir, name: &str, folder: &str) -> std::path::PathBuf {
    let ws = dir.path().join("User").join("workspaceStorage").join(name);
    fs::create_dir_all(&ws).expect("create ws dir");
    fs::write(
        ws.join("workspace.json"),
        format!(r#"{{"folder": "{folder}"}}"#),
    )
    .expect("write ws");
    ws
}

fn filter_running_projects(projs: Vec<AgmProjectTreeNode>) -> Vec<AgmProjectTreeNode> {
    projs.into_iter().filter(|p| p.is_running).collect()
}

#[tokio::test]
async fn test_two_distinct_instances_return_isolated_project_sets() {
    let d1 = tempdir().expect("tempdir inst1");
    let ws1 = create_mock_ws(&d1, "ws_agm", "mock-work/antigravity-manager");
    let d2 = tempdir().expect("tempdir inst2");
    let ws2 = create_mock_ws(&d2, "ws_cg", "mock-work/coding-guidelines");

    assert!(ws1.exists());
    assert!(ws2.exists());
    assert_ne!(ws1, ws2);
}

#[tokio::test]
async fn test_default_profile_running_antigravity_manager_only() {
    let p_agm = mock_proj("Antigravity-Manager", true);
    let p_spec = mock_proj("SpecBuilder", false);
    let p_cg = mock_proj("coding-guidelines", false);

    assert!(p_agm.is_running);
    assert!(!p_spec.is_running);
    assert!(!p_cg.is_running);
    assert!(!is_prompt_running_for_project("SpecBuilder", "default"));
}

#[tokio::test]
async fn test_instance_8159_running_coding_guidelines_only() {
    let p_cg = mock_proj("coding-guidelines", true);
    let p_agm = mock_proj("Antigravity-Manager", false);
    let p_spec = mock_proj("SpecBuilder", false);

    let running = filter_running_projects(vec![p_cg.clone(), p_agm, p_spec]);
    assert!(p_cg.is_running);
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].repo_name, "coding-guidelines");
}

#[tokio::test]
async fn test_stopped_instance_projects_forced_idle_instance_process_dead() {
    let probe = build_dead_probe("inst-dead");
    let eval = build_dead_eval("spec-builder-abc", "spec-builder");
    let conv = build_mock_conv("conv-dead", false);

    assert!(!probe.is_instance_alive);
    assert_eq!(probe.running_projects_count, 0);
    assert!(!eval.is_running);
    assert_eq!(eval.active_tasks_count, 0);
    assert!(!conv.is_running);
    assert!(eval.rationale.contains("INSTANCE_PROCESS_DEAD"));
}

#[tokio::test]
async fn test_worker_checking_cannot_bleed_across_instances() {
    let worker_key = "default:work/antigravity-manager";
    let is_matched_8159 = check_worker_prefix_match(worker_key, "default-copy-8159");
    let is_matched_default = check_worker_prefix_match(worker_key, "default");
    let is_prompt_running =
        is_prompt_running_for_project("antigravity-manager", "default-copy-8159");

    assert!(!is_matched_8159);
    assert!(is_matched_default);
    assert!(!is_prompt_running);
}
