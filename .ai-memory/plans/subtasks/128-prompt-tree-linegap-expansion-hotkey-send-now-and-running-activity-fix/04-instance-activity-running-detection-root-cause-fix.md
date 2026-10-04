---
plan: 128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix
subtask: "04"
title: Instance Activity Running Detection Root Cause Fix & Database Cache Purge
domain: backend-rust-sqlite-tauri
depends_on:
  - 01-architecture-spec.md
  - 02-component-spec.md
  - 03-root-cause-analysis.md
citations:
  app_spec: 02-spec/21-app/128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix/02-component-spec.md
  root_cause_analysis: 02-spec/21-app/128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix/03-root-cause-analysis.md
  coding_guidelines: 02-spec/02-coding-guidelines/readme.md
  error_management: 02-spec/03-error-manage/readme.md
target_files:
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/modules/instance.rs
  - src/pages/Instances.tsx
status: pending
---

# Subtask 04: Instance Activity Running Detection Root Cause Fix & Database Cache Purge

## 1. Context & Problem Definition

On the main Instances page (`src/pages/Instances.tsx`) and inside `PromptTreeViewModal.tsx`, the default instance falsely displayed the project `white-presentation-v1` as `[RUNNING]` when only Antigravity-Manager was open and the IDE was completely idle or opened to another project.

Investigation revealed three compounding backend failures:
1. **56 Corrupted Rows in `running_projects`**: Legacy scripts had inserted 56 rows with `workspace_storage_path = NULL` and un-namespaced project IDs, which permanently retained `is_running = 1`.
2. **Stale Serialization in `prompt_tree_cache`**: The cached tree JSON under key `tree:all:50:false` froze `white-presentation-v1` in a running state, which was repeatedly returned to the UI by `fetchRunningTasks()` and `loadTree(true, false)`.
3. **Loose Fallback SQL Traps**: SQL queries in `repo_db.rs` evaluated `(?2 = 'default' AND (instance_id = 'default' OR instance_id IS NULL OR instance_id = ''))`, falsely attributing un-namespaced or orphaned running tasks to the default instance.
4. **Unguarded Active AGY Workers**: Gate 2 in `is_prompt_running_for_project` checked whether *any* worker PID was alive without verifying whether that worker was bound to the target workspace.

---

## 2. Target Files & Symbols

- `src-tauri/src/modules/repo_db.rs`:
  - `init_tables`: Add startup migration to purge corrupted rows (`workspace_storage_path IS NULL`) and clear `prompt_tree_cache`.
  - `purge_corrupted_running_projects`: Dedicated cleanup helper.
  - `is_prompt_running_for_project`: Eliminate loose `IS NULL` / empty string SQL fallbacks in Gate 3; scope Gate 2 worker check strictly to workspace.
  - `compute_project_conversation_tree`: Eliminate fallback that marks empty workspace running without explicit workspace directory existence and active memory prompt matching.
  - `resend_running_commands_for_instance`: Guard `UPDATE running_projects SET is_running = 1` against un-namespaced IDs.
- `src-tauri/src/modules/instance.rs`:
  - `find_pids_for_data_dir`: Verify that default instance PID matching strictly excludes helper processes and ensures valid host process binding.
- `src/pages/Instances.tsx`:
  - `fetchRunningTasks`: Invalidate prompt tree cache or pass proper parameters so stale running badges are never displayed.

---

## 3. Granular Implementation Steps

### Step 3.1: Database Schema Startup Purge Migration in `repo_db.rs`
1. In `init_tables(conn: &Connection)`, execute a one-time database migration:
   ```sql
   DELETE FROM running_projects WHERE workspace_storage_path IS NULL OR trim(workspace_storage_path) = '';
   DELETE FROM prompt_tree_cache;
   ```
2. Log the number of purged rows using `crate::modules::logger::log_info`.
3. Ensure index `idx_running_projects_inst_running` is created on `running_projects(instance_id, is_running)`.

### Step 3.2: Eliminate Permissive SQL Fallback Traps in `repo_db.rs`
1. Review all occurrences of:
   ```sql
   (?2 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id IS NULL OR instance_id = ''))
   ```
2. Replace with strict instance matching:
   ```sql
   (?2 = 'default' AND (instance_id = 'default' OR instance_id = '__default__'))
   ```
3. Enforce that only rows explicitly tagged with `'default'` or `'__default__'` are matched.

### Step 3.3: Guard Empty Workspace Liveness in `compute_project_conversation_tree`
1. When `conv_nodes.is_empty()` is true, check:
   - Does `workspace_storage_path` exist on the filesystem? If not, unconditionally evaluate as `is_running: false`.
   - Is there an in-memory active prompt updated within the last 60 seconds?
   - Is there an active worker process registered specifically for this project path whose OS process is alive?
2. If neither condition is met, mark project strictly as `is_running: false` with rationale `"IDLE_EMPTY_WORKSPACE"`.

### Step 3.4: Guard `resend_running_commands_for_instance` SQL Updates
1. When updating `running_projects` in `resend_running_commands_for_instance`:
   - Only update rows where `id = ?1` (the exact composite ID `format!("{}__{}", project_id, inst_suffix)`).
   - Never update un-namespaced project IDs (`id = ?2`) that could bleed into other instances or default scopes.

### Step 3.5: Verify Process Liveness in `instance.rs`
1. Ensure `find_pids_for_data_dir` for default instance strictly checks processes whose executable path matches `Antigravity` or whose command line targets the default data directory.
2. If no host process is running, all default instance projects must immediately report `is_running: false`.

---

## 4. Constraints & Coding Guidelines

- **Zero Tolerance for Explicit Booleans**: Never write `is_running == true`; use `is_running` directly.
- **Strict Error Handling**: Wrap all SQLite calls in typed `Result` or `map_err`; never swallow errors silently.
- **US English Spelling**: All identifiers and comments must use standard US English.
- **Canonical Size Tiers**: Keep helper functions bounded (under 40 lines); extract reusable database operations into focused functions.

---

## 5. Verification & Done Criteria

- [ ] Startup migration deletes all 56 corrupted rows where `workspace_storage_path IS NULL`.
- [ ] `prompt_tree_cache` is wiped on startup, clearing the stale `tree:all:50:false` JSON.
- [ ] With Antigravity-Manager running and IDE idle, `white-presentation-v1` displays strictly as **IDLE** on both the Instances page and Prompt Tree modal.
- [ ] `cd src-tauri && cargo clippy --all-targets --all-features` passes with zero warnings.
- [ ] Unit tests in `src-tauri/src/modules/repo_db.rs` pass cleanly.
