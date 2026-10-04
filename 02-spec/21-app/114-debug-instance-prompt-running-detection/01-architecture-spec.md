# Specification: Strict Per-Instance Prompt & Running Detection Architecture

> **Spec ID:** `114-debug-instance-prompt-running-detection`  
> **Sub-Document:** `01-architecture-spec.md`  
> **Status:** APPROVED & READY FOR IMPLEMENTATION  
> **Domain:** Backend Rust (`src-tauri/src/modules/repo_db.rs`, `src-tauri/src/commands/instance.rs`, `src-tauri/src/modules/logger.rs`)  
> **Date:** October 2026  

---

## 1. Executive Summary & Root Cause Analysis

### 1.1 The Multi-Instance Bleed Phenomenon
In multi-profile Antigravity setups (e.g., Sequence 1 Default Profile and Sequence 2 Cloned Profile `8159`), users observed severe false-positive running states across instances:
1. **Default Profile** was actively running only `Antigravity-Manager`. However, the AGM UI erroneously marked `spec-builder` and `coding-guidelines` as `[RUNNING]`.
2. **Cloned Profile (8159)** was actively running only `coding-guidelines`. However, the AGM UI erroneously marked `spec-builder` and `Antigravity-Manager` as `[RUNNING]`.
3. Inactive projects that were merely opened in the past or present in VSCode/Antigravity `workspaceStorage` were labeled `is_running = true` indefinitely as long as the parent instance process was alive.

### 1.2 The Five Underlying Root Causes
An exhaustive code inspection revealed five compounding design flaws:

1. **Untagged Global Directory Scanning**:  
   `compute_project_conversation_tree` queried `crate::modules::agy_cleaner::get_gemini_candidate_dirs()`. This returned all candidate directories across all home folders without tagging which instance owned which directory. All conversations were aggregated into a global pool and assigned the hardcoded instance label `"default"`.

2. **Path-Only Conversation Mapping (`convs_by_path`)**:  
   Discovered conversations were stored in a single `HashMap<String, Vec<...>>` keyed solely by normalized repository path (`repo_path`). If both Default and `8159` instances opened or referenced the same repo or a project with the same path, their conversation trees and execution states merged into each other.

3. **Erroneous `is_recency_active` (Age < 600s) Override**:  
   In both `is_prompt_running_for_project` and `compute_project_conversation_tree`, the code checked:
   ```rust
   let is_recency_active = if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&last_time_str) {
       let age = (Utc::now() - parsed.with_timezone(&Utc)).num_seconds();
       age >= 0 && age < 600
   } else {
       false
   };
   let is_conv_running = not_fully_idle != 0 || status.contains("RUNNING") || is_recency_active;
   ```
   Antigravity IDE explicitly writes `not_fully_idle = 0` and status `"CASCADE_RUN_STATUS_IDLE"` (or completed/failed) when a turn finishes. The heuristic `is_recency_active` blindly overrode this explicit signal, forcing conversations to be considered `RUNNING` for 10 full minutes after completion.

4. **Indiscriminate WorkspaceStorage Liveness in `detect_running_projects`**:  
   `detect_running_projects` enumerated all folders in `data_dir/User/workspaceStorage`. If the instance process was alive (`is_instance_active`), and if `is_prompt_running_for_project` returned `true` (due to the recency override or cross-instance conversation bleed), every historical workspace ever opened in that instance was stamped `is_running = true`.

5. **Unpartitioned Global Cache Key**:  
   `get_project_conversation_tree_cached` ignored `instance_id` and cached all tree data under the global key `format!("tree:{}:{}", max_words, only_running)`. A cache query from instance `8159` would immediately receive the cached tree generated for `default`, and vice versa.

---

## 2. Architectural Design & Invariants

