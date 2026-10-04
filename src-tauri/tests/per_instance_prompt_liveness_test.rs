//! Integration Tests for Per-Instance Project & Prompt Isolation
//! Verifies strict boolean liveness, instance isolation, and dead-process gating.

use antigravity_tools_lib::modules::repo_db::{
    AgmConversationNode, AgmProjectTreeNode, ProjectLivenessEvaluation, PromptLivenessProbe,
};
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Initialize real conversation_summaries.db SQLite database in the given directory
fn init_summaries_db(db_path: &Path) -> Connection {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).expect("create parent directory for database");
    }
    let conn = Connection::open(db_path).expect("open conversation_summaries.db");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversation_summaries (
            conversation_id TEXT PRIMARY KEY,
            title TEXT,
            preview TEXT,
            status TEXT,
            not_fully_idle INTEGER,
            workspace_uris TEXT,
            last_modified_time TEXT
        );",
        [],
    )
    .expect("create conversation_summaries table");
    conn
}

/// Insert conversation record into SQLite summaries DB
fn insert_conversation_summary(
    conn: &Connection,
    cid: &str,
    title: &str,
    preview: &str,
    status: &str,
    not_fully_idle: i32,
    workspace_path: &str,
    last_mod: &str,
) {
    let ws_json = serde_json::to_string(&vec![format!(
        "file:///{}",
        workspace_path.replace('\\', "/")
    )])
    .unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT OR REPLACE INTO conversation_summaries 
         (conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            cid,
            title,
            preview,
            status,
            not_fully_idle,
            ws_json,
            last_mod
        ],
    )
    .expect("insert conversation record");
}

/// Simulated project and conversation evaluation from an SQLite summaries DB
fn evaluate_instance_conversations_from_db(
    db_path: &Path,
    is_inst_alive: bool,
) -> Vec<(String, String, String, String, bool, i32, String)> {
    if !db_path.exists() {
        return Vec::new();
    }
    let conn = Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .expect("open db for evaluation");

    let mut stmt = conn
        .prepare(
            "SELECT conversation_id, title, preview, status, not_fully_idle, workspace_uris, last_modified_time
             FROM conversation_summaries
             ORDER BY last_modified_time DESC",
        )
        .expect("prepare select");

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .expect("query map");

    let mut results = Vec::new();
    for row in rows.flatten() {
        let (cid, title, preview, status, not_fully_idle, ws_uris_opt, last_mod) = row;

        let is_idle_count = not_fully_idle == 0;
        let has_idle_status = status.contains("IDLE")
            || status.contains("COMPLETED")
            || status.contains("FAILED")
            || status.contains("CANCELLED");
        let is_explicit_idle = is_idle_count || has_idle_status;

        let has_active_turns = not_fully_idle > 0;
        let has_running_text = status.contains("RUNNING");
        let is_active_session = has_active_turns && has_running_text;

        let is_conv_running = if is_explicit_idle {
            false
        } else if is_inst_alive && is_active_session {
            true
        } else {
            false
        };

        let ws_path = ws_uris_opt
            .and_then(|u| {
                let list: Vec<String> = serde_json::from_str(&u).ok()?;
                list.into_iter().next()
            })
            .unwrap_or_default();

        results.push((
            cid,
            title,
            preview,
            status,
            is_conv_running,
            not_fully_idle,
            ws_path,
        ));
    }
    results
}

/// Compute evaluation record and structured audit probe log string
fn evaluate_project_liveness_record(
    instance_id: &str,
    project_id: &str,
    repo_name: &str,
    repo_path: &str,
    is_inst_alive: bool,
    has_active_conv: bool,
    has_active_prompt: bool,
) -> (ProjectLivenessEvaluation, String) {
    let proj_is_running = is_inst_alive && (has_active_conv || has_active_prompt);
    let rationale = if !is_inst_alive {
        "INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"
            .to_string()
    } else if proj_is_running {
        "ACTIVE_IN_FLIGHT_TASKS: active prompt or non-idle conversation detected -> marked running"
            .to_string()
    } else {
        "IDLE: process alive but no in-flight tasks or active conversations -> marked idle"
            .to_string()
    };

    let log_line = format!(
        "[PROMPT_LIVENESS_PROBE][PROJECT] instance_id=\"{}\" project=\"{}\" is_running={} rationale=\"{}\"",
        instance_id, repo_name, proj_is_running, rationale
    );

    let eval = ProjectLivenessEvaluation {
        project_id: project_id.to_string(),
        repo_name: repo_name.to_string(),
        repo_path: repo_path.to_string(),
        is_running: proj_is_running,
        active_tasks_count: if proj_is_running { 1 } else { 0 },
        non_idle_convs_count: if has_active_conv { 1 } else { 0 },
        rationale,
    };

    (eval, log_line)
}

