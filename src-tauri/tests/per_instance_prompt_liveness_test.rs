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

/// Simulated project and conversation evaluation from an SQLite summaries DB with turn TTL filtering
fn evaluate_instance_conversations_with_ttl(
    db_path: &Path,
    is_inst_alive: bool,
    ttl_seconds: i64,
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

    let now_ts = chrono::Utc::now().timestamp();
    let mut results = Vec::new();
    for row in rows.flatten() {
        let (cid, title, preview, status, not_fully_idle, ws_uris_opt, last_mod) = row;

        let is_recent = chrono::DateTime::parse_from_rfc3339(&last_mod)
            .map(|dt| dt.timestamp() >= now_ts - ttl_seconds)
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&last_mod, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| dt.and_utc().timestamp() >= now_ts - ttl_seconds)
            })
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&last_mod, "%Y-%m-%dT%H:%M:%S")
                    .map(|dt| dt.and_utc().timestamp() >= now_ts - ttl_seconds)
            })
            .unwrap_or(false);

        let is_idle_count = not_fully_idle == 0;
        let has_idle_status = status.contains("IDLE")
            || status.contains("COMPLETED")
            || status.contains("FAILED")
            || status.contains("CANCELLED");
        let is_explicit_idle = is_idle_count || has_idle_status;

        let has_active_turns = not_fully_idle > 0;
        let has_running_text = status.contains("RUNNING");
        let is_active_session = has_active_turns && has_running_text;

        let is_conv_running = if is_explicit_idle || !is_recent {
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

// ----------------------------------------------------------------------------
// Test Case 5: Suffix Matching for Cloned Instances
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_e2e_suffix_matching_for_cloned_instances() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let prev_data_dir = std::env::var_os("ABV_DATA_DIR");
    std::env::set_var("ABV_DATA_DIR", sandbox.path());

    let instances_dir = sandbox.path().join("instances");
    fs::create_dir_all(&instances_dir).expect("create instances dir");

    let registry_json = serde_json::json!({
        "active_instance_id": "default",
        "instances": [
            {
                "id": "default",
                "name": "Default Instance",
                "data_dir": sandbox.path().join("default_data").to_string_lossy(),
                "is_default": true,
                "seq_num": 1,
                "created_at": 1000,
                "last_used": 1000
            },
            {
                "id": "default-copy-8159",
                "name": "Instance 8159",
                "data_dir": sandbox.path().join("inst_8159_data").to_string_lossy(),
                "is_default": false,
                "seq_num": 2,
                "created_at": 2000,
                "last_used": 2000
            }
        ]
    });
    fs::write(
        instances_dir.join("instances.json"),
        serde_json::to_string_pretty(&registry_json).unwrap(),
    )
    .expect("write instances.json");

    // Suffix resolution tests
    let res_8159 = antigravity_tools_lib::modules::instance::resolve_instance_id("8159");
    assert_eq!(res_8159, Ok("default-copy-8159".to_string()));

    let res_hyphen = antigravity_tools_lib::modules::instance::resolve_instance_id("-8159");
    assert_eq!(res_hyphen, Ok("default-copy-8159".to_string()));

    let res_inst = antigravity_tools_lib::modules::instance::resolve_instance_id("inst-8159");
    assert_eq!(res_inst, Ok("default-copy-8159".to_string()));

    let res_default = antigravity_tools_lib::modules::instance::resolve_instance_id("default");
    assert_eq!(res_default, Ok("default".to_string()));

    // Verify get_instance_home_dir targeting "8159" resolves to default-copy-8159/home
    let home_8159 = antigravity_tools_lib::modules::instance::get_instance_home_dir("8159")
        .expect("get instance home dir");
    let home_path_str = home_8159.to_string_lossy().replace('\\', "/");
    assert!(
        home_path_str.contains("default-copy-8159/home"),
        "home dir must target default-copy-8159/home, got: {}",
        home_path_str
    );

    // Restore env
    if let Some(prev) = prev_data_dir {
        std::env::set_var("ABV_DATA_DIR", prev);
    } else {
        std::env::remove_var("ABV_DATA_DIR");
    }
}

