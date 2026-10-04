# Specification: Multi-Instance Prompt Execution Status & Liveness Detection Architecture

> **Spec ID:** `118-instance-prompt-execution-status-fix`  
> **Sub-Document:** `01-architecture-spec.md`  
> **Status:** APPROVED & READY FOR IMPLEMENTATION  
> **Domain:** Backend Rust (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/logger.rs`), Frontend React (`src/pages/Instances.tsx`)  
> **Lead Architect:** Spec Subagent 1  
> **Date:** October 2026  

---

## 1. Executive Summary & Problem Classification

### 1.1 The Multi-Instance Bleed & Status Inversion Phenomenon
In modern Antigravity Manager (AGM) multi-instance operating topologies, users run multiple concurrent Antigravity IDE instances:
- **Default Profile (Sequence 1)**: Operating with default User data and home directory paths (`dirs::home_dir()`).
- **Cloned Profile (Sequence 2, e.g., Instance ID `8159`)**: Operating within an isolated sandbox directory tree (`instances/8159/home`, `instances/8159/data`).

Users observed critical false-positive running badges and execution status cross-contamination across instances:
1. **Default Profile** was actively executing **ONLY** `Antigravity-Manager`. However, the AGM UI erroneously marked `spec-builder` and `coding-guidelines` as active or running.
2. **Instance 8159** was actively executing **ONLY** `coding-guidelines`. However, the AGM UI erroneously marked `spec-builder` and `Antigravity-Manager` as running.
3. Projects that were merely opened in the past or present in VSCode/Antigravity `workspaceStorage` were labeled `is_running = true` indefinitely whenever the parent instance process was alive.
4. Queued or backed-up tasks were reported as active running tasks, distorting the liveness telemetry.

### 1.2 The Ground Truth Reality
The physical ground truth in the reported multi-instance environment is unambiguous:
- **Default Profile**: Actively executing **ONLY** `Antigravity-Manager`. `SpecBuilder` is **IDLE**. `coding-guidelines` is **IDLE**.
- **Instance 8159**: Actively executing **ONLY** `coding-guidelines`. `Antigravity-Manager` is **IDLE**. `SpecBuilder` is **IDLE**.
- **Cross-Instance Invariant**: A task or prompt executing in Instance 8159 must **never** influence or mark any project in the Default profile as running, and vice versa.

---

## 2. Comprehensive Root Cause Analysis: The 7 Core Flaws

Through rigorous static analysis and codebase tracing of `repo_db.rs`, `instance.rs`, and `Instances.tsx`, seven distinct root causes were identified that compound into these false-positive states:

### Flaw 1: Worker Key Scoping Without `instance_id`
In `repo_db::is_prompt_running_for_project(project_id, instance_id)`:
```rust
// FLAW 1: Active workers map checked without instance prefix verification
if let Ok(workers) = get_active_agy_workers().lock() {
    for key in workers.keys() {
        if key.contains(project_id) {
            return true;
        }
    }
}
```
- **Mechanism**: Workers in `get_active_agy_workers()` are registered with keys formatted as `"{instance_id}:{repo_path}"` (e.g., `"default:d:/work/coding-guidelines"`).
- **Failure**: When querying liveness for project `"coding-guidelines"` in instance `"8159"`, `key.contains(project_id)` evaluates to `true` if *any* instance has an active worker for that repo. It completely ignores `instance_id`, bleeding running state across all instances.

### Flaw 2: Inverted OR Logic on `not_fully_idle` in `conversation_summaries.db`
In `repo_db::is_prompt_running_for_project`:
```rust
// FLAW 2: Inverted OR logic ignores authoritative idle signal
let (status, not_fully_idle, ws_uris_opt, _last_time_str) = item;
let is_conv_running = not_fully_idle != 0 || status.contains("RUNNING");
```
- **Mechanism**: Antigravity IDE's runtime engine explicitly sets `not_fully_idle = 0` and status to `"CASCADE_RUN_STATUS_IDLE"` (or completed/failed) when a turn finishes.
- **Failure**: Using `|| status.contains("RUNNING")` means if an old or un-updated session still has `"RUNNING"` in its status string while `not_fully_idle == 0`, the OR expression evaluates to `true`. When `not_fully_idle == 0` or status is `IDLE`, `COMPLETED`, `FAILED`, or `CANCELLED`, the session is definitively idle and must **never** evaluate to running.

### Flaw 3: Cloned Instance State & Database Pollution
In `instance::copy_instance_with_options` and `repo_db::clone_instance_repo_rows`:
```rust
// FLAW 3: Cloned rows preserve running state of source instance
conn.execute(
    "INSERT OR IGNORE INTO running_projects
     (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    rusqlite::params![new_id, target_id, name, path, storage, running, detected, updated],
)
```
- **Mechanism**: When cloning instance `default` to instance `8159`, the `running_projects` rows and `active_prompts` rows were copied with `is_running = source.running` and `status = source.status`. Additionally, copying the source data directory copied `conversation_summaries.db` containing in-flight session flags.
- **Failure**: The newly cloned instance immediately inherited `is_running = 1` and `status = 'running'` for projects that were only running in the source instance. Cloned instances must **always** be initialized into an idle, clean state (`is_running = 0`, prompts reset, stale summaries sanitized).

### Flaw 4: Queued and Backed-up Prompts Treated as Running in Tree Hierarchy
In `repo_db::compute_project_conversation_tree`:
```rust
// FLAW 4: Queued/backed_up prompts marked as running
let is_run = is_inst_alive
    && (ap.status == "running"
        || ap.status == "queued"
        || ap.status == "backed_up");
```
Followed immediately by:
```rust
let has_active_conv = conv_nodes.iter().any(|c| c.is_running);
let proj_is_running = is_inst_alive && (has_active_conv || has_active_prompt);
```
- **Mechanism**: Prompts waiting in queue (`queued`) or preserved for continuity (`backed_up`) were given `is_running = true`.
- **Failure**: `has_active_conv` became `true` for projects with queued tasks, marking the entire project `proj_is_running = true` even though zero processes or prompts were actively executing.

### Flaw 5: Lack of Terminal Transition & Indefinite Liveness in Active Prompts
In `repo_db::is_prompt_running_for_project`:
```rust
// FLAW 5: Stale prompts with status='running' never expire or transition
let running_count: usize = conn.query_row(
    "SELECT COUNT(*) FROM active_prompts WHERE ... AND status = 'running'",
    params![project_id, instance_id],
    |r| r.get(0),
).unwrap_or(0);
if running_count > 0 { return true; }
```
- **Mechanism**: If an execution terminated unexpectedly, crashed, or was killed without emitting a terminal IPC callback, its row in `active_prompts` remained `status = 'running'`.
- **Failure**: Without an automatic expiration timeout or reconciliation against live workers and IDE conversation summaries, `running_count > 0` persisted indefinitely, causing permanent false-positive running badges.

### Flaw 6: Frontend Raw Status Override
In `src/pages/Instances.tsx`:
```typescript
// FLAW 6: Frontend ignores backend is_running calculation
const running = data.filter(
    (node) => Boolean(node.is_running) || Boolean(node.conversations?.some((c) => c.is_running || c.status === 'RUNNING'))
);
```
- **Mechanism**: The frontend component inspected `c.status === 'RUNNING'` directly on child conversation nodes.
- **Failure**: Even when the backend correctly evaluated `c.is_running = false` (due to process death or idle signals), the raw text `c.status === 'RUNNING'` in the node bypassed the backend logic and caused the UI to render the project in the active running task section.

### Flaw 7: Unscoped Global Tree Query & Shared Cache Bleed
In `src/pages/Instances.tsx`:
```typescript
// FLAW 7: Query dispatched without instanceId filter
const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
    maxWords: 50,
    onlyRunning: false,
    force: false,
});
```
- **Mechanism**: Invoking `get_project_conversation_tree` without `instanceId` falls back to `instance_id = "all"`.
- **Failure**: The resulting cache key was `tree:all:50:false`. Subsequent queries from isolated instance views received contaminated aggregated results where conversations from other instances leaked into view.

---

## 3. System Architecture & Component Interaction

```
+---------------------------------------------------------------------------------------------------+
|                                     ANTIGRAVITY MANAGER CORE                                      |
|                                                                                                   |
|   +---------------------------------------+             +-------------------------------------+   |
|   |           Default Instance            |             |         8159 Cloned Profile         |   |
|   |   PID: Active (Antigravity-Manager)   |             |     PID: Active (coding-guidelines)   |   |
|   +-------------------+-------------------+             +------------------+------------------+   |
|                       |                                                    |                      |
|                       v                                                    v                      |
|       [Gate 1: Instance Process Liveness]                   [Gate 1: Instance Process Liveness]   |
|       is_instance_running("default") -> true                is_instance_running("8159") -> true   |
|                       |                                                    |                      |
|                       v                                                    v                      |
|       [Gate 2: Active Execution Verification]               [Gate 2: Active Execution Verification] |
|       1. Worker prefix: "default:" only                     1. Worker prefix: "8159:" only        |
|       2. active_prompts WHERE inst="default"                2. active_prompts WHERE inst="8159"   |
|       3. conversation_summaries (Default home)              3. conversation_summaries (8159 home) |
|                       |                                                    |                      |
|                       +--------------------------+-------------------------+                      |
|                                                  |                                                |
|                                                  v                                                |
|                         [Strict Two-Factor Evaluation Equation]                                   |
|                         is_running = is_inst_alive && has_active_execution                        |
|                                                  |                                                |
|            +-------------------------------------+-------------------------------------+          |
|            |                                                                           |          |
|            v                                                                           v          |
|   Project: "Antigravity-Manager"                                              Project: "coding-guidelines"|
|   - Default: ALIVE && ACTIVE -> is_running = true                             - Default: ALIVE && IDLE -> is_running = false |
|   - 8159:    ALIVE && IDLE   -> is_running = false                            - 8159:    ALIVE && ACTIVE -> is_running = true |
|                                                                                                   |
|   Project: "SpecBuilder"                                                                          |
|   - Default: ALIVE && IDLE   -> is_running = false                                                |
|   - 8159:    ALIVE && IDLE   -> is_running = false                                                |
|                                                  |                                                |
|                                                  v                                                |
|                   [Audit Telemetry: log_instance_prompt_audit]                                    |
|                   Emits structured forensic event for every evaluation                            |
+---------------------------------------------------------------------------------------------------+
```

---

## 4. Architectural Invariants

The following invariants are mathematically guaranteed by the architecture and must be preserved across all implementations:

### Invariant 1: Strict Profile Isolation (Default vs 8159)
Under no circumstances may a background worker, active prompt record, or conversation summary belonging to `instance_id = "default"` cause an evaluation in `instance_id = "8159"` to be marked `is_running = true`, and vice versa. Every data structure access must be partitioned by `instance_id`.

### Invariant 2: Ground Truth State Alignment
At steady state:
- Default profile evaluates `is_running = true` **ONLY** for `Antigravity-Manager`. `spec-builder` and `coding-guidelines` are strictly `is_running = false`.
- Instance 8159 evaluates `is_running = true` **ONLY** for `coding-guidelines`. `Antigravity-Manager` and `spec-builder` are strictly `is_running = false`.

### Invariant 3: Backend Condition Logic: Explicit Idle Supremacy
The liveness of any prompt or project is governed by the two-factor rule:
$$\text{is\_running} = \text{is\_inst\_alive} \land \text{has\_active\_execution}$$

Where:
1. If $\text{is\_inst\_alive} = \text{false}$, $\text{is\_running}$ is **strictly FALSE**.
2. If `not_fully_idle == 0` OR status matches any of `["IDLE", "COMPLETED", "FAILED", "CANCELLED", "CASCADE_RUN_STATUS_IDLE"]`, then $\text{has\_active\_execution}$ is **strictly FALSE**. Explicit idle/terminal signals strictly override any timestamp recency heuristic or stale `"RUNNING"` string fragment.
3. $\text{has\_active\_execution}$ is `true` **IF AND ONLY IF**:
   - There is a verified live OS worker PID in `get_active_agy_workers()` whose key starts with `"{instance_id}:"`, OR
   - There is an active record in `active_prompts` with `status = 'running'` updated within the active liveness TTL (300s) and without terminal transition, OR
   - The instance's authoritative `conversation_summaries.db` contains a row matching the workspace with `not_fully_idle != 0` AND status not containing `IDLE`/`COMPLETED`/`FAILED`/`CANCELLED`.

### Invariant 4: Worker Scoping in `get_active_agy_workers`
Active worker checks in `is_prompt_running_for_project` must verify that the worker key begins with the exact instance ID:
```rust
let expected_prefix = format!("{}:", instance_id);
let has_live_worker = workers.iter().any(|(key, &pid)| {
    if !key.starts_with(&expected_prefix) {
        return false;
    }
    // Verify OS process liveness for worker PID
    is_pid_alive(pid) && key.contains(project_id)
});
```

### Invariant 5: Cloning Sanitization & State Reset Protocol
When creating a cloned instance via `copy_instance_with_options` or `clone_instance_repo_rows`:
1. Cloned rows in `running_projects` must have `is_running = 0`.
2. Cloned prompts in `active_prompts` must never be in `status = 'running'` or `status = 'dispatched'`. They must be sanitized to `'completed'`, `'idle'`, or omitted.
3. Destination `conversation_summaries.db` (if copied) must be sanitized so all session records have `not_fully_idle = 0` and status `'IDLE'`. A clone must never inherit in-flight execution state.

### Invariant 6: Strict Separation of Queued/Backed-Up vs Active Running
In `compute_project_conversation_tree` and all status aggregators:
- `status == "queued"` represents a pending, non-executing prompt.
- `status == "backed_up"` represents a preserved snapshot for restart continuity.
- Neither `queued` nor `backed_up` may ever set `is_running = true` or `c.is_running = true`. They must be badged as `QUEUED` / `BACKED_UP` and flagged `is_running = false`.

### Invariant 7: Verbose Structured Audit Logging Protocol
Every execution liveness evaluation must emit a structured audit log using `crate::modules::logger::log_instance_prompt_audit`. This creates a traceable log line:
```text
[InstancePromptAudit] instance='8159' project='coding-guidelines' path='d:/work/coding-guidelines' is_instance_active=true is_running=true active_tasks=1 rationale='Active conversation summary with not_fully_idle=1'
```
The rationale must clearly describe the exact condition that triggered the state (e.g., `INSTANCE_DEAD`, `EXPLICIT_IDLE_NOT_FULLY_IDLE_ZERO`, `MATCHED_INSTANCE_WORKER`, `ACTIVE_PROMPT_DB`).

---

## 5. Detailed Component Architecture

### 5.1 Module: `src-tauri/src/modules/logger.rs`
The logger provides the structured audit emission helper:
```rust
/// Emit structured audit log for instance prompt and project liveness evaluation
pub fn log_instance_prompt_audit(
    instance_id: &str,
    project_name: &str,
    repo_path: &str,
    is_instance_active: bool,
    is_running: bool,
    active_tasks: usize,
    rationale: &str,
) {
    info!(
        "[InstancePromptAudit] instance='{}' project='{}' path='{}' is_instance_active={} is_running={} active_tasks={} rationale='{}'",
        instance_id, project_name, repo_path, is_instance_active, is_running, active_tasks, rationale
    );
}
```

### 5.2 Module: `src-tauri/src/modules/repo_db.rs`
Modifications required in `repo_db.rs`:

1. **`is_prompt_running_for_project(project_id: &str, instance_id: &str) -> bool`**:
   - Filter `get_active_agy_workers()` by `key.starts_with(&format!("{}:", instance_id))`.
   - Purge dead PIDs from workers map if found.
   - Enforce terminal transition check: if `status IN ('completed', 'failed')`, ignore older running counts.
   - In `conversation_summaries.db`: check `not_fully_idle != 0 && !status.contains("IDLE") && !status.contains("COMPLETED") && !status.contains("FAILED") && !status.contains("CANCELLED")`.
   - Log decision via `log_instance_prompt_audit`.

2. **`compute_project_conversation_tree`**:
   - Set `let is_run = is_inst_alive && ap.status == "running";` (strictly exclude `"queued"` and `"backed_up"`).
   - In conversation node mapping, ensure `is_running = is_run`.
   - Calculate project-level `proj_is_running` only from true running tasks or active conversations.

3. **`clone_repo_rows_on`**:
   - In `INSERT INTO running_projects`, force `is_running = 0`.
   - In `INSERT INTO active_prompts`, sanitize `status`:
     ```rust
     let sanitized_status = match status.as_str() {
         "running" | "dispatched" => "completed",
         other => other,
     };
     ```

### 5.3 Module: `src-tauri/src/modules/instance.rs`
Modifications required in `instance.rs`:

1. **`copy_instance_with_options` & `copy_instance_projects`**:
   - Ensure `clone_instance_repo_rows` is invoked with proper target sanitization.
   - After copying directory trees, if destination contains `conversation_summaries.db`, execute SQLite sanitization:
     ```sql
     UPDATE conversation_summaries 
     SET not_fully_idle = 0, status = 'IDLE' 
     WHERE not_fully_idle != 0 OR status LIKE '%RUNNING%';
     ```
   - Ensure cloned projects start completely clean and idle.

### 5.4 Module: `src/pages/Instances.tsx`
Modifications required in frontend:

1. **Pass `instanceId` to `get_project_conversation_tree`**:
   - Scopes cache and queries directly to the active or selected instance.
2. **Eliminate Raw Status Override**:
   - Replace:
     ```typescript
     const running = data.filter(
         (node) => Boolean(node.is_running) || Boolean(node.conversations?.some((c) => c.is_running || c.status === 'RUNNING'))
     );
     ```
   - With:
     ```typescript
     const running = data.filter(
         (node) => Boolean(node.is_running) || Boolean(node.conversations?.some((c) => c.is_running))
     );
     ```
   - Trust the backend's verified `is_running` boolean instead of checking arbitrary status strings.

---

## 6. Forensic Traceability & Audit Matrix

| Evaluation Target | Instance | Process Alive | Worker Key | DB Summary State | Final `is_running` | Audit Log Rationale |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `Antigravity-Manager` | `default` | `true` | `default:d:/work/AGM` | `not_fully_idle=1`, `RUNNING` | `true` | `ACTIVE_WORKER_AND_CONV_RUNNING` |
| `coding-guidelines` | `default` | `true` | `None` | `not_fully_idle=0`, `IDLE` | `false` | `EXPLICIT_IDLE_NOT_FULLY_IDLE_ZERO` |
| `SpecBuilder` | `default` | `true` | `None` | `None` (empty) | `false` | `NO_ACTIVE_TASKS_OR_CONVERSATIONS` |
| `coding-guidelines` | `8159` | `true` | `8159:d:/work/CG` | `not_fully_idle=1`, `RUNNING` | `true` | `ACTIVE_WORKER_AND_CONV_RUNNING` |
| `Antigravity-Manager` | `8159` | `true` | `None` | `not_fully_idle=0`, `IDLE` | `false` | `EXPLICIT_IDLE_NOT_FULLY_IDLE_ZERO` |
| `SpecBuilder` | `8159` | `true` | `None` | `None` (empty) | `false` | `NO_ACTIVE_TASKS_OR_CONVERSATIONS` |
| Any Project | Any | `false` | Ignored | Ignored | `false` | `INSTANCE_PROCESS_DEAD` |
| Cloned Project | `target` | `true` | `None` | Sanitized (`IDLE`) | `false` | `CLONED_STATE_SANITIZED_IDLE` |

---

## 7. Verification & Proof Scenarios

1. **Default vs 8159 Isolation Verification**:
   - Start Antigravity IDE on Default profile running `Antigravity-Manager`.
   - Start Antigravity IDE on Instance 8159 running `coding-guidelines`.
   - Call `is_prompt_running_for_project("coding-guidelines", "default")` -> MUST BE `false`.
   - Call `is_prompt_running_for_project("Antigravity-Manager", "8159")` -> MUST BE `false`.
   - Call `is_prompt_running_for_project("Antigravity-Manager", "default")` -> MUST BE `true`.
   - Call `is_prompt_running_for_project("coding-guidelines", "8159")` -> MUST BE `true`.
2. **Process Death Invariant Verification**:
   - Kill process for Instance 8159.
   - Immediate re-check -> All projects in Instance 8159 MUST BE `false`.
3. **Cloning Sanitization Verification**:
   - Execute clone of Default instance with active task to a new instance `test_clone`.
   - Inspect `running_projects` and `active_prompts` in DB for `test_clone` -> `is_running == 0` and no active running prompts.
4. **Audit Log Inspection**:
   - Verify `[InstancePromptAudit]` lines appear in `app.log` with correct instance tagging and descriptive rationale.