```
+-----------------------------------------------------------------------------------------------+
|                                    Antigravity Manager Gateway                                |
|                                                                                               |
|   +------------------------------------+             +------------------------------------+   |
|   |         Default Instance           |             |         8159 Cloned Profile        |   |
|   |  Home: dirs::home_dir()            |             |  Home: instances/8159/home         |   |
|   |  Data: Default AppData             |             |  Data: instances/8159/data         |   |
|   |  PID: 10420 (Antigravity-Manager)  |             |  PID: 22108 (coding-guidelines)   |   |
|   +-----------------+------------------+             +-----------------+------------------+   |
|                     |                                                  |                      |
|                     v                                                  v                      |
|      gemini_dirs_tagged(Some("default"))              gemini_dirs_tagged(Some("8159"))        |
|      [("default", ~/.gemini/antigravity)]             [("8159", .../8159/home/.gemini)]       |
|                     |                                                  |                      |
|                     +------------------------+-------------------------+                      |
|                                              |                                                |
|                                              v                                                |
|                   convs_by_inst_and_path: HashMap<(instance_id, repo_path), Vec<Conv>>        |
|                                              |                                                |
|               +------------------------------+------------------------------+                 |
|               |                                                             |                 |
|               v                                                             v                 |
|      Key: ("default", "Antigravity-Manager")                       Key: ("8159", "coding-guidelines") 
|      not_fully_idle: 1, RUNNING -> is_running = true               not_fully_idle: 1, RUNNING -> is_running = true
|                                                                                               |
|      Key: ("default", "coding-guidelines")                         Key: ("8159", "Antigravity-Manager")
|      not_fully_idle: 0, IDLE -> is_running = false                 not_fully_idle: 0, IDLE -> is_running = false
|                                                                                               |
|      Key: ("default", "spec-builder")                              Key: ("8159", "spec-builder")
|      not_fully_idle: 0, IDLE -> is_running = false                 not_fully_idle: 0, IDLE -> is_running = false
|                                              |                                                |
|                                              v                                                |
|                         prompt_tree_cache: "tree:{instance_id}:{max_words}:{only_running}"    |
|                                              |                                                |
|                                              v                                                |
|                  Structured Audit Log: [InstancePromptAudit] via log_instance_prompt_audit    |
+-----------------------------------------------------------------------------------------------+
```

### 2.1 Invariant 1: Strict Per-Instance Candidate Directory Resolution
Directories are resolved deterministically based on the requested instance ID:
- `gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf>`:
  - If `instance_id` is `"default"` or `"__default__"` or empty: resolves exclusively from `dirs::home_dir()`.
  - If `instance_id` is a specific instance (e.g., `"8159"`): resolves exclusively from `crate::modules::instance::get_instance_home_dir(instance_id)`.
  - Traverses `.gemini/antigravity`, `.gemini/antigravity-cli`, and `.gemini/antigravity-ide`.
- `gemini_dirs_tagged(instance_id: Option<&str>) -> Vec<(String, PathBuf)>`:
  - Returns pairs of `(instance_id, candidate_path)`.
  - When `instance_id` is `Some(id)` (and `id != "all"`): tags all resolved paths with `id`.
  - When `instance_id` is `None` or `Some("all")`: tags default paths with `"default"` and each registered instance's paths with `inst.id`.
  - **Guarantee**: No database file is ever read without an explicit, verifiable instance ownership tag.

### 2.2 Invariant 2: Isolated Conversation Mapping via `convs_by_inst_and_path`
Conversations are grouped using a compound key:
```rust
let mut convs_by_inst_and_path: HashMap<
    (String, String),
    Vec<(String, String, String, String, bool, usize, String, String)>,
> = HashMap::new();
```
- Key: `(instance_id, normalized_repo_path)`
- When reading `conversation_summaries.db` from candidate directory `(inst_id, base_dir)`:
  - Discovered conversations belong strictly to `inst_id`.
  - Assigned repository paths map into `(inst_id.clone(), p_key)`.
- When assembling `AgmProjectTreeNode` for a project belonging to `proj.instance_id`:
  - Looks up strictly `convs_by_inst_and_path.get(&(proj.instance_id.clone(), norm_path))`.
  - Cross-instance bleeding is mathematically prevented at the map key level.

