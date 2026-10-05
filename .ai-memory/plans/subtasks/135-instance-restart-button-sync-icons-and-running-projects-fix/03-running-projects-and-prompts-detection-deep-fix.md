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
  - src/pages/Instances.tsx
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# Subtask 03: Running Projects & Prompts Detection Deep Fix

## 1. Context & Objectives

Users identified recurrent bugs where projects falsely showed as running when the IDE process was dead or crashed, or falsely flipped to idle while modern thinking models were generating reasoning chains for several minutes. Additionally, cross-instance project bleed and zombie flags lingering from previous application sessions compromised UI accuracy.

This subtask delivers an exhaustive, multi-layer root-cause resolution across Rust backend modules (`repo_db.rs`, `instance.rs`) and React components (`Instances.tsx`, `PromptTreeViewModal.tsx`):
1. **Host Process PID Liveness Enforcement (Gate 0)**: In `detect_running_projects` and `is_instance_running`, strictly verify that the target instance has living OS processes before permitting any project to be marked active.
2. **Adaptive 10-Minute Thinking Window**: Replace the rigid 60s/120s timeout in Gate 4 with an adaptive 600s window to support extended reasoning models (o1, o3-mini, Claude 3.7 Thinking, Gemini 2.5 Flash Thinking), while respecting explicit idle indicators.
3. **Multi-Format Fractional Timestamp Parser**: Support ISO 8601 with fractional seconds, RFC 3339, and space-delimited SQLite datetime formats to prevent parser fallback to timestamp `0`.
4. **Ghost Conversation Pruning**: Exclude 0-word untitled scratch sessions spawned by the IDE from inflating running counts.
5. **Exact Instance ID Matching**: In `isNodeOwnedByInstance` in `Instances.tsx`, eliminate loose suffix matching (`instConfig.id.endsWith(node.instance_id)`) that caused cross-instance node bleed.
6. **Startup Zombie Flag Reset**: In `purge_corrupted_running_projects`, execute `UPDATE running_projects SET is_running = 0` on initialization to clear lingering active states from ungraceful shutdowns.

---

## 2. Target Files & Symbol Breakdown

### 2.1 `src-tauri/src/modules/repo_db.rs`
- [`detect_running_projects`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L510-L650):
  - Pre-flight verify instance process liveness (`find_pids_for_data_dir`).
  - If process table returns empty, set `is_project_active = false` and log `INSTANCE_PROCESS_DEAD`.
- [`is_prompt_running_for_project`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L1594-L1930):
  - Enforce Gate 0 host process liveness.
  - Upgrade Gate 4 timestamp parsing to handle fractional seconds (`%Y-%m-%dT%H:%M:%S%.f`, `%Y-%m-%d %H:%M:%S%.f`).
  - Expand thinking window from 120s to 600s (`now - conv_time <= 600`).
  - Maintain strict idle supremacy (`not_fully_idle == 0` or status contains `IDLE`, `COMPLETED`, `FAILED`, `CANCELLED`).
  - Filter 0-word untitled ghost sessions.
- [`compute_project_conversation_tree`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L3949-L4030):
  - Require target instance process liveness before tree computation.
  - Partition cache strictly by instance ID (`tree:{instance_id}:{max_words}:{only_running}`).
