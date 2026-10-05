---
plan: 135-instance-restart-button-sync-icons-and-running-projects-fix
subtask: "03"
title: Running Projects & Prompts Detection Deep Fix
domain: backend-rust-sqlite-tauri-frontend
depends_on:
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
citations:
  component_spec: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  root_cause_analysis: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
  coding_guidelines: 02-spec/02-coding-guidelines/readme.md
  error_management: 02-spec/03-error-manage/readme.md
target_files:
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/modules/email_watcher.rs
  - src/pages/Instances.tsx
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# Subtask 03: Running Projects & Prompts Detection Deep Fix

## 1. Context & Objectives

Users identified recurring anomalies where projects falsely showed as running when the IDE process was dead or crashed, or falsely flipped to idle while modern thinking models were generating reasoning chains for several minutes. Additionally, cross-instance project bleed, idle sensor notification suppression, and 5-second TTL cache lag compromised operational reliability.

This subtask delivers an exhaustive, multi-tier root-cause resolution across Rust backend modules (`repo_db.rs`, `instance.rs`, `email_watcher.rs`) and React components (`Instances.tsx`, `PromptTreeViewModal.tsx`) addressing all 6 structural defects:

1. **Defect 1 (Sandbox Home Collision in `gemini_dirs_for_instance`)**: Prevent named profiles from falling back to the user home directory (`dirs::home_dir()`), maintaining strict sandbox path isolation.
2. **Defect 2 (String Mismatch in Gate 4)**: Implement two-tier path and repository basename/ID matching so that absolute workspace paths in `conversation_summaries.db` match repository names, workspace hashes, and composite keys.
3. **Defect 3 (Double-Tagging and False Alive via `is_antigravity_running(None)`)**: Replace global OS process checks with strict per-instance PID verification (`find_pids_for_data_dir`) across Gate 0, Gate 4, and `compute_project_conversation_tree`.
4. **Defect 4 (Adaptive 10-Minute Thinking Window & Fractional Parser)**: Expand the Gate 4 recency cutoff from 120s to 600s (10 minutes) for active reasoning models, support ISO 8601 microseconds and SQL datetimes, and strictly enforce idle supremacy.
5. **Defect 5 (False Positive in `is_any_prompt_actively_running` Blocking `email_watcher.rs`)**: Remove the blind IDE process check that conflated open editors with active prompts, restoring idle sensor telemetry alerts.
6. **Defect 6 (Cache Invalidation Contract on Lifecycle Transitions)**: Call `invalidate_prompt_tree_cache` in `close_instance`, `launch_instance`, and `save_or_requeue_prompt`, update frontend `fetchRunningTasks` to pass `{ force: true }`, and reset zombie database flags (`UPDATE running_projects SET is_running = 0`) on startup.

---

## 2. Target Files & Symbol Breakdown

### 2.1 `src-tauri/src/modules/repo_db.rs`
- [`gemini_dirs_for_instance`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L805-L825):
  - Strictly resolve named profiles via `get_instance_home_dir(instance_id)`. If unresolvable, return an empty vector rather than falling back to `dirs::home_dir()`.
- [`gemini_dirs_tagged`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L827-L860):
  - Ensure candidate directories are uniquely tagged and never double-scanned.
- [`is_prompt_running_for_project`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L1654-L1995):
  - Gate 0: Validate instance-specific PID liveness; if dead, return `false` and log `INSTANCE_PROCESS_DEAD`.
  - Gate 4: Implement two-tier path matching (`clean_p == clean_target || path_basename == target_basename || is_prefix`).
  - Gate 4: Use `parse_flexible_timestamp` to handle microseconds and ISO formats.
  - Gate 4: Expand thinking window to 600s (`now - conv_time <= 600`) when `not_fully_idle > 0` and status is `RUNNING`.
  - Gate 4: Enforce strict idle supremacy (`not_fully_idle == 0` or terminal status).
  - Gate 4: Prune 0-word untitled ghost sessions.
- [`compute_project_conversation_tree`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L3995-L4350):
  - Replace `is_antigravity_running(None)` calls with `find_pids_for_data_dir(&default_dir, true)`.