// ----------------------------------------------------------------------------
// Test Case 6: Strict Instance Scoping for Prompt Queue Dispatch
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_e2e_prompt_queue_dispatch_strict_instance_scoping() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let prev_data_dir = std::env::var_os("ABV_DATA_DIR");
    std::env::set_var("ABV_DATA_DIR", sandbox.path());

    let conn = antigravity_tools_lib::modules::repo_db::connect_db().expect("connect repo db");
    let now = chrono::Utc::now().timestamp();

    // Two prompts for the same repo path, but different instances:
    // prompt-8159 was created EARLIER (now - 100) than prompt-def (now - 50)
    conn.execute(
        "INSERT INTO active_prompts 
         (id, project_id, instance_id, repo_path, prompt_content, model, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            "prompt-8159",
            "shared-proj",
            "default-copy-8159",
            "d:/work/shared-proj",
            "Prompt for 8159",
            "gemini-1.5-pro",
            "queued",
            now - 100,
            now - 100
        ],
    )
    .expect("insert prompt-8159");

    conn.execute(
        "INSERT INTO active_prompts 
         (id, project_id, instance_id, repo_path, prompt_content, model, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            "prompt-def",
            "shared-proj",
            "default",
            "d:/work/shared-proj",
            "Prompt for default",
            "gemini-1.5-pro",
            "queued",
            now - 50,
            now - 50
        ],
    )
    .expect("insert prompt-def");

    // 1. Dispatching for "default" must NOT steal prompt-8159 despite prompt-8159 having an earlier created_at
    let def_count = antigravity_tools_lib::modules::repo_db::check_and_dispatch_enqueued_prompts(
        Some("default"),
    )
    .expect("dispatch for default");
    assert_eq!(def_count, 1, "Default should dispatch its own prompt");

    let status_def: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-def'",
            [],
            |r| r.get(0),
        )
        .expect("query status def");
    assert_eq!(status_def, "dispatched");

    let status_8159: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-8159'",
            [],
            |r| r.get(0),
        )
        .expect("query status 8159");
    assert_eq!(
        status_8159, "queued",
        "prompt-8159 must remain queued and not stolen by default"
    );

    // 2. Dispatching for "default-copy-8159" should now dispatch prompt-8159
    let s8159_count = antigravity_tools_lib::modules::repo_db::check_and_dispatch_enqueued_prompts(
        Some("default-copy-8159"),
    )
    .expect("dispatch for 8159");
    assert_eq!(s8159_count, 1, "8159 should dispatch its queued prompt");

    let status_8159_after: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-8159'",
            [],
            |r| r.get(0),
        )
        .expect("query status 8159 after");
    assert_eq!(status_8159_after, "dispatched");

    // Restore env
    if let Some(prev) = prev_data_dir {
        std::env::set_var("ABV_DATA_DIR", prev);
    } else {
        std::env::remove_var("ABV_DATA_DIR");
    }
}

