---
plan: code-dryness-and-guard-inversion
subtask: "02"
title: Nested If Flattening and Guard Clause Inversion across Core Backend Modules
domain: backend-rust
depends_on: none
citations:
  component_spec: ../../../../02-spec/21-app/code-dryness-and-guard-inversion/02-component-spec.md
  master_plan: ../../code-dryness-100-steps-master-plan.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/repo_db/goal_heartbeat.rs
  - src-tauri/src/modules/repo_db/detection.rs
  - src-tauri/src/modules/repo_db/tree_collect.rs
  - src-tauri/src/modules/telegram_inbound/injection.rs
  - src-tauri/src/modules/telegram_inbound/send_flow.rs
  - src-tauri/src/modules/instance/pid_scan.rs
status: pending
---

# Subtask 02: Nested If Flattening and Guard Clause Inversion across Core Backend Modules

## 1. Context & Motivation
Deeply nested conditional blocks ("Pyramid of Doom") and monolithic control flows degrade codebase maintainability and increase cognitive load. Specifically:
1. `src-tauri/src/modules/repo_db/goal_heartbeat.rs` nests 8 levels deep when inspecting prompt goal heartbeats.
2. `src-tauri/src/modules/repo_db/detection.rs` nests 7 levels deep when parsing workspace storage directories.
3. `src-tauri/src/modules/repo_db/tree_collect.rs` nests 6 levels deep inside SQLite summaries database iteration.
4. `src-tauri/src/modules/telegram_inbound/injection.rs` bundles massive local and remote dispatch implementations inside a single 200+ line `if/else` block.
5. `src-tauri/src/modules/telegram_inbound/send_flow.rs` embeds nested loops with manual outer-loop break flags to search conversation trees.
6. `src-tauri/src/modules/instance/pid_scan.rs` duplicates a 10-level ancestor PID traversal across default and instance detection branches and inlines IDE marker checks.

This subtask inverts conditions into early exits (`return;` / `continue;`), applies `let ... else { continue; }`, and extracts clean, focused helper functions, reducing maximum nesting to 1 level.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Functions | Refactoring Action | Target Nesting Depth |
| :--- | :--- | :--- | :---: |
| `src-tauri/src/modules/repo_db/goal_heartbeat.rs` | `inspect_prompt_goal_status`, `check_heartbeat_freshness` | Extract `check_heartbeat_freshness`, flatten loop using `let ... else { continue; }` | 1 level |
| `src-tauri/src/modules/repo_db/detection.rs` | `detect_running_projects` | Invert `storage_dir.exists()`, flatten workspace loop with early `continue` guards | 1 level |
| `src-tauri/src/modules/repo_db/tree_collect.rs` | `collect_tree_conversation_items` | Flatten SQLite connection, statement, and query handling using `let ... else { continue; }` | 1 level |
| `src-tauri/src/modules/telegram_inbound/injection.rs` | `execute_prompt_injection`, `execute_remote_prompt_injection` | Invert `is_current_machine` with early return, extract remote dispatch into `execute_remote_prompt_injection` | 1 level |
| `src-tauri/src/modules/telegram_inbound/send_flow.rs` | `handle_turn_rerun_callback`, `find_conversation_in_tree` | Extract `find_conversation_in_tree` helper, eliminate outer break flags in rerun callback | 1 level |
| `src-tauri/src/modules/instance/pid_scan.rs` | `find_pids_for_data_dir`, `has_ide_markers`, `has_ancestor_in` | Extract `has_ide_markers` and `has_ancestor_in`, deduplicate 10-level ancestor traversal | 1 level |

---

## 3. Step-by-Step Implementation Guide

