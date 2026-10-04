---
plan: 114-debug-instance-prompt-running-detection
subtask: "01"
title: Backend Detection Refactor & Isolated Per-Instance Prompt Liveness
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/114-debug-instance-prompt-running-detection/01-architecture-spec.md
  task_spec: ../../../../02-spec/21-app/114-debug-instance-prompt-running-detection.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/logger.rs
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/commands/instance.rs
status: pending
---

# Subtask 01 — Backend Detection Refactor & Isolated Per-Instance Prompt Liveness

## 1. Objective & Scope

Resolve the cross-instance running prompt state bleed between Default profile and secondary profile `8159`:
1. Ensure the Default profile only detects running projects that are actually running on Default (`Antigravity-Manager`), marking idle projects (`spec-builder`, `coding-guidelines`) as idle.
2. Ensure the `8159` profile only detects running projects that are actually running on `8159` (`coding-guidelines`), marking idle projects (`spec-builder`, `Antigravity-Manager`) as idle.
3. Eliminate the blind 600-second timestamp recency heuristic (`is_recency_active`) when Antigravity explicitly reports `not_fully_idle == 0` or `"CASCADE_RUN_STATUS_IDLE"`.
4. Decouple historical workspaces in `workspaceStorage` from live process execution in `detect_running_projects`.
5. Implement strict tagged candidate directory resolution (`gemini_dirs_tagged`) and compound-key conversation grouping (`convs_by_inst_and_path`).
6. Partition the `prompt_tree_cache` by instance ID and wire structured audit logging `[InstancePromptAudit]`.

---

## 2. Target Files & Symbol Breakdown

| File Target | Key Symbols & Functions | Action |
| :--- | :--- | :--- |
| `src-tauri/src/modules/logger.rs` | `log_instance_prompt_audit` | Add structured audit logger emitting `[InstancePromptAudit]` |
| `src-tauri/src/modules/repo_db.rs` | `gemini_dirs_for_instance`, `gemini_dirs_tagged` | Implement strict per-instance path resolution and tagged tuples |
| `src-tauri/src/modules/repo_db.rs` | `is_prompt_running_for_project` | Eliminate `is_recency_active` override on explicit idle; scope to instance |
| `src-tauri/src/modules/repo_db.rs` | `detect_running_projects` | Check affirmative active task; do not blindly mark historical workspaces |
| `src-tauri/src/modules/repo_db.rs` | `compute_project_conversation_tree` | Use `gemini_dirs_tagged` and `convs_by_inst_and_path` `(instance_id, repo_path)` |
| `src-tauri/src/modules/repo_db.rs` | `get_project_conversation_tree_cached` | Accept `instance_id: Option<&str>` and partition cache key `tree:{inst}:{w}:{r}` |
| `src-tauri/src/commands/instance.rs` | `get_project_conversation_tree` | Ensure IPC command passes `instance_id.as_deref()` to cached retriever |

---

## 3. Step-by-Step Implementation Tasks

### Step 3.1: Add Structured Audit Logging in `src-tauri/src/modules/logger.rs`
1. Open `src-tauri/src/modules/logger.rs`.
2. Add public function `log_instance_prompt_audit`:
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
3. Ensure no lint errors or missing imports.

### Step 3.2: Implement `gemini_dirs_for_instance` & `gemini_dirs_tagged` in `src-tauri/src/modules/repo_db.rs`
1. Locate `gemini_dirs_for_instance(instance_id: &str)` in `src-tauri/src/modules/repo_db.rs`.
2. Refactor to ensure:
   - If `instance_id` is `"default"`, `"__default__"`, or empty: resolves exclusively from `dirs::home_dir()`.
   - If `instance_id` is a specific instance ID: resolves exclusively from `crate::modules::instance::get_instance_home_dir(instance_id)`.
   - Iterates over subdirectories `["antigravity", "antigravity-cli", "antigravity-ide"]`.
3. Implement `gemini_dirs_tagged(instance_id: Option<&str>) -> Vec<(String, PathBuf)>`:
   ```rust
   pub fn gemini_dirs_tagged(instance_id: Option<&str>) -> Vec<(String, PathBuf)> {
       let mut tagged = Vec::new();
       let target = instance_id.unwrap_or("all");

       if target != "all" {
           let norm_id = if target == "__default__" || target.is_empty() {
               "default"
           } else {
               target
           };
           for dir in gemini_dirs_for_instance(norm_id) {
               tagged.push((norm_id.to_string(), dir));
           }
           return tagged;
       }

       // For "all": tag default directories
       for dir in gemini_dirs_for_instance("default") {
           tagged.push(("default".to_string(), dir));
       }

       // Tag each registered secondary instance
       if let Ok(reg) = crate::modules::instance::load_registry() {
           for inst in reg.instances {
               if !inst.is_default && inst.id != "default" {
                   for dir in gemini_dirs_for_instance(&inst.id) {
                       tagged.push((inst.id.clone(), dir));
                   }
               }
           }
       }

       tagged
   }
   ```

### Step 3.3: Eliminate `is_recency_active` Override in `src-tauri/src/modules/repo_db.rs`
1. In `is_prompt_running_for_project`:
   - Inspect status evaluation against `conversation_summaries.db`.
   - Replace any logic that overrides `not_fully_idle == 0` or `"CASCADE_RUN_STATUS_IDLE"`.
   - Formulate strict boolean check:
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
2. Apply the same strict idle rule inside `compute_project_conversation_tree` when scanning rows from `conversation_summaries.db`.
3. Ensure that recency (`age < 600`) is never used to flip an explicitly idle conversation to `RUNNING`.