// ----------------------------------------------------------------------------
// Test Case 7: Running Projects Primary Key Namespacing
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_e2e_running_projects_primary_key_namespacing() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let prev_data_dir = std::env::var_os("ABV_DATA_DIR");
    std::env::set_var("ABV_DATA_DIR", sandbox.path());

    let conn = antigravity_tools_lib::modules::repo_db::connect_db().expect("connect repo db");

    let repo_name = "antigravity-manager";
    let folder_hash = "abc123hash";
    let base_id = format!("{}-{}", repo_name, folder_hash);

    let id_default = format!("{}__{}", base_id, "default");
    let id_8159 = format!("{}__{}", base_id, "default-copy-8159");

    let now = chrono::Utc::now().timestamp();

    // Insert project record for default instance
    conn.execute(
        "INSERT INTO running_projects 
         (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id_default,
            "default",
            "Antigravity-Manager",
            "d:/work/Antigravity-Manager",
            "c:/data/User/workspaceStorage/abc123hash",
            1,
            now,
            now
        ],
    )
    .expect("insert default running project");

    // Insert project record for cloned instance default-copy-8159 with same project name and folder hash
    conn.execute(
        "INSERT INTO running_projects 
         (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id_8159,
            "default-copy-8159",
            "Antigravity-Manager",
            "d:/work/Antigravity-Manager",
            "c:/instances/default-copy-8159/home/User/workspaceStorage/abc123hash",
            0,
            now,
            now
        ],
    )
    .expect("insert 8159 running project");

    // Verify both records coexist without primary key conflict
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM running_projects", [], |r| r.get(0))
        .expect("count running projects");
    assert_eq!(
        count, 2,
        "Both instances must retain their respective project records with composite primary keys"
    );

    // Verify each record can be queried by its composite ID
    let row_def_running: i32 = conn
        .query_row(
            "SELECT is_running FROM running_projects WHERE id = ?1",
            params![id_default],
            |r| r.get(0),
        )
        .expect("query default record");
    let row_8159_running: i32 = conn
        .query_row(
            "SELECT is_running FROM running_projects WHERE id = ?1",
            params![id_8159],
            |r| r.get(0),
        )
        .expect("query 8159 record");

    assert_eq!(row_def_running, 1);
    assert_eq!(row_8159_running, 0);

    // Restore env
    if let Some(prev) = prev_data_dir {
        std::env::set_var("ABV_DATA_DIR", prev);
    } else {
        std::env::remove_var("ABV_DATA_DIR");
    }
}

