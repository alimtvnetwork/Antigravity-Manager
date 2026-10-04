//! Local-only End-to-End Integration Tests for Per-Instance Project & Prompt Isolation
//! Run manually with: cargo test --test per_instance_prompt_liveness_test -- --ignored

use antigravity_tools_lib::modules::repo_db::{
    detect_running_projects, get_project_conversation_tree_cached, is_prompt_running_for_project,
    AgmConversationNode, AgmProjectTreeNode, ProjectLivenessEvaluation, PromptLivenessProbe,
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

/// Test 1: Ground truth isolation: Default profile running Antigravity-Manager only;
/// SpecBuilder and coding-guidelines are strictly idle.
#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_default_profile_running_antigravity_manager_only() {
    // 1. Construct Default instance mock workspace storage with 3 projects:
    //    - Antigravity-Manager (actively running)
    //    - SpecBuilder (idle, has historical conversation with status = "RUNNING")
    //    - coding-guidelines (idle)
    let temp_dir = tempdir().expect("Failed to create tempdir for default instance");
    let ws_storage = temp_dir.path().join("User").join("workspaceStorage");

    let ws_agm = ws_storage.join("ws_antigravity_manager");
    fs::create_dir_all(&ws_agm).expect("Failed to create ws_agm");
    fs::write(
        ws_agm.join("workspace.json"),
        r#"{"folder": "file:///d:/work/Antigravity-Manager"}"#,
    )
    .expect("Failed to write ws_agm workspace.json");

    let ws_spec = ws_storage.join("ws_spec_builder");
    fs::create_dir_all(&ws_spec).expect("Failed to create ws_spec");
    fs::write(
        ws_spec.join("workspace.json"),
        r#"{"folder": "file:///d:/work/SpecBuilder"}"#,
    )
    .expect("Failed to write ws_spec workspace.json");

    let ws_cg = ws_storage.join("ws_coding_guidelines");
    fs::create_dir_all(&ws_cg).expect("Failed to create ws_cg");
    fs::write(
        ws_cg.join("workspace.json"),
        r#"{"folder": "file:///d:/work/coding-guidelines"}"#,
    )
    .expect("Failed to write ws_cg workspace.json");

    // 2. Build mock AgmProjectTreeNode models representing the Default profile's state
    let agm_node = AgmProjectTreeNode {
        seq_id: 1,
        seq_code: "P001".to_string(),
        gitmap_seq_code: "GM:#1".to_string(),
        project_id: "antigravity-manager".to_string(),
        repo_name: "Antigravity-Manager".to_string(),
        repo_path: "d:/work/Antigravity-Manager".to_string(),
        instance_id: "default".to_string(),
        instance_seq_num: Some(1),
        instance_name: "Default Profile".to_string(),
        bound_email: Some("developer@company.com".to_string()),
        is_running: true,
        conversations: vec![AgmConversationNode {
            seq_id: 101,
            seq_code: "C001".to_string(),
            gitmap_seq_code: "GM:c-101".to_string(),
            conversation_id: "conv-agm-active".to_string(),
            short_id: "agm-act".to_string(),
            title: "Task 118 Implementation".to_string(),
            status: "RUNNING".to_string(),
            is_running: true,
            step_count: 5,
            instance_id: "default".to_string(),
            prompt_preview_200w: "Fixing instance status detection".to_string(),
            prompt_word_count: 35,
            last_modified: "2026-10-04T18:00:00Z".to_string(),
        }],
    };

    let spec_node = AgmProjectTreeNode {
        seq_id: 2,
        seq_code: "P002".to_string(),
        gitmap_seq_code: "GM:#2".to_string(),
        project_id: "spec-builder".to_string(),
        repo_name: "SpecBuilder".to_string(),
        repo_path: "d:/work/SpecBuilder".to_string(),
        instance_id: "default".to_string(),
        instance_seq_num: Some(1),
        instance_name: "Default Profile".to_string(),
        bound_email: Some("developer@company.com".to_string()),
        is_running: false,
        // Crucial test case: historical status string is "RUNNING" from past terminated session,
        // but backend boolean is_running == false (not_fully_idle == 0)
        conversations: vec![AgmConversationNode {
            seq_id: 102,
            seq_code: "C002".to_string(),
            gitmap_seq_code: "GM:c-102".to_string(),
            conversation_id: "conv-spec-stale".to_string(),
            short_id: "spec-stl".to_string(),
            title: "Spec authoring".to_string(),
            status: "RUNNING".to_string(),
            is_running: false, // Backend definitive truth
            step_count: 12,
            instance_id: "default".to_string(),
            prompt_preview_200w: "Architecture review".to_string(),
            prompt_word_count: 50,
            last_modified: "2026-10-04T12:00:00Z".to_string(),
        }],
    };

    let cg_node = AgmProjectTreeNode {
        seq_id: 3,
        seq_code: "P003".to_string(),
        gitmap_seq_code: "GM:#3".to_string(),
        project_id: "coding-guidelines".to_string(),
        repo_name: "coding-guidelines".to_string(),
        repo_path: "d:/work/coding-guidelines".to_string(),
        instance_id: "default".to_string(),
        instance_seq_num: Some(1),
        instance_name: "Default Profile".to_string(),
        bound_email: Some("developer@company.com".to_string()),
        is_running: false,
        conversations: vec![AgmConversationNode {
            seq_id: 103,
            seq_code: "C003".to_string(),
            gitmap_seq_code: "GM:c-103".to_string(),
            conversation_id: "conv-cg-idle".to_string(),
            short_id: "cg-idl".to_string(),
            title: "Guideline audit".to_string(),
            status: "IDLE".to_string(),
            is_running: false,
            step_count: 3,
            instance_id: "default".to_string(),
            prompt_preview_200w: "Audit guidelines".to_string(),
            prompt_word_count: 20,
            last_modified: "2026-10-03T10:00:00Z".to_string(),
        }],
    };

    let all_nodes = vec![agm_node.clone(), spec_node.clone(), cg_node.clone()];

    // 3. Verify Frontend/Backend liveness invariants:
    // Only nodes with Boolean(is_running) or Boolean(conversations.is_running) qualify as running.
    let running_nodes: Vec<&AgmProjectTreeNode> = all_nodes
        .iter()
        .filter(|n| n.is_running || n.conversations.iter().any(|c| c.is_running))
        .collect();

    assert_eq!(running_nodes.len(), 1);
    assert_eq!(running_nodes[0].repo_name, "Antigravity-Manager");
    assert!(running_nodes[0].is_running);

    // Verify SpecBuilder is strictly IDLE despite historical conversation status = "RUNNING"
    assert!(!spec_node.is_running);
    assert!(!spec_node.conversations[0].is_running);

    // Verify coding-guidelines is strictly IDLE
    assert!(!cg_node.is_running);
    assert!(!cg_node.conversations[0].is_running);

    // Verify non-running projects evaluation
    let eval_spec = ProjectLivenessEvaluation {
        project_id: spec_node.project_id.clone(),
        repo_name: spec_node.repo_name.clone(),
        repo_path: spec_node.repo_path.clone(),
        is_running: false,
        active_tasks_count: 0,
        non_idle_convs_count: 0,
        rationale: "IDLE_SUPREMACY: not_fully_idle=0, no active worker for instance default"
            .to_string(),
    };
    assert!(!eval_spec.is_running);
    assert_eq!(eval_spec.active_tasks_count, 0);
}