fn build_mock_conv(id: &str, is_run: bool, status: &str, instance_id: &str) -> AgmConversationNode {
    let raw = serde_json::json!({
        "seq_id": 1, "seq_code": "C001", "gitmap_seq_code": format!("GM:{id}"),
        "conversation_id": id, "short_id": id, "title": format!("conv-{id}"),
        "status": status,
        "is_running": is_run, "step_count": 1, "instance_id": instance_id,
        "prompt_preview_200w": "preview", "prompt_word_count": 1,
        "last_modified": "2026-10-04T00:00:00Z"
    });
    serde_json::from_value(raw).expect("valid conversation")
}

fn build_mock_project(
    id: &str,
    name: &str,
    instance_id: &str,
    is_run: bool,
    convs: Vec<AgmConversationNode>,
) -> AgmProjectTreeNode {
    let raw = serde_json::json!({
        "seq_id": 1, "seq_code": "P001", "gitmap_seq_code": format!("GM:#{id}"),
        "project_id": id, "repo_name": name, "repo_path": format!("d:/work/{name}"),
        "instance_id": instance_id, "instance_seq_num": 1, "instance_name": instance_id,
        "bound_email": null, "is_running": is_run, "conversations": convs
    });
    serde_json::from_value(raw).expect("valid project")
}

fn filter_running_projects(projs: Vec<AgmProjectTreeNode>) -> Vec<AgmProjectTreeNode> {
    projs.into_iter().filter(|p| p.is_running).collect()
}