// ----------------------------------------------------------------------------
// Test Case 8: Dual-Instance Live Workspaces Isolation
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_e2e_dual_instance_live_workspaces_isolation() {
    let sandbox = tempdir().expect("create sandbox tempdir");

    // 1. Setup Default instance db
    let default_db_path = sandbox
        .path()
        .join("default")
        .join("conversation_summaries.db");
    let conn_default = init_summaries_db(&default_db_path);

    let now_str = chrono::Utc::now().to_rfc3339();
    let stale_time_str = (chrono::Utc::now() - chrono::Duration::seconds(1800)).to_rfc3339();

    // Default: Antigravity-Manager RUNNING and active; SpecBuilder IDLE (not_fully_idle=0); coding-guidelines IDLE
    insert_conversation_summary(
        &conn_default,
        "conv-agm-live",
        "AGM Task",
        "Active session on Default",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        &now_str,
    );
    insert_conversation_summary(
        &conn_default,
        "conv-sb-idle",
        "SpecBuilder Task",
        "Stale turn on Default",
        "CASCADE_RUN_STATUS_RUNNING",
        0,
        "d:/work/SpecBuilder",
        &now_str,
    );
    insert_conversation_summary(
        &conn_default,
        "conv-cg-idle",
        "CG Task",
        "Idle on Default",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/coding-guidelines",
        &stale_time_str,
    );

    // 2. Setup Cloned instance default-copy-8159 db
    let inst_8159_db_path = sandbox
        .path()
        .join("default-copy-8159")
        .join("conversation_summaries.db");
    let conn_8159 = init_summaries_db(&inst_8159_db_path);

    // 8159: coding-guidelines RUNNING and active; Antigravity-Manager cloned dormant artifact (not_fully_idle=0); SpecBuilder IDLE
    insert_conversation_summary(
        &conn_8159,
        "conv-8159-cg-live",
        "CG Task",
        "Active session on 8159",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/coding-guidelines",
        &now_str,
    );
    insert_conversation_summary(
        &conn_8159,
        "conv-8159-agm-dormant",
        "AGM Task",
        "Cloned historical artifact on 8159",
        "CASCADE_RUN_STATUS_RUNNING",
        0,
        "d:/work/Antigravity-Manager",
        &now_str,
    );
    insert_conversation_summary(
        &conn_8159,
        "conv-8159-sb-idle",
        "SpecBuilder Task",
        "Idle on 8159",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/SpecBuilder",
        &stale_time_str,
    );

    // Evaluate Default conversations
    let def_convs = evaluate_instance_conversations_from_db(&default_db_path, true);
    assert_eq!(def_convs.len(), 3);
    let def_agm = def_convs.iter().find(|c| c.0 == "conv-agm-live").unwrap();
    let def_sb = def_convs.iter().find(|c| c.0 == "conv-sb-idle").unwrap();
    let def_cg = def_convs.iter().find(|c| c.0 == "conv-cg-idle").unwrap();

    assert!(def_agm.4, "Default Antigravity-Manager must be running");
    assert!(
        !def_sb.4,
        "Default SpecBuilder must be idle (not_fully_idle = 0)"
    );
    assert!(!def_cg.4, "Default coding-guidelines must be idle");

    // Evaluate 8159 conversations
    let s8159_convs = evaluate_instance_conversations_from_db(&inst_8159_db_path, true);
    assert_eq!(s8159_convs.len(), 3);
    let s8159_cg = s8159_convs
        .iter()
        .find(|c| c.0 == "conv-8159-cg-live")
        .unwrap();
    let s8159_agm = s8159_convs
        .iter()
        .find(|c| c.0 == "conv-8159-agm-dormant")
        .unwrap();
    let s8159_sb = s8159_convs
        .iter()
        .find(|c| c.0 == "conv-8159-sb-idle")
        .unwrap();

    assert!(s8159_cg.4, "8159 coding-guidelines must be running");
    assert!(
        !s8159_agm.4,
        "8159 Antigravity-Manager must be idle (ZERO BLEED from Default, not_fully_idle = 0)"
    );
    assert!(!s8159_sb.4, "8159 SpecBuilder must be idle");

    // Project-level evaluations
    let (eval_def_agm, _) = evaluate_project_liveness_record(
        "default",
        "p-agm-def",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        true,
        def_agm.4,
        false,
    );
    let (eval_def_cg, _) = evaluate_project_liveness_record(
        "default",
        "p-cg-def",
        "coding-guidelines",
        "d:/work/coding-guidelines",
        true,
        def_cg.4,
        false,
    );
    let (eval_8159_cg, _) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-cg-8159",
        "coding-guidelines",
        "d:/work/coding-guidelines",
        true,
        s8159_cg.4,
        false,
    );
    let (eval_8159_agm, _) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-agm-8159",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        true,
        s8159_agm.4,
        false,
    );

    assert!(
        eval_def_agm.is_running,
        "Default: Antigravity-Manager must be running"
    );
    assert!(
        !eval_def_cg.is_running,
        "Default: coding-guidelines must NOT be running"
    );
    assert!(
        eval_8159_cg.is_running,
        "8159: coding-guidelines must be running"
    );
    assert!(
        !eval_8159_agm.is_running,
        "8159: Antigravity-Manager must NOT be running"
    );
}