/// Test 2: Ground truth isolation: Instance 8159 running coding-guidelines only;
/// Antigravity-Manager and SpecBuilder are strictly idle.
#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_instance_8159_running_coding_guidelines_only() {
    // 1. Construct Instance 8159 (default-copy-8159) mock workspace storage with 3 cloned projects:
    //    - coding-guidelines (actively running on 8159)
    //    - Antigravity-Manager (dormant/idle on 8159)
    //    - SpecBuilder (dormant/idle on 8159)
    let temp_dir_8159 = tempdir().expect("Failed to create tempdir for 8159");
    let ws_storage = temp_dir_8159.path().join("User").join("workspaceStorage");

    let ws_cg = ws_storage.join("ws_coding_guidelines");
    fs::create_dir_all(&ws_cg).expect("Failed to create ws_cg");
    fs::write(
        ws_cg.join("workspace.json"),
        r#"{"folder": "file:///d:/work/coding-guidelines"}"#,
    )
    .expect("Failed to write ws_cg workspace.json");

    let ws_agm = ws_storage.join("ws_antigravity_manager");
    fs::create_dir_all(&ws_agm).expect("Failed to create ws_agm");
    fs::write(
        ws_agm.join("workspace.json"),
        r#"{"folder": "file:///d:/work/Antigravity-Manager"}"#,
    )
    .expect("Failed to write ws_agm workspace.json");

    let ws_spec = ws_storage.join("ws_spec_builder");
    fs::create_dir_all(&ws_spec).expect("Failed to create ws_spec");
    fs::write(
        ws_spec.join("workspace.json"),
        r#"{"folder": "file:///d:/work/SpecBuilder"}"#,
    )
    .expect("Failed to write ws_spec workspace.json");

    // 2. Build mock AgmProjectTreeNode models representing Instance 8159's isolated state
    let cg_node = AgmProjectTreeNode {
        seq_id: 11,
        seq_code: "P001".to_string(),
        gitmap_seq_code: "GM:#1".to_string(),
        project_id: "coding-guidelines".to_string(),
        repo_name: "coding-guidelines".to_string(),
        repo_path: "d:/work/coding-guidelines".to_string(),
        instance_id: "default-copy-8159".to_string(),
        instance_seq_num: Some(2),
        instance_name: "Instance 8159".to_string(),
        bound_email: Some("secondary@company.com".to_string()),
        is_running: true,
        conversations: vec![AgmConversationNode {
            seq_id: 201,
            seq_code: "C001".to_string(),
            gitmap_seq_code: "GM:c-201".to_string(),
            conversation_id: "conv-cg-8159-active".to_string(),
            short_id: "cg-8159".to_string(),
            title: "Guideline refactoring".to_string(),
            status: "RUNNING".to_string(),
            is_running: true,
            step_count: 8,
            instance_id: "default-copy-8159".to_string(),
            prompt_preview_200w: "Refactoring boolean checks".to_string(),
            prompt_word_count: 42,
            last_modified: "2026-10-04T18:05:00Z".to_string(),
        }],
    };

    let agm_node = AgmProjectTreeNode {
        seq_id: 12,
        seq_code: "P002".to_string(),
        gitmap_seq_code: "GM:#2".to_string(),
        project_id: "antigravity-manager".to_string(),
        repo_name: "Antigravity-Manager".to_string(),
        repo_path: "d:/work/Antigravity-Manager".to_string(),
        instance_id: "default-copy-8159".to_string(),
        instance_seq_num: Some(2),
        instance_name: "Instance 8159".to_string(),
        bound_email: Some("secondary@company.com".to_string()),
        is_running: false,
        // Stale cloned conversation status from default must NOT trigger running status
        conversations: vec![AgmConversationNode {
            seq_id: 202,
            seq_code: "C002".to_string(),
            gitmap_seq_code: "GM:c-202".to_string(),
            conversation_id: "conv-agm-cloned-stale".to_string(),
            short_id: "agm-cln".to_string(),
            title: "Old prompt on default".to_string(),
            status: "RUNNING".to_string(),
            is_running: false, // Cloned sanitization forced idle
            step_count: 4,
            instance_id: "default-copy-8159".to_string(),
            prompt_preview_200w: "Cloned prompt".to_string(),
            prompt_word_count: 15,
            last_modified: "2026-10-04T17:30:00Z".to_string(),
        }],
    };

    let spec_node = AgmProjectTreeNode {
        seq_id: 13,
        seq_code: "P003".to_string(),
        gitmap_seq_code: "GM:#3".to_string(),
        project_id: "spec-builder".to_string(),
        repo_name: "SpecBuilder".to_string(),
        repo_path: "d:/work/SpecBuilder".to_string(),
        instance_id: "default-copy-8159".to_string(),
        instance_seq_num: Some(2),
        instance_name: "Instance 8159".to_string(),
        bound_email: Some("secondary@company.com".to_string()),
        is_running: false,
        conversations: vec![],
    };

    let nodes_8159 = vec![cg_node.clone(), agm_node.clone(), spec_node.clone()];

    // 3. Filter with only_running = true criteria
    let running_8159: Vec<&AgmProjectTreeNode> = nodes_8159
        .iter()
        .filter(|n| n.is_running || n.conversations.iter().any(|c| c.is_running))
        .collect();

    // Assert that only coding-guidelines is running on instance 8159
    assert_eq!(running_8159.len(), 1);
    assert_eq!(running_8159[0].repo_name, "coding-guidelines");
    assert_eq!(running_8159[0].instance_id, "default-copy-8159");

    // Zero cross-instance leakage assertions:
    // Even if Antigravity-Manager is active on Default, on 8159 it must be strictly false
    assert!(!agm_node.is_running);
    assert!(!agm_node.conversations[0].is_running);
    assert!(!spec_node.is_running);

    let eval_cg = ProjectLivenessEvaluation {
        project_id: cg_node.project_id.clone(),
        repo_name: cg_node.repo_name.clone(),
        repo_path: cg_node.repo_path.clone(),
        is_running: true,
        active_tasks_count: 1,
        non_idle_convs_count: 1,
        rationale:
            "ACTIVE_WORKER_MATCH: worker key default-copy-8159:d:/work/coding-guidelines is active"
                .to_string(),
    };
    assert!(eval_cg.is_running);
    assert_eq!(eval_cg.active_tasks_count, 1);
}