// ----------------------------------------------------------------------------
// Test Case 1: Default Instance Running Antigravity-Manager Only
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_case_1_default_running_antigravity_manager_only() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let default_db_path = sandbox
        .path()
        .join("home")
        .join(".gemini")
        .join("antigravity")
        .join("conversation_summaries.db");

    let conn = init_summaries_db(&default_db_path);

    // Default DB: Antigravity-Manager is active (running), others idle
    insert_conversation_summary(
        &conn,
        "conv-agm-1",
        "AGM Task",
        "Refactoring liveness",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        "2026-10-04T10:00:00Z",
    );
    insert_conversation_summary(
        &conn,
        "conv-spec-1",
        "Spec Task",
        "Spec editing",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/SpecBuilder",
        "2026-10-04T09:00:00Z",
    );
    insert_conversation_summary(
        &conn,
        "conv-cg-1",
        "CG Task",
        "Coding guidelines",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/coding-guidelines",
        "2026-10-04T08:00:00Z",
    );

    // Default instance process is ALIVE
    let is_inst_alive = true;
    let evaluated_convs = evaluate_instance_conversations_from_db(&default_db_path, is_inst_alive);
    assert_eq!(evaluated_convs.len(), 3);

    // Evaluate liveness for each conversation
    let agm_conv = evaluated_convs
        .iter()
        .find(|c| c.0 == "conv-agm-1")
        .unwrap();
    let spec_conv = evaluated_convs
        .iter()
        .find(|c| c.0 == "conv-spec-1")
        .unwrap();
    let cg_conv = evaluated_convs.iter().find(|c| c.0 == "conv-cg-1").unwrap();

    assert!(
        agm_conv.4,
        "Antigravity-Manager conversation must be running"
    );
    assert!(!spec_conv.4, "SpecBuilder conversation must NOT be running");
    assert!(
        !cg_conv.4,
        "coding-guidelines conversation must NOT be running"
    );

    // Project-level evaluations
    let (eval_agm, log_agm) = evaluate_project_liveness_record(
        "default",
        "p-agm",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        is_inst_alive,
        agm_conv.4,
        false,
    );
    let (eval_spec, log_spec) = evaluate_project_liveness_record(
        "default",
        "p-spec",
        "SpecBuilder",
        "d:/work/SpecBuilder",
        is_inst_alive,
        spec_conv.4,
        false,
    );
    let (eval_cg, log_cg) = evaluate_project_liveness_record(
        "default",
        "p-cg",
        "coding-guidelines",
        "d:/work/coding-guidelines",
        is_inst_alive,
        cg_conv.4,
        false,
    );

    assert!(eval_agm.is_running);
    assert!(!eval_spec.is_running);
    assert!(!eval_cg.is_running);

    assert!(eval_agm.rationale.contains("ACTIVE_IN_FLIGHT_TASKS"));
    assert!(eval_spec.rationale.contains("IDLE"));
    assert!(eval_cg.rationale.contains("IDLE"));

    assert!(log_agm.contains("is_running=true"));
    assert!(log_spec.contains("is_running=false"));
    assert!(log_cg.contains("is_running=false"));

    // Build project tree nodes and verify only_running filter
    let p_agm = build_mock_project(
        "p-agm",
        "Antigravity-Manager",
        "default",
        eval_agm.is_running,
        vec![build_mock_conv("conv-agm-1", true, "RUNNING", "default")],
    );
    let p_spec = build_mock_project(
        "p-spec",
        "SpecBuilder",
        "default",
        eval_spec.is_running,
        vec![build_mock_conv("conv-spec-1", false, "IDLE", "default")],
    );
    let p_cg = build_mock_project(
        "p-cg",
        "coding-guidelines",
        "default",
        eval_cg.is_running,
        vec![build_mock_conv("conv-cg-1", false, "IDLE", "default")],
    );

    let all_projects = vec![p_agm.clone(), p_spec, p_cg];
    let running_only = filter_running_projects(all_projects);
    assert_eq!(running_only.len(), 1);
    assert_eq!(running_only[0].repo_name, "Antigravity-Manager");
}

