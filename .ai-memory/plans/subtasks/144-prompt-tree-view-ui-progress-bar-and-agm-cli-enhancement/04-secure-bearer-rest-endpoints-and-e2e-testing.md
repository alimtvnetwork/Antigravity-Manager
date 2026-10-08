# Subtask 04 Spec: Secure Bearer REST Endpoints & E2E Testing Suite

**Document ID:** `.ai-memory/plans/subtasks/144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/04-secure-bearer-rest-endpoints-and-e2e-testing.md`  
**Parent Task:** `144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement`  
**Target Codebase:** `src-tauri/src/proxy/server.rs`, `src-tauri/src/proxy/middleware/auth.rs`, `03-ai-scripts/`  
**Author:** Author 02 (Antigravity Backend & CLI Architecture Specialist)  
**Status:** Approved Engineering Specification  

---

## 1. Scope & Objective

The objective of Subtask 04 is twofold:
1. **Mount Authenticated REST Endpoints in `src-tauri/src/proxy/server.rs`:**  
   Provide direct HTTP REST pathways on port 8045 under `/api/instances/*` and `/api/prompts/*` secured by `admin_auth_middleware` (mandatory Bearer token / API key) and backed by structured audit logging.
2. **Author the Complete End-to-End Test Harness & Runbook:**  
   Provide a step-by-step verification harness that tests the entire lifecycle across 5 critical test cases (Instance Creation & Isolation, Prompt Dispatch & Running Detection, Account Switching Continuity, Compact Tree View Grouping, and CLI/REST Parity).

---

## 2. Server Implementation Architecture (`src-tauri/src/proxy/server.rs`)

### 2.1 Route Registration under `admin_routes`

In `src-tauri/src/proxy/server.rs`, the `admin_routes` router is nested at `/api` and wrapped with `admin_auth_middleware`. We mount the `/instances` and `/prompts` route families directly onto `admin_routes`:

```rust
// In src-tauri/src/proxy/server.rs inside build_server_router()

let admin_routes = Router::new()
    .route("/health", get(health_check_handler))
    // Existing account and stats routes...
    .route("/accounts", get(admin_list_accounts).post(admin_add_account))
    // ...
    
    // ── Instance Management REST Routes ──
    .route(
        "/instances",
        get(admin_list_instances).post(admin_create_instance),
    )
    .route(
        "/instances/:id",
        get(admin_get_instance_detail).delete(admin_delete_instance),
    )
    .route("/instances/:id/start", post(admin_start_instance))
    .route("/instances/:id/stop", post(admin_stop_instance))
    .route("/instances/:id/restart", post(admin_restart_instance))
    .route("/instances/:id/switch", post(admin_switch_instance_account))
    .route("/instances/:id/clone", post(admin_clone_instance))
    
    // ── Prompt Lifecycle REST Routes ──
    .route("/prompts", get(admin_list_prompts).post(admin_send_prompt))
    .route("/prompts/tree", get(admin_get_prompt_tree))
    .route("/prompts/running", get(admin_get_running_prompts))
    .route(
        "/prompts/queue",
        get(admin_list_queued_prompts).post(admin_enqueue_prompt),
    )
    .route("/prompts/backup", post(admin_backup_prompts))
    .route("/prompts/restore", post(admin_restore_prompts))
    .route("/prompts/export", get(admin_export_prompts))

    // Apply strict admin authentication layer across all routes
    .layer(axum::middleware::from_fn_with_state(
        state.clone(),
        admin_auth_middleware,
    ));
```

### 2.2 Route Handlers & Integration with Core Modules

All REST handlers delegate directly to the underlying domain logic in `src-tauri/src/modules/`, ensuring strict behavioral parity with both the CLI and Tauri IPC:

