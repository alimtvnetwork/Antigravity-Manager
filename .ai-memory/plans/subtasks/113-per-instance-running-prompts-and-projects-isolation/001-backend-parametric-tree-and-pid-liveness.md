---
plan: 113-per-instance-running-prompts-and-projects-isolation
subtask: "001"
title: Parametric Project Conversation Tree & Strict PID Liveness Verification
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/113-per-instance-running-prompts-and-projects-isolation.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../../02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md
  parent_plan: ../../113-per-instance-running-prompts-and-projects-isolation.md
target_files:
  - src-tauri/src/commands/instance.rs
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/modules/instance.rs
status: pending
---

# 001 — Parametric Project Conversation Tree & Strict PID Liveness Verification

## 1. Context & Problem Statement
Currently, `get_project_conversation_tree` aggregates all projects across every instance indiscriminately and relies on a blind 10-minute timestamp recency window (`age < 600`) to mark conversations and projects as `[RUNNING]`. This causes severe state bleed across instances:
1. When viewing an individual instance card (e.g. `8159`), projects from the `default` instance (e.g., `Antigravity-Manager`, `spec-builder`) appear inside `8159` with false `[RUNNING]` badges.
2. If an instance has been terminated or its OS process is dead, conversations modified within the last 10 minutes continue to report `is_running = true`.
3. The IPC command `get_project_conversation_tree` lacks an `instance_id` filter parameter, and `prompt_tree_cache` in `repo_prompts.db` caches a single global tree key (`tree:{max_words}:{only_running}`) under `instance_id = 'all'`.

## 2. Target Files and Symbols
- `src-tauri/src/commands/instance.rs`:
  - `get_project_conversation_tree(instance_id: Option<String>, max_words: Option<usize>, only_running: Option<bool>, force: Option<bool>)`
- `src-tauri/src/modules/repo_db.rs`:
  - `get_project_conversation_tree(max_words: usize, only_running: bool) -> Vec<AgmProjectTreeNode>` (legacy wrapper)
  - `get_project_conversation_tree_cached(instance_id: Option<&str>, max_words: usize, only_running: bool, force: bool) -> Vec<AgmProjectTreeNode>`
  - `compute_project_conversation_tree(instance_id: Option<&str>, max_words: usize, only_running: bool) -> Vec<AgmProjectTreeNode>`
  - `prompt_tree_cache` cache key formatting and query persistence
- `src-tauri/src/modules/instance.rs`:
  - `is_instance_running(instance_id: &str, data_dir: &str, config_pid: Option<u32>) -> bool`
  - `find_pids_for_data_dir(data_dir: &str, is_default: bool) -> Vec<u32>`

## 3. Concrete Implementation Steps

### Step 3.1: Parametric IPC Command in `src-tauri/src/commands/instance.rs`
1. Update `get_project_conversation_tree` signature to accept `instance_id: Option<String>`:
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
2. Maintain backward compatibility for existing callers omitting `instance_id`.

### Step 3.2: Partition `prompt_tree_cache` by Instance in `src-tauri/src/modules/repo_db.rs`
1. Update `get_project_conversation_tree_cached` to accept `instance_id: Option<&str>`.
2. Format the partitioned cache key:
   ```rust
   let inst_key = instance_id.unwrap_or("all");
   let cache_key = format!("tree:{}:{}:{}", inst_key, max_words, only_running);
   ```
3. Update SQLite lookup and upsert into `prompt_tree_cache`:
   - Query `SELECT tree_json, updated_at, ttl_seconds FROM prompt_tree_cache WHERE cache_key = ?1` using the partitioned `cache_key`.
   - On write, insert `inst_key` into the `instance_id` column of `prompt_tree_cache`.

### Step 3.3: Strict Process PID Liveness Enforcement in `compute_project_conversation_tree`
1. Update `compute_project_conversation_tree(instance_id: Option<&str>, max_words: usize, only_running: bool)`.
2. Evaluate process liveness per instance using `crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid)`.
3. Eliminate the blind 10-minute recency assumption:
   - Previously: `let is_conv_running = not_fully_idle != 0 || status.contains("RUNNING") || is_recency_active;`
   - New rule: Recency (`is_recency_active`) alone MUST NOT mark a conversation or project as running if the instance process is dead (`!is_inst_alive`).
   - If `!is_inst_alive`, both conversation `is_running` and project `is_running` MUST be strictly `false`.
   - If `is_inst_alive`, conversation `is_running` is `true` if and only if there is an active in-flight prompt in `active_prompts` table OR the conversation summary indicates active execution (`not_fully_idle != 0 || status.contains("RUNNING")`).

### Step 3.4: Per-Instance WorkspaceStorage Scoping
1. When `instance_id` is supplied (`Some(target_id)`):
   - Locate the target instance in the registry (or default instance if target is `default`/`__default__`).
   - Only execute `detect_running_projects(target_id)` against that instance's `data_dir/User/workspaceStorage`.
   - Strictly filter `projects` to those belonging to `target_id`. Do NOT include projects from any other instance.
   - Restrict conversation transcripts and summaries scanning to folders whose resolved path matches projects discovered in that instance's `workspaceStorage`.
2. When `instance_id` is `None` (or `"all"`):
   - Retain full-fleet aggregation for CLI tree formatters and global views, but still enforce PID liveness on every individual instance node.

## 4. Architectural Constraints & Coding Standards
- **Coding Guidelines Compliance**:
  - Follow `02-spec/02-coding-guidelines/` naming: boolean variables and parameters must use positive prefixes (`is_`, `has_`).
  - No double negatives; never compare booleans explicitly with `== true` or `== false`.
  - Maintain function size tiers: keep helper functions decomposed (under 15-25 lines).
  - Use `strutil` or safe path comparison (`normalize_path_for_compare`) for Windows/Unix path normalization.
- **Cross-Platform Compatibility**:
  - Support Windows, macOS, and Linux path separators and process discovery.
- **No Speculative Mutations**:
  - Do not alter unassociated database schemas or command signatures outside the instance/repo_db boundary.

## 5. Out of Scope
- Frontend UI modifications in `Instances.tsx` or `PromptTreeViewModal.tsx` (delegated to Subtask 003).
- End-to-end integration test authoring and RCA documentation (delegated to Subtasks 002 and 004).
- Version bump, changelog generation, or release execution.

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::repo_db
cd src-tauri && cargo test commands::instance
```

## 7. Done When
- [ ] `get_project_conversation_tree` IPC command accepts `instance_id: Option<String>`.
- [ ] `prompt_tree_cache` stores partitioned entries under `tree:{instance_id}:{max_words}:{only_running}`.
- [ ] Supplying `instance_id` returns strictly the projects located in that instance's `workspaceStorage`.
- [ ] If an instance process is dead, projects and conversations for that instance are guaranteed `is_running = false`.
- [ ] Blind 10-minute recency fallback is eliminated from marking dead instance conversations as active.
- [ ] Cargo fmt, clippy, and unit tests compile and pass cleanly without warnings.

## 8. Ambiguities & Fallback Defaults
- When `instance_id` is omitted or `None`, the backend defaults to fleet-wide aggregation (`all`), preserving compatibility with existing CLI callers (`agm tree`) while applying strict per-node PID liveness.
- For legacy unassigned conversations (`__unassigned__`), they are excluded from instance-scoped queries unless explicitly attached to the instance's known workspace path.