- [`is_any_prompt_actively_running`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L2660-L2728):
  - Remove blind IDE process check (`find_pids_for_data_dir`) that erroneously blocked `email_watcher.rs`.
- [`save_or_requeue_prompt`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L2731-L2830):
  - Call `invalidate_prompt_tree_cache(Some(&canonical_inst))` on prompt write.
- [`purge_corrupted_running_projects`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L310-L330):
  - Add `UPDATE running_projects SET is_running = 0` and `DELETE FROM prompt_tree_cache` on startup.

### 2.2 `src-tauri/src/modules/instance.rs`
- [`close_instance`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/instance.rs#L3920-L4190):
  - Invoke `crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id))` upon process termination.
- [`launch_instance_inner_with_extra_workspaces`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/instance.rs#L3127-L3683):
  - Invoke `crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id))` upon successful launch.

### 2.3 `src-tauri/src/modules/email_watcher.rs`
- Line 393: Audit `is_any_prompt_actively_running()` call site to ensure idle telemetry alerts fire correctly when all projects are idle.

### 2.4 `src/pages/Instances.tsx`
- [`fetchRunningTasks`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L279-L296):
  - Support `options: { force?: boolean }`, passing `force: isForce` across IPC to bypass `prompt_tree_cache`.
  - Pass `{ force: true }` in `handleLaunch`, `handleStop`, and `handleRestart`.
- [`isNodeOwnedByInstance`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L102-L130):
  - Enforce exact instance ID matching (`node.instance_id === instConfig.id`), eliminating loose suffix matching (`instConfig.id.endsWith(...)`).

### 2.5 `src/components/instances/PromptTreeViewModal.tsx`
- Ensure modal mount and `handleRestore` invoke `loadTree(true, true)` to force fresh data from SQLite.

---

## 3. Granular Step-by-Step Implementation

### Step 3.1: Fix Defect 1 (Sandbox Home Collision) in `repo_db.rs`
1. Locate `gemini_dirs_for_instance(instance_id: &str)`:
   ```rust
   pub fn gemini_dirs_for_instance(instance_id: &str) -> Vec<PathBuf> {
       let named = instance_id != "all"
           && instance_id != "default"
           && !instance_id.is_empty()
           && instance_id != "__default__";

       let home = if named {
           crate::modules::instance::get_instance_home_dir(instance_id).ok()
       } else {
           dirs::home_dir()
       };

       let mut dirs = Vec::new();
       if let Some(home) = home {
           for sub in ["antigravity", "antigravity-ide", "antigravity-cli"] {
               let path = home.join(".gemini").join(sub);
               if path.exists() {
                   dirs.push(path);
               }
           }
       }
       dirs
   }
   ```
2. Ensure named instances never fall back to `dirs::home_dir()`. If `get_instance_home_dir` fails, `home` remains `None`, returning `Vec::new()`.

### Step 3.2: Fix Defect 2 (Two-Tier Path & Basename Matching) in Gate 4
1. In `is_prompt_running_for_project`:
   ```rust
   let clean_target = normalize_path_for_compare(project_id);
   let target_basename = Path::new(project_id)
       .file_name()
       .map(|n| n.to_string_lossy().to_lowercase())
       .unwrap_or_else(|| {
           project_id
               .split("__")
               .next()
               .unwrap_or(project_id)
               .to_lowercase()
       });
   ```
2. In Gate 4 workspace evaluation:
   ```rust
   let clean_p = normalize_path_for_compare(&decode_uri_to_path(&u));
   let path_basename = Path::new(&clean_p)
       .file_name()
       .map(|n| n.to_string_lossy().to_lowercase())
       .unwrap_or_default();

   let is_direct_match = !clean_target.is_empty() && clean_p == clean_target;
   let is_basename_match = !target_basename.is_empty() && path_basename == target_basename;
   let is_subpath_match = clean_target.starts_with(&clean_p) || clean_p.starts_with(&clean_target);

   if is_direct_match || is_basename_match || is_subpath_match {
       // Confirmed active turn for target project!
       return true;
   }
   ```

### Step 3.3: Fix Defect 3 (Eliminate `is_antigravity_running(None)`)
1. In `compute_project_conversation_tree` (lines 4059, 4143, 4281):
   - Replace `crate::modules::process::is_antigravity_running(None)` with:
     ```rust
     let is_default_alive = {
         let default_dir = crate::modules::instance::get_default_antigravity_data_dir();
         let pids = crate::modules::instance::find_pids_for_data_dir(&default_dir.to_string_lossy(), true);
         !pids.is_empty()
     };
     ```
   - This ensures the default instance is only reported alive if an Antigravity process with the default user data directory is actively running.

### Step 3.4: Fix Defect 4 (Adaptive 10-Minute Thinking Window & Fractional Parser)
1. In `repo_db.rs`, implement `parse_flexible_timestamp(raw: &str) -> i64`:
   - Supports `%Y-%m-%dT%H:%M:%S%.f%:z`, RFC 3339, `%Y-%m-%d %H:%M:%S%.f`, `%Y-%m-%d %H:%M:%S`, and epoch integers.
2. In Gate 4:
   ```rust
   let conv_time = parse_flexible_timestamp(&_last_time_str);
   let is_recent = conv_time > 0 && (now - conv_time <= 600); // 10-minute adaptive window
   ```
3. Enforce strict idle supremacy:
   ```rust
   let is_idle_count = not_fully_idle == 0;
   let has_idle_status = status.contains("IDLE")
       || status.contains("COMPLETED")
       || status.contains("FAILED")
       || status.contains("CANCELLED");
   let is_explicit_idle = is_idle_count || has_idle_status;
   if is_explicit_idle {
       continue;
   }
   ```

### Step 3.5: Fix Defect 5 (Unblock `email_watcher.rs` in `is_any_prompt_actively_running`)
1. In `is_any_prompt_actively_running()` in `repo_db.rs`:
   - Delete lines 2719–2726 (the blind `find_pids_for_data_dir` process check).
   - Only return `true` if `conversation_summaries.db` has non-idle running rows or `active_prompts` has running/queued rows.
2. Verify that when all conversations and queues are idle, `is_any_prompt_actively_running()` returns `false`, allowing `email_watcher.rs` to send idle alerts.

### Step 3.6: Fix Defect 6 (Cache Invalidation Contract & Startup Purge)
1. In `src-tauri/src/modules/instance.rs`:
   - At the end of `close_instance(instance_id)`:
     ```rust
     crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
     ```
   - At the end of `launch_instance_inner_with_extra_workspaces`:
     ```rust
     crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
     ```
2. In `src-tauri/src/modules/repo_db.rs`:
   - In `save_or_requeue_prompt`:
     ```rust
     invalidate_prompt_tree_cache(Some(&canonical_inst));
     ```
   - In `purge_corrupted_running_projects`:
     ```rust
     let _ = conn.execute("UPDATE running_projects SET is_running = 0", []);
     let _ = conn.execute("DELETE FROM prompt_tree_cache", []);
     ```
3. In `src/pages/Instances.tsx`:
   - Update `fetchRunningTasks` signature: `const fetchRunningTasks = async (options: { force?: boolean } = {}) => { ... }`.
   - Pass `options.force ?? false` to `get_project_conversation_tree`.
   - Call `fetchRunningTasks({ force: true })` inside `handleLaunch`, `handleStop`, `handleRestart`, and on window focus.

---

## 4. Verification Checklist

- [ ] **Defect 1**: Calling `gemini_dirs_for_instance` on named instances never returns paths in user home `~/.gemini`.
- [ ] **Defect 2**: `is_prompt_running_for_project` matches projects whether passed an absolute path, repo basename, or workspace hash.
- [ ] **Defect 3**: Closing the default instance while a secondary instance is running results in default instance reporting dead (`is_alive = false`).
- [ ] **Defect 4**: Prompts generating reasoning chains for >120s retain `is_running: true` up to 600s as long as idle status is not set; fractional timestamps parse correctly.
- [ ] **Defect 5**: An open but idle IDE does not prevent `email_watcher.rs` from detecting idleness.
- [ ] **Defect 6**: Stopping or launching an instance immediately clears the prompt tree cache; frontend reflects the new state without 5-second lag; startup resets zombie `is_running = 1` rows to 0.