### 2.3 Invariant 3: Elimination of Erroneous `is_recency_active` Override
Antigravity IDE's runtime engine provides authoritative state in `conversation_summaries.db`:
- `not_fully_idle != 0`: Turn execution is currently processing.
- `status.contains("RUNNING")`: Session is actively dispatching or waiting for model response.
- `not_fully_idle == 0` AND (`status.contains("IDLE")` OR `status.contains("COMPLETED")` OR `status.contains("FAILED")` OR `status.contains("CANCELLED")`): Turn is strictly idle.

**The Strict Idle Rule**:
If `not_fully_idle == 0` or status explicitly indicates an idle/terminal state, the conversation is **IDLE**. The heuristic `is_recency_active` (`age < 600`) MUST NOT override an explicit idle signal.
```rust
let is_explicit_idle = not_fully_idle == 0 
    || status.contains("IDLE") 
    || status.contains("COMPLETED") 
    || status.contains("FAILED") 
    || status.contains("CANCELLED");

let is_conv_running = if is_explicit_idle {
    false
} else {
    not_fully_idle != 0 || status.contains("RUNNING")
};
```
Recency is only permitted as a fallback when status and idle flags are completely null, missing, or indeterminate, and even then, only if the OS process for that instance is actively running.

### 2.4 Invariant 4: Decoupled Workspace Storage & Active Task Verification
In `detect_running_projects(instance_id: &str)`:
1. `workspaceStorage` reflects historical workspaces opened in the editor across the lifetime of the profile.
2. Even if the editor process is running (`is_instance_active == true`), an individual workspace is only `is_running = true` if:
   - There is an active, uncompleted task in `active_prompts` table with `status = 'running'` for that exact `(instance_id, repo_path)`, OR
   - The instance's `conversation_summaries.db` has an active non-idle conversation (`is_conv_running == true`) whose workspace path matches that repository.
3. If no active task or non-idle conversation matches the project path, `is_running` is strictly `false`.
4. Stale historical records in `running_projects` table are updated to `is_running = 0`.

### 2.5 Invariant 5: Partitioned Caching for `prompt_tree_cache`
In `get_project_conversation_tree_cached`:
```rust
let inst_key = instance_id.unwrap_or("all");
let cache_key = format!("tree:{}:{}:{}", inst_key, max_words, only_running);
```
- Query:
  ```sql
  SELECT tree_json, updated_at, ttl_seconds 
  FROM prompt_tree_cache 
  WHERE cache_key = ?1 AND instance_id = ?2
  ```
- Upsert:
  ```sql
  INSERT INTO prompt_tree_cache (cache_key, instance_id, tree_json, project_count, conversation_count, updated_at, ttl_seconds)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, 60)
  ON CONFLICT(cache_key) DO UPDATE SET
     instance_id = excluded.instance_id,
     tree_json = excluded.tree_json,
     project_count = excluded.project_count,
     conversation_count = excluded.conversation_count,
     updated_at = excluded.updated_at,
     ttl_seconds = excluded.ttl_seconds
  ```
- Cache entries for `8159` will never conflict with or serve `default` or `all`.

### 2.6 Invariant 6: Structured Audit Logging `[InstancePromptAudit]`
To provide instant forensic visibility without modifying production flow:
```rust
crate::modules::logger::log_instance_prompt_audit(
    instance_id,
    &repo_name,
    &repo_path,
    is_instance_active,
    is_running,
    active_tasks_count,
    &rationale,
);
```
Outputs standardized log line:
`[InstancePromptAudit] instance='8159' project='coding-guidelines' path='d:/work/coding-guidelines' is_instance_active=true is_running=true active_tasks=1 rationale='Active conversation summary with not_fully_idle=1'`

---

## 3. Detailed Component Architecture

### 3.1 Module: `src-tauri/src/modules/logger.rs`
Add dedicated helper:
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