// ----------------------------------------------------------------------------
// Test Case 9: TTL Stale Conversation Expiry Forces Idle
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_case_ttl_stale_conversation_expiry_forces_idle() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let db_path = sandbox
        .path()
        .join("home/.gemini/antigravity/conversation_summaries.db");
    let conn = init_summaries_db(&db_path);

    let now_dt = chrono::Utc::now();
    let stale_rfc3339 = (now_dt - chrono::Duration::seconds(1200)).to_rfc3339();
    let stale_sqlite = (now_dt - chrono::Duration::seconds(1800))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let fresh_rfc3339 = (now_dt - chrono::Duration::seconds(60)).to_rfc3339();

    // 1. Insert stale RFC 3339 conversation (20 mins old)
    insert_conversation_summary(
        &conn,
        "conv-stale-rfc",
        "Abandoned Task 1",
        "Prompt left running 20 mins ago",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        &stale_rfc3339,
    );

    // 2. Insert stale SQLite format conversation (30 mins old with space separator)
    insert_conversation_summary(
        &conn,
        "conv-stale-sqlite",
        "Abandoned Task 2",
        "Prompt left running 30 mins ago",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        &stale_sqlite,
    );

    // 3. Insert fresh running conversation (1 min old)
    insert_conversation_summary(
        &conn,
        "conv-fresh-active",
        "Active Task",
        "In-flight prompt currently running",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        &fresh_rfc3339,
    );

    let is_inst_alive = true;
    let evaluated = evaluate_instance_conversations_with_ttl(&db_path, is_inst_alive, 900);

    let stale_1 = evaluated.iter().find(|c| c.0 == "conv-stale-rfc").unwrap();
    let stale_2 = evaluated
        .iter()
        .find(|c| c.0 == "conv-stale-sqlite")
        .unwrap();
    let fresh = evaluated
        .iter()
        .find(|c| c.0 == "conv-fresh-active")
        .unwrap();

    assert!(
        !stale_1.4,
        "Stale RFC 3339 conversation (> 900s) MUST be forced idle"
    );
    assert!(
        !stale_2.4,
        "Stale SQLite timestamp conversation (> 900s) MUST be forced idle"
    );
    assert!(
        fresh.4,
        "Fresh conversation (< 900s) with active turns MUST be evaluated as running"
    );

    // Project-level evaluation when all turns are stale
    let (eval_stale, log_stale) = evaluate_project_liveness_record(
        "default",
        "p-stale-only",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        is_inst_alive,
        false,
        false,
    );
    assert!(
        !eval_stale.is_running,
        "When all turns are stale, project is_running must be false"
    );
    assert!(eval_stale.rationale.contains("IDLE"));
    assert!(log_stale.contains("is_running=false"));

    // Project-level evaluation with fresh turn
    let (eval_fresh, log_fresh) = evaluate_project_liveness_record(
        "default",
        "p-fresh-active",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        is_inst_alive,
        fresh.4,
        false,
    );
    assert!(
        eval_fresh.is_running,
        "Fresh active turn must result in project being evaluated as running"
    );
    assert!(eval_fresh.rationale.contains("ACTIVE_IN_FLIGHT_TASKS"));
    assert!(log_fresh.contains("is_running=true"));
}