### 3.1 Step 1: Flatten `inspect_prompt_goal_status` in `goal_heartbeat.rs`
1. Open `src-tauri/src/modules/repo_db/goal_heartbeat.rs`.
2. Above `inspect_prompt_goal_status`, define private helper `check_heartbeat_freshness`:
   ```rust
   fn check_heartbeat_freshness(
       cfg: &PromptGoalHeartbeatConfig,
       now: i64,
   ) -> (bool, Option<String>) {
       let hb_p = Path::new(&cfg.heartbeat_file);
       if !hb_p.exists() {
           return (false, None);
       }
       let is_fresh = fs::metadata(hb_p)
           .ok()
           .and_then(|meta| meta.modified().ok())
           .map(|modified| {
               let dt: chrono::DateTime<Utc> = modified.into();
               let age = now - dt.timestamp();
               age <= (cfg.interval_secs as i64 + 4)
           })
           .unwrap_or(false);

       let last_line = fs::read_to_string(hb_p).ok().and_then(|content| {
           content
               .lines()
               .rev()
               .find(|l| !l.trim().is_empty())
               .map(|s| s.to_string())
       });

       (is_fresh, last_line)
   }
   ```
3. Refactor `inspect_prompt_goal_status`:
   - Replace nested `if cfg_p.exists()` and `if let Ok(...)` with `if !cfg_p.exists() { continue; }`.
   - Use `let Ok(c) = fs::read_to_string(&cfg_p) else { continue; };`.
   - Use `let Ok(cfg) = serde_json::from_str::<PromptGoalHeartbeatConfig>(&c) else { continue; };`.
   - Call `check_heartbeat_freshness(&cfg, now)`.
   - Return `(is_fresh, Some(cfg.heartbeat_file), last_line)` immediately.

### 3.2 Step 2: Flatten `detect_running_projects` Storage Loop in `detection.rs`
1. Open `src-tauri/src/modules/repo_db/detection.rs`.
2. Locate `detect_running_projects` (lines 53–125).
3. Invert the storage directory check:
   ```rust
   if !storage_dir.exists() {
       invalidate_prompt_tree_cache(Some(target_id));
       return Ok(projects);
   }

   let Ok(entries) = fs::read_dir(&storage_dir) else {
       invalidate_prompt_tree_cache(Some(target_id));
       return Ok(projects);
   };
   ```
4. In the `for entry in entries.flatten()` loop, flatten using early continues:
   - `if !ws_folder.is_dir() { continue; }`
   - `let ws_json_path = ws_folder.join("workspace.json");`
   - `if !ws_json_path.exists() { continue; }`
   - `let Ok(content) = fs::read_to_string(&ws_json_path) else { continue; };`
   - `let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) else { continue; };`
   - `let Some(uri) = folder_uri else { continue; };`
5. Keep project parsing and logging at root indentation.

### 3.3 Step 3: Flatten `collect_tree_conversation_items` Loop in `tree_collect.rs`
1. Open `src-tauri/src/modules/repo_db/tree_collect.rs`.
2. In the `for (owning_inst_id, base) in &candidate_dirs` loop (lines 79–147):
   - Guard missing database: `if !summaries_db.exists() { continue; }`
   - Guard connection acquisition: `let Ok(s_conn) = Connection::open_with_flags(...) else { continue; };`
   - Guard statement preparation: `let Ok(mut stmt) = s_conn.prepare(...) else { continue; };`
   - Guard query mapping: `let Ok(rows) = stmt.query_map(...) else { continue; };`
3. Unindent the `for item in rows.flatten()` loop body to depth 1.

### 3.4 Step 4: Decompose `execute_prompt_injection` in `injection.rs`
1. Open `src-tauri/src/modules/telegram_inbound/injection.rs`.
2. Extract lines 271–373 into a standalone async function `execute_remote_prompt_injection`:
   ```rust
   pub async fn execute_remote_prompt_injection(
       target_node: &str,
       target_project: &str,
       prompt_text: &str,
       explicit_instance: Option<&str>,
       local_config: &supabase_sync::SupabaseSyncConfig,
   ) -> String
   ```
3. In `execute_prompt_injection`, replace the monolithic `if/else` with an inverted guard:
   ```rust
   if !super::node_label::is_current_machine(&target_node) {
       return execute_remote_prompt_injection(
           &target_node,
           &target_project,
           &prompt_text,
           explicit_instance.as_deref(),
           &local_config,
       )
       .await;
   }
   ```