```rust
// In src-tauri/src/proxy/server.rs

// ── Instance Handlers ──

async fn admin_list_instances() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    match crate::modules::instance::list_instances() {
        Ok(instances) => Ok(Json(serde_json::json!({
            "success": true,
            "data": instances,
            "error": null
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

#[derive(Deserialize)]
struct CreateInstancePayload {
    name: String,
}

async fn admin_create_instance(
    Json(payload): Json<CreateInstancePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    match crate::modules::instance::create_instance(payload.name) {
        Ok(inst) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": inst,
                "error": null
            })),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )),
    }
}

async fn admin_start_instance(
    Path(instance_id): Path<String>,
    Json(payload): Json<Option<serde_json::Value>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let repo_path = payload
        .as_ref()
        .and_then(|v| v.get("repoPath"))
        .and_then(|p| p.as_str());

    match crate::modules::instance::launch_instance(&instance_id, repo_path) {
        Ok(pid) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "pid": pid, "status": "running" },
            "error": null
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

// ── Prompt Handlers ──

#[derive(Deserialize)]
struct PromptQueryParameters {
    #[serde(rename = "instanceId")]
    instance_id: Option<String>,
    repo: Option<String>,
}

async fn admin_get_prompt_tree(
    Query(params): Query<PromptQueryParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let target_instance = params.instance_id.as_deref().unwrap_or("default");
    match crate::modules::repo_db::get_project_conversation_tree_for_instance(
        target_instance,
        params.repo.as_deref(),
    ) {
        Ok(tree) => Ok(Json(serde_json::json!({
            "success": true,
            "data": tree,
            "error": null
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

async fn admin_get_running_prompts(
    Query(params): Query<PromptQueryParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let target_instance = params.instance_id.as_deref().unwrap_or("default");
    match crate::modules::repo_db::list_running_prompts_for_instance(target_instance) {
        Ok(running) => Ok(Json(serde_json::json!({
            "success": true,
            "data": running,
            "error": null
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}
```

---

## 3. Bearer Token Enforcement & Audit Trail Logging

### 3.1 Authentication Pipeline

The authentication pipeline is enforced by `admin_auth_middleware` (`src-tauri/src/proxy/middleware/auth.rs`):
1. **Extraction:** Looks for `Authorization: Bearer <secret>` or `x-api-key: <secret>`.
2. **Comparison:** Matches against `security.admin_password` (strict admin password), falling back to `security.api_key`.
3. **Rejection:** Returns HTTP `401 Unauthorized` immediately if credentials are missing or do not match.

```
   Client Request ──► [ admin_auth_middleware ]
                             │
                      Auth Header Present?
                        ├── No  ──► HTTP 401 Unauthorized
                        └── Yes ──► Token Matches?
                                      ├── No  ──► HTTP 401 Unauthorized
                                      └── Yes ──► [ Audit Logger ] ──► Handler
```

### 3.2 Security Audit Logging

All administrative requests passing through `/api/instances/*` and `/api/prompts/*` are recorded into the security audit ledger in `security.db`:
```rust
// Recorded audit attributes:
// - timestamp: ISO 8601 string
// - client_ip: Extracted client IP (IPv4 / IPv6 sanitized)
// - method: HTTP verb (GET, POST, DELETE)
// - endpoint: Normalized URI path
// - status_code: Resulting HTTP status code
// - user_agent: Client User-Agent header
crate::modules::security::record_admin_audit_event(
    &client_ip,
    &method,
    &path,
    status_code,
);
```

---

## 4. End-to-End Verification Test Harness

A dedicated automated test harness is authored at `03-ai-scripts/44-agm-cli-rest-e2e.py`. This cross-platform Python runner executes the entire 5-test matrix with strict invariant verification.

### 4.1 Test Harness Structure (`03-ai-scripts/44-agm-cli-rest-e2e.py`)

```python
#!/usr/bin/env python3
"""
E2E Test Harness: AGM CLI 46-Command & Secure REST Parity Suite
Target: Task 144
"""

import os
import sys
import json
import time
import subprocess
import urllib.request
import urllib.error

API_BASE = "http://127.0.0.1:8045/api"
ADMIN_TOKEN = os.environ.get("AGM_ADMIN_TOKEN", "test_secret_token_144")

def run_agm_cli(args):
    cmd = ["cargo", "run", "--bin", "agm", "--"] + args + ["--json"]
    proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, cwd="src-tauri")
    if proc.returncode != 0:
        raise RuntimeError(f"AGM CLI failed: {proc.stderr}")
    return json.loads(proc.stdout)

def call_rest_api(path, method="GET", body=None):
    url = f"{API_BASE}{path}"
    headers = {
        "Authorization": f"Bearer {ADMIN_TOKEN}",
        "Content-Type": "application/json"
    }
    data = json.dumps(body).encode("utf-8") if body else None
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        return {"success": False, "status": e.code, "error": e.read().decode("utf-8")}
```

---

## 5. Step-by-Step E2E Verification Runbook

Follow this runbook sequentially to verify all 5 test cases.