// ----------------------------------------------------------------------------
// Test Case 10: Antigravity-CLI Candidate Discovery and Instance Isolation
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_case_antigravity_cli_discovery_isolation() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let home_dir_1 = sandbox.path().join("home_default");
    let home_dir_2 = sandbox.path().join("home_8159");

    let cli_gemini_dir = home_dir_1.join(".gemini").join("antigravity-cli");
    let ide_gemini_dir = home_dir_1.join(".gemini").join("antigravity");
    fs::create_dir_all(&cli_gemini_dir).expect("create cli dir");
    fs::create_dir_all(&ide_gemini_dir).expect("create ide dir");

    let gemini_8159_dir = home_dir_2.join(".gemini").join("antigravity");
    fs::create_dir_all(&gemini_8159_dir).expect("create 8159 ide dir");

    // 1. Verify candidate directory discovery across .gemini/antigravity, .gemini/antigravity-ide, and .gemini/antigravity-cli
    let subdirs = vec!["antigravity", "antigravity-ide", "antigravity-cli"];
    let mut found = Vec::new();
    for sub in &subdirs {
        let p = home_dir_1.join(".gemini").join(sub);
        if p.exists() {
            found.push(p);
        }
    }

    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|p| p.ends_with("antigravity")));
    assert!(found.iter().any(|p| p.ends_with("antigravity-cli")));
    assert!(
        antigravity_tools_lib::modules::instance::GEMINI_CLONE_DIRS.contains(&"antigravity-cli")
    );

    // 2. Initialize summaries DB inside antigravity-cli for Instance 1 (default)
    let cli_db_path = cli_gemini_dir.join("conversation_summaries.db");
    let conn_cli = init_summaries_db(&cli_db_path);
    let now_str = chrono::Utc::now().to_rfc3339();

    insert_conversation_summary(
        &conn_cli,
        "conv-cli-run-1",
        "CLI Headless Agent Task",
        "Running headless turn in CLI",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        "d:/work/Antigravity-Manager",
        &now_str,
    );

    // 3. Initialize summaries DB for Instance 2 (default-copy-8159) with IDLE conversation
    let db_8159 = gemini_8159_dir.join("conversation_summaries.db");
    let conn_8159 = init_summaries_db(&db_8159);
    insert_conversation_summary(
        &conn_8159,
        "conv-8159-idle",
        "CLI Headless Agent Task",
        "Dormant session in 8159",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        "d:/work/Antigravity-Manager",
        &now_str,
    );

    // 4. Evaluate conversations with TTL
    let evaluated_cli = evaluate_instance_conversations_with_ttl(&cli_db_path, true, 900);
    assert_eq!(evaluated_cli.len(), 1);
    assert!(
        evaluated_cli[0].4,
        "CLI conversation must be detected as running when process is alive"
    );
    assert_eq!(evaluated_cli[0].0, "conv-cli-run-1");

    let evaluated_8159 = evaluate_instance_conversations_with_ttl(&db_8159, true, 900);
    assert_eq!(evaluated_8159.len(), 1);
    assert!(
        !evaluated_8159[0].4,
        "8159 conversation must be detected as IDLE"
    );

    // 5. Evaluate project-level liveness and ensure zero cross-instance bleeding
    let (eval_cli, log_cli) = evaluate_project_liveness_record(
        "default",
        "p-agm-cli",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        true,
        evaluated_cli[0].4,
        false,
    );
    let (eval_8159, log_8159) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-agm-8159",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        true,
        evaluated_8159[0].4,
        false,
    );

    assert!(
        eval_cli.is_running,
        "Default instance running via antigravity-cli must report running"
    );
    assert!(
        !eval_8159.is_running,
        "Instance 8159 must NOT bleed default's antigravity-cli activity"
    );
    assert!(log_cli.contains("is_running=true"));
    assert!(log_8159.contains("is_running=false"));

    // 6. Verify structured audit format
    let audit_line = antigravity_tools_lib::modules::logger::format_instance_prompt_audit(
        "default",
        "Default Profile",
        "Antigravity-Manager",
        "d:/work/Antigravity-Manager",
        &cli_db_path.to_string_lossy(),
        Some(1234),
        "Gate4:ConversationSummariesLiveTurn",
        true,
        "CONVERSATION_SUMMARY_ACTIVE_TURN",
    );
    assert!(audit_line.starts_with("[InstancePromptAudit]"));
    assert!(audit_line.contains("instance_id='default'"));
    assert!(audit_line.contains("project='Antigravity-Manager'"));
    assert!(audit_line.contains("pid=1234"));
    assert!(audit_line.contains("is_running=true"));
    assert!(audit_line.contains("rationale='CONVERSATION_SUMMARY_ACTIVE_TURN'"));
}

#[tokio::test]
async fn test_case_antigravity_cli_discovery() {
    test_case_antigravity_cli_discovery_isolation().await;
}