### Step 3.4: Decouple `workspaceStorage` from Indiscriminate Liveness in `detect_running_projects`
1. In `detect_running_projects(instance_id: &str)`:
   - Identify each workspace folder found in `data_dir/User/workspaceStorage`.
   - Calculate `is_project_active`:
     ```rust
     let is_project_active = is_instance_active && is_prompt_running_for_project(&raw_path, target_id);
     ```
   - Build rationale string and active task count.
   - Invoke `crate::modules::logger::log_instance_prompt_audit`:
     ```rust
     crate::modules::logger::log_instance_prompt_audit(
         target_id,
         &repo_name,
         &raw_path,
         is_instance_active,
         is_project_active,
         if is_project_active { 1 } else { 0 },
         if is_project_active {
             "Active prompt detected in instance runtime"
         } else if !is_instance_active {
             "Instance OS process is not alive"
         } else {
             "No active prompt found; workspace is idle"
         },
     );
     ```
   - Persist accurate `is_running = is_project_active` into `running_projects` table.

### Step 3.5: Isolate Conversations via `convs_by_inst_and_path` in `compute_project_conversation_tree`
1. In `compute_project_conversation_tree(instance_id: Option<&str>, max_words: usize, only_running: bool)`:
   - Obtain tagged candidate directories using `gemini_dirs_tagged(instance_id)`.
   - Replace `convs_by_path: HashMap<String, ...>` with:
     ```rust
     let mut convs_by_inst_and_path: std::collections::HashMap<
         (String, String),
         Vec<(String, String, String, String, bool, usize, String, String)>,
     > = std::collections::HashMap::new();
     ```
   - For each candidate directory `(inst_id, base_dir)`:
     - Read conversation summaries.
     - Determine `is_conv_running` using the strict idle rule.
     - For each assigned path `p_key`, insert into:
       ```rust
       convs_by_inst_and_path.entry((inst_id.clone(), p_key)).or_default().push((
           cid.clone(),
           title.clone(),
           effective_prompt.clone(),
           status.clone(),
           is_conv_running,
           steps,
           last_time_str.clone(),
           inst_id.clone(),
       ));
       ```
2. When iterating over `projects` to create `AgmProjectTreeNode`:
   - Filter projects to `target_id` if `instance_id` was specified and not `"all"`.
   - Look up conversations strictly matching `&(proj.instance_id.clone(), norm_path)`.
   - Evaluate `node.is_running`:
     - If the owning instance has no alive OS process, `node.is_running = false`.
     - Otherwise, `node.is_running = true` if any matched conversation or active prompt for that instance has `is_conv_running == true`.
     - Emit `log_instance_prompt_audit` for the tree node.

### Step 3.6: Partition Cache in `get_project_conversation_tree_cached`
1. Update signature:
   ```rust
   pub fn get_project_conversation_tree_cached(
       instance_id: Option<&str>,
       max_words: usize,
       only_running: bool,
       force: bool,
   ) -> Vec<AgmProjectTreeNode>
   ```
2. Compute partitioned cache key:
   ```rust
   let inst_key = instance_id.unwrap_or("all");
   let cache_key = format!("tree:{}:{}:{}", inst_key, max_words, only_running);
   ```
3. Update query and upsert to bind `inst_key` to column `instance_id`.
4. Provide legacy wrapper for existing 2-parameter callers:
   ```rust
   pub fn get_project_conversation_tree(
       max_words: usize,
       only_running: bool,
   ) -> Vec<AgmProjectTreeNode> {
       get_project_conversation_tree_cached(None, max_words, only_running, false)
   }
   ```

### Step 3.7: Align IPC Command in `src-tauri/src/commands/instance.rs`
1. Verify `get_project_conversation_tree`:
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

## 4. Verification & Acceptance Criteria

- [ ] **Candidate Resolution Isolation**:
  - `gemini_dirs_tagged(Some("default"))` yields only default paths tagged `"default"`.
  - `gemini_dirs_tagged(Some("8159"))` yields only `8159` paths tagged `"8159"`.
  - No directory path from `8159` is ever tagged `"default"`.
- [ ] **Compound Map Isolation**:
  - Conversations from `8159` are keyed under `("8159", path)`.
  - Conversations from Default are keyed under `("default", path)`.
  - Querying Default tree nodes never reads or includes `("8159", path)` entries.
- [ ] **Strict Idle Signal Enforcement**:
  - When `not_fully_idle == 0` or status is `"CASCADE_RUN_STATUS_IDLE"`, `is_conv_running` is strictly `false`.
  - Timestamp recency (`age < 600`) never flips an explicit idle state to `true`.
- [ ] **WorkspaceStorage Decoupling**:
  - An inactive workspace present in `User/workspaceStorage` has `is_running = false`.
  - Only the workspace with an affirmative active prompt or turn has `is_running = true`.
- [ ] **Cache Partitioning**:
  - `prompt_tree_cache` keys follow `tree:{instance_id}:{max_words}:{only_running}`.
  - Querying `8159` after `default` results in a separate cache lookup and distinct response.
- [ ] **Structured Audit Logging**:
  - Every evaluation logs `[InstancePromptAudit]` with `instance`, `project`, `is_running`, and `rationale`.
- [ ] **Quality Gates**:
  - Strictly no git commands run during subagent execution.
  - Rust formatting and Clippy compatibility preserved.