4. Unindent the local prompt dispatch logic (resolving dual sequence IDs, creating active prompt, spawning via agy) to the root level of `execute_prompt_injection`.

### 3.5 Step 5: Flatten `handle_turn_rerun_callback` in `send_flow.rs`
1. Open `src-tauri/src/modules/telegram_inbound/send_flow.rs`.
2. Define helper `find_conversation_in_tree`:
   ```rust
   fn find_conversation_in_tree(
       tree: &[crate::modules::repo_db::models::ProjectConversationItem],
       cid8: &str,
   ) -> Option<(
       crate::modules::repo_db::models::ProjectConversationItem,
       crate::modules::repo_db::models::ConversationItem,
   )> {
       let cid8_lower = cid8.to_lowercase();
       for proj in tree {
           for conv in &proj.conversations {
               if conv.short_id.eq_ignore_ascii_case(cid8)
                   || conv.conversation_id.to_lowercase().starts_with(&cid8_lower)
                   || conv.seq_code.eq_ignore_ascii_case(cid8)
               {
                   return Some((proj.clone(), conv.clone()));
               }
           }
       }
       None
   }
   ```
3. In `handle_turn_rerun_callback`, replace lines 850–871 with:
   ```rust
   let tree = crate::modules::repo_db::get_project_conversation_tree(200, false);
   let (proj, conv) = match find_conversation_in_tree(&tree, cid8) {
       Some(m) => m,
       None => { ... fallback logic ... }
   };
   ```

### 3.6 Step 6: Linearize PID Ancestor Traversal in `pid_scan.rs`
1. Open `src-tauri/src/modules/instance/pid_scan.rs`.
2. Extract `has_ide_markers(exe: &str, name: &str) -> bool`:
   - Checks Electron resources directory, language server binaries, or `antigravity.exe` / `antigravity` names.
3. Extract `has_ancestor_in(start_pid: u32, parent_map: &HashMap<u32, u32>, target_set: &HashSet<u32>, max_depth: usize) -> bool`:
   - Climbs `parent_map` up to `max_depth` (10 iterations) and returns `true` if `target_set.contains(&curr)`.
4. In `find_pids_for_data_dir`:
   - Use `has_ide_markers` when checking `is_default_candidate`.
   - Use `!has_ancestor_in(cand_pid, &parent_map, &instance_root_pids, 10)` for default candidate validation.
   - Use `has_ancestor_in(parent, &parent_map, &matched_set, 10)` for instance child process validation.

---

## 4. Verification & Quality Gates

Run the local pre-flight checks after code implementation:
1. **Formatting Gate**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
2. **Clippy Gate**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
3. **Targeted Module Unit Tests**:
   ```bash
   cd src-tauri && cargo test --package antigravity-manager --lib modules::telegram_inbound
   cd src-tauri && cargo test --package antigravity-manager --lib modules::instance
   cd src-tauri && cargo test --package antigravity-manager --lib modules::repo_db
   ```

---

## 5. Acceptance Criteria Checklist
- [ ] `inspect_prompt_goal_status` nesting reduced from 8 levels to 1 level with `check_heartbeat_freshness`.
- [ ] `detect_running_projects` storage loop nesting reduced from 7 levels to 1 level with inverted guards.
- [ ] `collect_tree_conversation_items` summaries loop nesting reduced from 6 levels to 1 level.
- [ ] `execute_prompt_injection` unindented by inverting `is_current_machine` and extracting `execute_remote_prompt_injection`.
- [ ] `handle_turn_rerun_callback` eliminates nested search loops via `find_conversation_in_tree`.
- [ ] `find_pids_for_data_dir` ancestor traversal deduplicated via `has_ancestor_in` and `has_ide_markers`.
- [ ] `cargo fmt -- --check` passes with zero formatting diffs.
- [ ] `cargo clippy --all-targets --all-features` passes with zero warnings.