- [`purge_corrupted_running_projects`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/repo_db.rs#L310-L328):
  - Add `UPDATE running_projects SET is_running = 0` to reset zombie records on startup.
  - Wipe `prompt_tree_cache`.

### 2.2 `src-tauri/src/modules/instance.rs`
- [`is_instance_running`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/instance.rs#L2584-L2605):
  - Validate saved PID against OS process table via `sysinfo`.
  - Enforce `process_identity_matches` verification on executable path and name.
  - Fall back to scanning process command lines for `--user-data-dir` matching `data_dir`.

### 2.3 `src/pages/Instances.tsx`
- [`isNodeOwnedByInstance`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L102-L128):
  - Enforce exact instance ID matching (`node.instance_id === instConfig.id`).
  - Remove loose suffix matching (`instConfig.id.endsWith(node.instance_id)`).

### 2.4 `src/components/instances/PromptTreeViewModal.tsx`
- [`useEffect` (modal open)](file:///d:/work/Antigravity-Manager/src/components/instances/PromptTreeViewModal.tsx#L1060-L1074):
  - Guarantee `loadTree(true, true)` on mount to bypass both client memory and SQLite cache.
- [`handleRestore`](file:///d:/work/Antigravity-Manager/src/components/instances/PromptTreeViewModal.tsx#L1117-L1132):
  - Guarantee `loadTree(true, true)` on post-restoration refresh instead of parameter-less `loadTree()`.

---

## 3. Granular Step-by-Step Implementation

### Step 3.1: Host Process Liveness Gate (Gate 0) in `repo_db.rs`
1. In `detect_running_projects(instance_id: &str)`:
   - Call `crate::modules::instance::find_pids_for_data_dir(&instance.data_dir, instance.is_default)`.
   - Compute `let is_instance_active = !pids.is_empty();`.
   - If `!is_instance_active`:
     - Every project in `workspaceStorage` must have `is_running = false`.
     - Log `INSTANCE_PROCESS_DEAD` via `log_instance_prompt_audit`.
2. In `is_prompt_running_for_project(project_id: &str, instance_id: &str)`:
   - Verify `has_active_process` at Gate 0 before inspecting in-memory maps or SQLite records.
   - If `!has_active_process`, immediately return `false`.

### Step 3.2: Multi-Format Robust Timestamp Parsing in Gate 4
1. In Gate 4 of `is_prompt_running_for_project`:
   - Parse `last_modified_time` using cascading parser:
     ```rust
     let raw_time = _last_time_str.trim();
     let norm_time = raw_time.replacen(' ', "T", 1);
     let conv_time = chrono::DateTime::parse_from_rfc3339(&norm_time)
         .map(|dt| dt.timestamp())
         .or_else(|_| {
             chrono::NaiveDateTime::parse_from_str(&norm_time, "%Y-%m-%dT%H:%M:%S%.f")
                 .map(|dt| dt.and_utc().timestamp())
         })
         .or_else(|_| {
             chrono::NaiveDateTime::parse_from_str(raw_time, "%Y-%m-%d %H:%M:%S%.f")
                 .map(|dt| dt.and_utc().timestamp())
         })
         .or_else(|_| {
             chrono::NaiveDateTime::parse_from_str(&norm_time, "%Y-%m-%dT%H:%M:%S")
                 .map(|dt| dt.and_utc().timestamp())
         })
         .or_else(|_| {
             chrono::NaiveDateTime::parse_from_str(raw_time, "%Y-%m-%d %H:%M:%S")
                 .map(|dt| dt.and_utc().timestamp())
         })
         .unwrap_or(0);
     ```
2. Replace rigid 120s TTL with adaptive 600s thinking window:
   ```rust
   let is_recent = conv_time > 0 && (now - conv_time <= 600);
   ```

### Step 3.3: Ghost Conversation & Idle Supremacy Filter
1. Preserve explicit idle override:
   ```rust
   let is_idle_count = not_fully_idle == 0;
   let has_idle_status = status.contains("IDLE")
       || status.contains("COMPLETED")
       || status.contains("FAILED")
       || status.contains("CANCELLED");
   let is_explicit_idle = is_idle_count || has_idle_status;
   let is_conv_running = !is_explicit_idle && not_fully_idle > 0 && status.contains("RUNNING");
   ```
2. In `compute_project_conversation_tree`, filter out conversation records where:
   - `prompt_word_count == 0` AND title begins with "untitled" or is empty.

### Step 3.4: Exact Instance ID Ownership in `Instances.tsx`
1. In `src/pages/Instances.tsx`:
   - Update `isNodeOwnedByInstance`:
     ```typescript
     export const isNodeOwnedByInstance = (
         node: AgmProjectTreeNode,
         instConfig: { id: string; is_default?: boolean; seq_num?: number }
     ): boolean => {
         if (!node.instance_id || node.instance_id.trim() === '') {
             return false;
         }
         if (instConfig.is_default) {
             return isDefaultOwned(node.instance_id, instConfig.id);
         }
         if (node.instance_id === 'default' || node.instance_id === '__default__') {
             return false;
         }
         if (node.instance_id === instConfig.id) {
             return true;
         }
         const hasSeqNum = typeof instConfig.seq_num === 'number' && instConfig.seq_num > 1;
         if (hasSeqNum && node.instance_seq_num === instConfig.seq_num) {
             return true;
         }
         return false;
     };
     ```

### Step 3.5: Startup Zombie Flag Reset in `purge_corrupted_running_projects`
1. In `src-tauri/src/modules/repo_db.rs`:
   - Enhance `purge_corrupted_running_projects(conn: &Connection)`:
     ```rust
     pub fn purge_corrupted_running_projects(conn: &Connection) -> Result<(usize, usize), String> {
         let deleted_projects = conn
             .execute(
                 "DELETE FROM running_projects 
                  WHERE workspace_storage_path IS NULL 
                     OR trim(workspace_storage_path) = '' 
                     OR instr(id, '__') = 0 
                     OR trim(instance_id) = ''",
                 [],
             )
             .map_err(|e| format!("Failed to purge corrupted running_projects: {}", e))?;

         let _ = conn.execute("UPDATE running_projects SET is_running = 0", []);

         let deleted_cache = conn
             .execute("DELETE FROM prompt_tree_cache", [])
             .map_err(|e| format!("Failed to wipe prompt_tree_cache: {}", e))?;

         Ok((deleted_projects, deleted_cache))
     }
     ```

### Step 3.6: Cache Invalidation in `PromptTreeViewModal.tsx`
1. Ensure modal mount calls `loadTree(true, true, latestArchived, latestPinned)`.
2. Update `handleRestore` to invoke `loadTree(true, true)` upon successful restoration dispatch.

---

## 4. Verification & Testing Checklist

- [ ] **PID Dead Gating**: Kill Antigravity process; verify `detect_running_projects` sets `is_running: false` for all projects.
- [ ] **Extended Thinking Window**: Verify prompts with turn age between 120s and 600s continue reporting `is_running: true` when status is `RUNNING`.
- [ ] **Fractional Timestamp Test**: Verify timestamps formatted with microseconds parse to correct Unix timestamps.
- [ ] **Ghost Pruning**: Verify 0-word untitled conversations are not counted as running.
- [ ] **Cross-Instance Isolation**: Verify instances with matching ID suffixes do not claim each other's project nodes.
- [ ] **Startup Purge**: Verify `UPDATE running_projects SET is_running = 0` runs cleanly without database lock errors.