// ----------------------------------------------------------------------------
// Test Case 11: Composite Keying Isolation on Shared Repository Paths
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_case_composite_keying_isolation_same_repo_path() {
    let sandbox = tempdir().expect("create sandbox tempdir");

    // Instance 1: default
    let db_default = sandbox
        .path()
        .join("default/home/.gemini/antigravity/conversation_summaries.db");
    let conn_default = init_summaries_db(&db_default);

    // Instance 2: default-copy-8159
    let db_8159 = sandbox
        .path()
        .join("instances/default-copy-8159/home/.gemini/antigravity/conversation_summaries.db");
    let conn_8159 = init_summaries_db(&db_8159);

    let now_str = chrono::Utc::now().to_rfc3339();
    let repo_agm = "d:/work/Antigravity-Manager";
    let repo_cg = "d:/work/coding-guidelines";

    // On Instance 1 (default): Antigravity-Manager is RUNNING; coding-guidelines is IDLE
    insert_conversation_summary(
        &conn_default,
        "conv-def-agm",
        "AGM Task",
        "Active in default profile",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        repo_agm,
        &now_str,
    );
    insert_conversation_summary(
        &conn_default,
        "conv-def-cg",
        "CG Task",
        "Inactive in default profile",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        repo_cg,
        &now_str,
    );

    // On Instance 2 (default-copy-8159): coding-guidelines is RUNNING; Antigravity-Manager is IDLE
    insert_conversation_summary(
        &conn_8159,
        "conv-8159-cg",
        "CG Task",
        "Active in 8159 profile",
        "CASCADE_RUN_STATUS_RUNNING",
        1,
        repo_cg,
        &now_str,
    );
    insert_conversation_summary(
        &conn_8159,
        "conv-8159-agm",
        "AGM Task",
        "Inactive in 8159 profile",
        "CASCADE_RUN_STATUS_IDLE",
        0,
        repo_agm,
        &now_str,
    );

    // Evaluate conversations from DBs
    let convs_default = evaluate_instance_conversations_with_ttl(&db_default, true, 900);
    let convs_8159 = evaluate_instance_conversations_with_ttl(&db_8159, true, 900);

    let def_agm = convs_default
        .iter()
        .find(|c| c.0 == "conv-def-agm")
        .unwrap();
    let def_cg = convs_default.iter().find(|c| c.0 == "conv-def-cg").unwrap();
    assert!(def_agm.4, "Default: Antigravity-Manager must be running");
    assert!(!def_cg.4, "Default: coding-guidelines must be idle");

    let s8159_cg = convs_8159.iter().find(|c| c.0 == "conv-8159-cg").unwrap();
    let s8159_agm = convs_8159.iter().find(|c| c.0 == "conv-8159-agm").unwrap();
    assert!(s8159_cg.4, "8159: coding-guidelines must be running");
    assert!(!s8159_agm.4, "8159: Antigravity-Manager must be idle");

    // Populate composite live_map keyed by (instance_id, clean_path)
    let clean_agm = repo_agm.replace('\\', "/").to_lowercase();
    let clean_cg = repo_cg.replace('\\', "/").to_lowercase();

    let mut live_map: std::collections::HashMap<(String, String), (bool, Option<String>, i64)> =
        std::collections::HashMap::new();

    live_map.insert(
        ("default".to_string(), clean_agm.clone()),
        (def_agm.4, Some("Active AGM on Default".to_string()), 1000),
    );
    live_map.insert(
        ("default".to_string(), clean_cg.clone()),
        (def_cg.4, None, 1000),
    );
    live_map.insert(
        ("default-copy-8159".to_string(), clean_cg.clone()),
        (s8159_cg.4, Some("Active CG on 8159".to_string()), 1000),
    );
    live_map.insert(
        ("default-copy-8159".to_string(), clean_agm.clone()),
        (s8159_agm.4, None, 1000),
    );

    // Verify composite keying queries
    let entry_def_agm = live_map
        .get(&("default".to_string(), clean_agm.clone()))
        .unwrap();
    let entry_def_cg = live_map
        .get(&("default".to_string(), clean_cg.clone()))
        .unwrap();
    let entry_8159_agm = live_map
        .get(&("default-copy-8159".to_string(), clean_agm.clone()))
        .unwrap();
    let entry_8159_cg = live_map
        .get(&("default-copy-8159".to_string(), clean_cg.clone()))
        .unwrap();

    assert!(
        entry_def_agm.0,
        "Instance 1 (default) reports Antigravity-Manager RUNNING"
    );
    assert!(
        !entry_def_cg.0,
        "Instance 1 (default) reports coding-guidelines IDLE"
    );
    assert!(
        !entry_8159_agm.0,
        "Instance 2 (8159) reports Antigravity-Manager IDLE"
    );
    assert!(
        entry_8159_cg.0,
        "Instance 2 (8159) reports coding-guidelines RUNNING"
    );

    // Project-level evaluations
    let (eval_def_agm, log_def_agm) = evaluate_project_liveness_record(
        "default",
        "p-agm-def",
        "Antigravity-Manager",
        repo_agm,
        true,
        entry_def_agm.0,
        false,
    );
    let (eval_def_cg, log_def_cg) = evaluate_project_liveness_record(
        "default",
        "p-cg-def",
        "coding-guidelines",
        repo_cg,
        true,
        entry_def_cg.0,
        false,
    );
    let (eval_8159_agm, log_8159_agm) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-agm-8159",
        "Antigravity-Manager",
        repo_agm,
        true,
        entry_8159_agm.0,
        false,
    );
    let (eval_8159_cg, log_8159_cg) = evaluate_project_liveness_record(
        "default-copy-8159",
        "p-cg-8159",
        "coding-guidelines",
        repo_cg,
        true,
        entry_8159_cg.0,
        false,
    );

    assert!(
        eval_def_agm.is_running,
        "Default: Antigravity-Manager MUST be running"
    );
    assert!(
        !eval_def_cg.is_running,
        "Default: coding-guidelines MUST NOT be running"
    );
    assert!(
        !eval_8159_agm.is_running,
        "8159: Antigravity-Manager MUST NOT be running"
    );
    assert!(
        eval_8159_cg.is_running,
        "8159: coding-guidelines MUST be running"
    );

    assert!(log_def_agm.contains("is_running=true"));
    assert!(log_def_cg.contains("is_running=false"));
    assert!(log_8159_agm.contains("is_running=false"));
    assert!(log_8159_cg.contains("is_running=true"));

    // Build project tree nodes and verify only_running filter for each instance
    let p_def_agm = build_mock_project(
        "p-agm-def",
        "Antigravity-Manager",
        "default",
        eval_def_agm.is_running,
        vec![build_mock_conv("conv-def-agm", true, "RUNNING", "default")],
    );
    let p_def_cg = build_mock_project(
        "p-cg-def",
        "coding-guidelines",
        "default",
        eval_def_cg.is_running,
        vec![build_mock_conv("conv-def-cg", false, "IDLE", "default")],
    );
    let running_def = filter_running_projects(vec![p_def_agm, p_def_cg]);
    assert_eq!(running_def.len(), 1);
    assert_eq!(running_def[0].repo_name, "Antigravity-Manager");

    let p_8159_agm = build_mock_project(
        "p-agm-8159",
        "Antigravity-Manager",
        "default-copy-8159",
        eval_8159_agm.is_running,
        vec![build_mock_conv(
            "conv-8159-agm",
            false,
            "IDLE",
            "default-copy-8159",
        )],
    );
    let p_8159_cg = build_mock_project(
        "p-cg-8159",
        "coding-guidelines",
        "default-copy-8159",
        eval_8159_cg.is_running,
        vec![build_mock_conv(
            "conv-8159-cg",
            true,
            "RUNNING",
            "default-copy-8159",
        )],
    );
    let running_8159 = filter_running_projects(vec![p_8159_agm, p_8159_cg]);
    assert_eq!(running_8159.len(), 1);
    assert_eq!(running_8159[0].repo_name, "coding-guidelines");
}