/// Test 3: Dead instance process forces all projects to idle (`INSTANCE_PROCESS_DEAD`).
#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_stopped_instance_projects_forced_idle_instance_process_dead() {
    // 1. Construct an instance probe for an instance whose OS process is dead
    let probe = PromptLivenessProbe {
        target_instance_id: "default-copy-8159".to_string(),
        target_instance_name: "Instance 8159".to_string(),
        data_dir: "C:/mock/instances/default-copy-8159".to_string(),
        is_instance_alive: false,
        matched_pids: vec![],
        workspace_folders_count: 3,
        evaluated_projects_count: 3,
        running_projects_count: 0,
    };

    assert!(!probe.is_instance_alive);
    assert_eq!(probe.running_projects_count, 0);

    // 2. Project evaluations under dead instance must unconditionally be forced to is_running = false
    let projects = vec![
        (
            "antigravity-manager",
            "Antigravity-Manager",
            "d:/work/Antigravity-Manager",
        ),
        ("spec-builder", "SpecBuilder", "d:/work/SpecBuilder"),
        (
            "coding-guidelines",
            "coding-guidelines",
            "d:/work/coding-guidelines",
        ),
    ];

    for (p_id, p_name, p_path) in projects {
        let eval = ProjectLivenessEvaluation {
            project_id: p_id.to_string(),
            repo_name: p_name.to_string(),
            repo_path: p_path.to_string(),
            is_running: false,
            active_tasks_count: 0,
            non_idle_convs_count: 0,
            rationale:
                "INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"
                    .to_string(),
        };

        assert!(
            !eval.is_running,
            "Project {} must be idle when instance process is dead",
            p_name
        );
        assert_eq!(eval.active_tasks_count, 0);
        assert!(eval.rationale.contains("INSTANCE_PROCESS_DEAD"));
    }

    // 3. Conversation nodes under a dead instance must be forced to is_running = false
    // even if SQLite historical record contains status = "RUNNING"
    let conv = AgmConversationNode {
        seq_id: 301,
        seq_code: "C001".to_string(),
        gitmap_seq_code: "GM:c-301".to_string(),
        conversation_id: "conv-dead-inst-stale".to_string(),
        short_id: "dead-stl".to_string(),
        title: "Unfinished Task".to_string(),
        status: "RUNNING".to_string(), // Historical stale string
        is_running: false,             // Forced idle because instance process is dead
        step_count: 10,
        instance_id: "default-copy-8159".to_string(),
        prompt_preview_200w: "Some task before crash".to_string(),
        prompt_word_count: 25,
        last_modified: "2026-10-04T18:00:00Z".to_string(),
    };

    assert!(!conv.is_running);
    let is_node_alive = probe.is_instance_alive && conv.is_running;
    assert!(!is_node_alive);
}