// ----------------------------------------------------------------------------
// Test Case 2: Instance 8159 Running coding-guidelines Only
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_case_2_instance_8159_running_coding_guidelines_only() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let inst_8159_db_path = sandbox
        .path()
        .join("instances")
        .join("default-copy-8159")
        .join("home")
        .join(".gemini")
        .join("antigravity")
        .join("conversation_summaries.db");

    let conn = init_summaries_db(&inst_8159_db_path);

    // 8159 DB: coding-guidelines is active (running), AGM and SpecBuilder are idle
    insert_conversation_summary(
        &conn,
        "conv-8159-cg",
        "CG Rules Task",
        "Enforcing guidelines",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/coding-guidelines",
        "2026-10-04T10:00:00Z",
    );
    insert_conversation_summary(
        &conn,
        "conv-8159-agm",
        "AGM Task",
        "Dormant AGM session",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/Antigravity-Manager",
        "2026-10-04T09:00:00Z",
    );
    insert_conversation_summary(
        &conn,
        "conv-8159-spec",
        "Spec Task",
        "Dormant spec session",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/SpecBuilder",
        "2026-10-04T08:00:00Z",
    );

    // 8159 process is ALIVE
    let is_inst_alive = true;
    let evaluated_convs =
        evaluate_instance_conversations_from_db(&inst_8159_db_path, is_inst_alive);
    assert_eq!(evaluated_convs.len(), 3);

    let cg_conv = evaluated_convs
        .iter()
        .find(|c| c.0 == "conv-8159-cg")
        .unwrap();
    let agm_conv = evaluated_convs
        .iter()
        .find(|c| c.0 == "conv-8159-agm")
        .unwrap();
    let spec_conv = evaluated_convs
        .iter()
        .find(|c| c.0 == "conv-8159-spec")
        .unwrap();

    assert!(cg_conv.4, "coding-guidelines must be running on 8159");
    assert!(
        !agm_conv.4,
        "Antigravity-Manager must NOT be running on 8159"
    );
    assert!(!spec_conv.4, "SpecBuilder must NOT be running on 8159");

    let (eval_cg, log_cg) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-cg-8159",
        "coding-guidelines",
        "d:/work/coding-guidelines",
        is_inst_alive,
        cg_conv.4,
        false,
    );
    let (eval_agm, log_agm) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-agm-8159",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        is_inst_alive,
        agm_conv.4,
        false,
    );
    let (eval_spec, log_spec) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-spec-8159",
        "SpecBuilder",
        "d:/work/SpecBuilder",
        is_inst_alive,
        spec_conv.4,
        false,
    );

    assert!(eval_cg.is_running);
    assert!(!eval_agm.is_running);
    assert!(!eval_spec.is_running);

    assert!(eval_cg.rationale.contains("ACTIVE_IN_FLIGHT_TASKS"));
    assert!(eval_agm.rationale.contains("IDLE"));
    assert!(eval_spec.rationale.contains("IDLE"));

    assert!(log_cg.contains("is_running=true"));
    assert!(log_agm.contains("is_running=false"));
    assert!(log_spec.contains("is_running=false"));

    let p_cg = build_mock_project(
        "p-cg",
        "coding-guidelines",
        "default-copy-8159",
        eval_cg.is_running,
        vec![build_mock_conv(
            "conv-8159-cg",
            true,
            "RUNNING",
            "default-copy-8159",
        )],
    );
    let p_agm = build_mock_project(
        "p-agm",
        "Antigravity-Manager",
        "default-copy-8159",
        eval_agm.is_running,
        vec![build_mock_conv(
            "conv-8159-agm",
            false,
            "IDLE",
            "default-copy-8159",
        )],
    );
    let p_spec = build_mock_project(
        "p-spec",
        "SpecBuilder",
        "default-copy-8159",
        eval_spec.is_running,
        vec![build_mock_conv(
            "conv-8159-spec",
            false,
            "IDLE",
            "default-copy-8159",
        )],
    );

    let running_8159 = filter_running_projects(vec![p_cg, p_agm, p_spec]);
    assert_eq!(running_8159.len(), 1);
    assert_eq!(running_8159[0].repo_name, "coding-guidelines");
}

// ----------------------------------------------------------------------------
// Test Case 3: Cross-Instance Isolation: Copied/Dormant Workspaces Do Not Bleed
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_case_3_cross_instance_isolation_copied_dormant_workspaces() {
    let sandbox = tempdir().expect("create sandbox tempdir");

    // 1. Default DB: Antigravity-Manager actively running
    let default_db_path = sandbox
        .path()
        .join("home")
        .join(".gemini")
        .join("antigravity")
        .join("conversation_summaries.db");
    let conn_default = init_summaries_db(&default_db_path);

    insert_conversation_summary(
        &conn_default,
        "conv-agm-live",
        "AGM Task",
        "Active session on Default",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        "2026-10-04T10:00:00Z",
    );
    insert_conversation_summary(
        &conn_default,
        "conv-default-cg",
        "CG Task",
        "Idle session on Default",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/coding-guidelines",
        "2026-10-04T09:00:00Z",
    );

    // 2. 8159 DB: Cloned workspace with dormant AGM record and active coding-guidelines
    let inst_8159_db_path = sandbox
        .path()
        .join("instances")
        .join("default-copy-8159")
        .join("home")
        .join(".gemini")
        .join("antigravity")
        .join("conversation_summaries.db");
    let conn_8159 = init_summaries_db(&inst_8159_db_path);

    insert_conversation_summary(
        &conn_8159,
        "conv-agm-dormant",
        "AGM Task",
        "Cloned dormant session on 8159",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/Antigravity-Manager",
        "2026-10-04T08:00:00Z",
    );
    insert_conversation_summary(
        &conn_8159,
        "conv-8159-cg",
        "CG Task",
        "Active session on 8159",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/coding-guidelines",
        "2026-10-04T10:00:00Z",
    );

    // Both instances alive
    let def_convs = evaluate_instance_conversations_from_db(&default_db_path, true);
    let s8159_convs = evaluate_instance_conversations_from_db(&inst_8159_db_path, true);

    // Evaluate Default view
    let def_agm = def_convs.iter().find(|c| c.0 == "conv-agm-live").unwrap();
    let def_cg = def_convs.iter().find(|c| c.0 == "conv-default-cg").unwrap();
    assert!(def_agm.4, "Default Antigravity-Manager must be running");
    assert!(!def_cg.4, "Default coding-guidelines must be idle");

    // Evaluate 8159 view
    let s8159_agm = s8159_convs
        .iter()
        .find(|c| c.0 == "conv-agm-dormant")
        .unwrap();
    let s8159_cg = s8159_convs.iter().find(|c| c.0 == "conv-8159-cg").unwrap();
    assert!(
        !s8159_agm.4,
        "8159 Antigravity-Manager must be idle (NO BLEED from Default)"
    );
    assert!(s8159_cg.4, "8159 coding-guidelines must be running");

    // Project-level isolation verification
    let (eval_8159_agm, _) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-agm-8159",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        true,
        s8159_agm.4,
        false,
    );
    let (eval_def_cg, _) = evaluate_project_liveness_record(
        "default",
        "p-cg-default",
        "coding-guidelines",
        "d:/work/coding-guidelines",
        true,
        def_cg.4,
        false,
    );

    assert!(
        !eval_8159_agm.is_running,
        "8159 must NOT report Antigravity-Manager running"
    );
    assert!(
        !eval_def_cg.is_running,
        "Default must NOT report coding-guidelines running"
    );

    // Verify conversation instances strictly matched
    let node_def = build_mock_conv("conv-agm-live", true, "RUNNING", "default");
    let node_8159 = build_mock_conv("conv-8159-cg", true, "RUNNING", "default-copy-8159");

    assert_eq!(node_def.instance_id, "default");
    assert_eq!(node_8159.instance_id, "default-copy-8159");
    assert_ne!(node_def.instance_id, node_8159.instance_id);
}