```
   Step 1: Invariant Check & Sandbox Setup
     │
   Step 2: Test Case 1 - Instance Creation & Sandbox Isolation
     │
   Step 3: Test Case 2 - Prompt Dispatch & Running Detection
     │
   Step 4: Test Case 3 - Account Switching Continuity
     │
   Step 5: Test Case 4 - Compact Tree View Grouping
     │
   Step 6: Test Case 5 - CLI & REST Parity Comparison
     │
   Step 7: Teardown & Protected PID Verification
```

### Step 1: Pre-flight Verification & Safety Gate
1. Scan for running IDE instances: Capture PIDs for the main Antigravity IDE and Cursor. Record these PIDs as **strictly protected**.
2. Verify that the target test repository `scratch/test-repo` is created without a `.git` directory.

### Step 2: Execute Test Case 1 (Instance Creation & Isolation)
1. Execute CLI: `agm instances create test-cli-flow-a-4401 --json`.
2. Inspect output: Verify `success == true`. Extract `data.id` and `data.data_dir`.
3. Verify filesystem isolation:
   - Check that `data_dir/home` exists.
   - Start instance: `agm instances start -i test-cli-flow-a-4401 --repo scratch/test-repo --json`.
   - Verify environment: Confirm child process environment variable `USERPROFILE` (or `HOME`) equals `data_dir/home`.
   - Assert: Global `%USERPROFILE%\.gemini` remains completely unmodified.

### Step 3: Execute Test Case 2 (Prompt Dispatch & Running Detection)
1. Send prompt:  
   `agm prompts send -i test-cli-flow-a-4401 --repo scratch/test-repo "E2E144-TEST2-DISPATCH" --json`
2. Poll running prompts via CLI:  
   `agm prompts running -i test-cli-flow-a-4401 --json`
3. Poll running prompts via REST:  
   `curl -H "Authorization: Bearer <token>" http://127.0.0.1:8045/api/prompts/running?instanceId=test-cli-flow-a-4401`
4. Assert: Both CLI and REST show prompt status as `in_flight` within 500ms of dispatch.

### Step 4: Execute Test Case 3 (Account Switching Continuity)
1. Snapshot current prompt state:  
   `agm prompts backup -i test-cli-flow-a-4401 --json`
2. Perform account rotation:  
   `agm instances switch -i test-cli-flow-a-4401 --account test-acc-02 --json`
3. Verify resume hand-off file:  
   Confirm `scratch/test-repo/.antigravity_resume_task.json` was created with `instance_id == test-cli-flow-a-4401`.
4. Restore prompts:  
   `agm prompts restore -i test-cli-flow-a-4401 --json`
5. Assert: Zero prompt data loss; prompt resumes execution on `test-acc-02`.

### Step 5: Execute Test Case 4 (Compact Tree View Grouping)
1. Dispatch 3 repeated prompts with identical content: `"Check memory leaks and run audit"`.
2. Retrieve prompt tree:  
   `agm prompts tree -i test-cli-flow-a-4401 --json`
3. Inspect tree hierarchy:
   - Confirm identical prompts are condensed into a single grouped parent node.
   - Confirm parent node shows repeat badge count `x3`.
   - Confirm children are nested in `sub_runs` array.

### Step 6: Execute Test Case 5 (CLI & REST Parity Verification)
1. Compare instance listing:
   - CLI: `agm instances list --json`
   - REST: `GET /api/instances`
   - Assert: Exact match on all fields (`id`, `name`, `status`, `data_dir`).
2. Compare prompt tree:
   - CLI: `agm prompts tree -i test-cli-flow-a-4401 --json`
   - REST: `GET /api/prompts/tree?instanceId=test-cli-flow-a-4401`
   - Assert: Deep structural identity between both responses.

### Step 7: Teardown & Protected PID Verification
1. Stop test instance:  
   `agm instances stop -i test-cli-flow-a-4401 --json`
2. Delete test instance:  
   `agm instances delete -i test-cli-flow-a-4401 --purge-storage --yes --json`
3. Final Safety Audit:  
   Verify that all protected main IDE PIDs recorded in Step 1 are still running and unharmed.

---

## 6. Pre-flight Quality Gates & Acceptance Criteria

Prior to marking this subtask complete, the following gates must be met:
1. `cd src-tauri && cargo fmt -- --check`: Zero formatting errors.
2. `cd src-tauri && cargo clippy --all-targets --all-features`: Zero compiler warnings or lint errors.
3. Automated E2E test script `03-ai-scripts/44-agm-cli-rest-e2e.py` passes 5/5 test cases cleanly.
4. No hardcoded absolute paths; strictly relative paths throughout all code and tests.