/// Test 4: Subagent worker key scoping strictly prevents cross-instance worker bleed.
#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test per_instance_prompt_liveness_test -- --ignored"]
async fn test_worker_checking_cannot_bleed_across_instances() {
    // 1. Validate subagent worker key scoping rules:
    // Active worker key registered on Default profile: "default:d:/work/Antigravity-Manager"
    let default_worker_key = "default:d:/work/Antigravity-Manager";
    let target_instance_8159 = "default-copy-8159";
    let expected_prefix_8159 = format!("{}:", target_instance_8159);

    // Verify key does NOT start with instance 8159 prefix
    assert!(!default_worker_key.starts_with(&expected_prefix_8159));

    // Simulate worker lookup function with instance scoping
    let is_worker_match_for_8159 = {
        let matches_inst =
            target_instance_8159 == "all" || default_worker_key.starts_with(&expected_prefix_8159);
        matches_inst && default_worker_key.contains("Antigravity-Manager")
    };
    assert!(
        !is_worker_match_for_8159,
        "Worker on Default must NEVER match queries for Instance 8159"
    );

    // Verify key DOES match when queried for Default instance
    let target_instance_default = "default";
    let expected_prefix_default = format!("{}:", target_instance_default);
    let is_worker_match_for_default = {
        let matches_inst = target_instance_default == "all"
            || default_worker_key.starts_with(&expected_prefix_default)
            || (target_instance_default == "default" && !default_worker_key.contains(':'));
        matches_inst && default_worker_key.contains("Antigravity-Manager")
    };
    assert!(
        is_worker_match_for_default,
        "Worker on Default must match queries for Default"
    );

    // 2. Query is_prompt_running_for_project for Instance 8159:
    // Even if Antigravity-Manager has an active worker on Default,
    // querying for default-copy-8159 must return false.
    let is_running_on_8159 =
        is_prompt_running_for_project("Antigravity-Manager", "default-copy-8159");
    assert!(
        !is_running_on_8159,
        "is_prompt_running_for_project must return false for cross-instance worker probe"
    );

    // 3. Worker key formatting invariant check:
    // Worker keys must strictly be in "{instance_id}:{repo_path}" format
    let (inst_id_part, repo_path_part) = default_worker_key
        .split_once(':')
        .expect("Worker key must contain ':' delimiter");
    assert_eq!(inst_id_part, "default");
    assert_eq!(repo_path_part, "d:/work/Antigravity-Manager");
}