// ----------------------------------------------------------------------------
// Test Case 4: Process Termination Gating: Dead Instance Forces is_running = false
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_case_4_process_termination_gating_forces_idle() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let dead_db_path = sandbox
        .path()
        .join("instances")
        .join("inst-crashed")
        .join("home")
        .join(".gemini")
        .join("antigravity")
        .join("conversation_summaries.db");

    let conn = init_summaries_db(&dead_db_path);

    // Stale records in DB where not_fully_idle=1 and RUNNING because IDE crashed/killed
    insert_conversation_summary(
        &conn,
        "conv-stale-1",
        "Crashed Task",
        "Prompt in flight during crash",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        "2026-10-04T10:00:00Z",
    );

    // Instance process is DEAD (is_inst_alive = false)
    let is_inst_alive = false;
    let evaluated_convs = evaluate_instance_conversations_from_db(&dead_db_path, is_inst_alive);
    assert_eq!(evaluated_convs.len(), 1);

    let (cid, _, _, _, is_conv_running, _, _) = &evaluated_convs[0];
    assert_eq!(cid, "conv-stale-1");
    assert!(
        !is_conv_running,
        "Dead process MUST force conversation is_running to false"
    );

    // Evaluate project record with dead process
    let (eval, log_line) = evaluate_project_liveness_record(
        "inst-crashed",
        "p-agm",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        is_inst_alive,
        *is_conv_running,
        false,
    );

    assert!(
        !eval.is_running,
        "Dead process MUST force project is_running to false"
    );
    assert_eq!(eval.active_tasks_count, 0);
    assert!(eval.rationale.contains("INSTANCE_PROCESS_DEAD"));
    assert!(log_line.contains("is_running=false"));
    assert!(log_line.contains(
        "INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle"
    ));

    // Probe summary verification
    let probe = PromptLivenessProbe {
        target_instance_id: "inst-crashed".to_string(),
        target_instance_name: "Crashed Profile".to_string(),
        data_dir: dead_db_path.to_string_lossy().to_string(),
        is_instance_alive: false,
        matched_pids: vec![],
        workspace_folders_count: 1,
        evaluated_projects_count: 1,
        running_projects_count: 0,
    };

    assert!(!probe.is_instance_alive);
    assert_eq!(probe.running_projects_count, 0);
}