### 3.2 Module: `src-tauri/src/modules/repo_db.rs`
1. **Refactor `gemini_dirs_for_instance` & Implement `gemini_dirs_tagged`**:
   - `gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf>`
   - `gemini_dirs_tagged(instance_id: Option<&str>) -> Vec<(String, PathBuf)>`
2. **Refactor `is_prompt_running_for_project`**:
   - Scope queries strictly to `instance_id`.
   - In SQLite `active_prompts`: filter strictly by `instance_id`.
   - In `conversation_summaries.db`: check `not_fully_idle != 0 || status.contains("RUNNING")`. If `not_fully_idle == 0` or status contains `"IDLE"`, return `false`.
   - Emit `log_instance_prompt_audit` explaining whether the project is running or idle.
3. **Refactor `detect_running_projects`**:
   - For each workspace in `workspaceStorage`: check `is_prompt_running_for_project(&raw_path, target_id)`.
   - If `!is_instance_active`, force `is_running = false`.
   - Update SQLite `running_projects` table accordingly.
4. **Refactor `compute_project_conversation_tree` & `get_project_conversation_tree_cached`**:
   - Signature:
     ```rust
     pub fn get_project_conversation_tree_cached(
         instance_id: Option<&str>,
         max_words: usize,
         only_running: bool,
         force: bool,
     ) -> Vec<AgmProjectTreeNode>
     ```
   - Partition cache by `instance_id`.
   - Scope `candidate_dirs` via `gemini_dirs_tagged(instance_id)`.
   - Populate `convs_by_inst_and_path`.
   - Match conversations strictly by `(proj.instance_id, norm_path)`.

### 3.3 Module: `src-tauri/src/commands/instance.rs`
Ensure Tauri IPC command passes `instance_id.as_deref()`:
```rust
#[tauri::command]
pub fn get_project_conversation_tree(
    instance_id: Option<String>,
    max_words: Option<usize>,
    only_running: Option<bool>,
    force: Option<bool>,
) -> Result<Vec<crate::modules::repo_db::AgmProjectTreeNode>, String> {
    Ok(
        crate::modules::repo_db::get_project_conversation_tree_cached(
            instance_id.as_deref(),
            max_words.unwrap_or(200),
            only_running.unwrap_or(false),
            force.unwrap_or(false),
        ),
    )
}
```

---

## 4. Verification Scenarios & Expected Outcomes

| Scenario | Instance Target | Physical Execution State | Expected AGM UI Badging | Audit Log Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **A** | Default Profile | Running `Antigravity-Manager` | `Antigravity-Manager`: `[RUNNING]`<br>`spec-builder`: `[IDLE]`<br>`coding-guidelines`: `[IDLE]` | `not_fully_idle=1` on `Antigravity-Manager`; `not_fully_idle=0` on others |
| **B** | 8159 Profile | Running `coding-guidelines` | `coding-guidelines`: `[RUNNING]`<br>`spec-builder`: `[IDLE]`<br>`Antigravity-Manager`: `[IDLE]` | `not_fully_idle=1` on `coding-guidelines`; others not executing in 8159 |
| **C** | Default Profile | Terminated (PID dead) | All projects: `[IDLE]` | `is_instance_active=false`; process PID check failed |
| **D** | Cache Query | Query 8159 after Default | Returns 8159 nodes only | Partitioned cache hit: `tree:8159:200:false` |

---

## 5. Architectural Non-Negotiables
1. **Total Isolation**: No conversation row read from instance A's directory shall ever be attached to instance B's project node.
2. **Explicit Status Supremacy**: Explicit runtime idle signals (`not_fully_idle == 0` or `"CASCADE_RUN_STATUS_IDLE"`) strictly override any timestamp recency heuristics.
3. **No Phantom Workspaces**: Historical presence in `workspaceStorage` does not constitute active execution.
4. **Structured Auditability**: All liveness decisions must emit `[InstancePromptAudit]` logs with clear rationale.
