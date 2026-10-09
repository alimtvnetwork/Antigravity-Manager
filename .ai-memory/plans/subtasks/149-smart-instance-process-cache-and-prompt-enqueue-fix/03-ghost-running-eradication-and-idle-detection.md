# Subtask 03: Ghost Running Eradication & Strict Idle Detection

- **Subtask ID**: `149-03`
- **Parent Task**: `149-smart-instance-process-cache-and-prompt-enqueue-fix`
- **Target Files**:
  - `src-tauri/src/modules/repo_db.rs`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: `[READY_FOR_EXECUTION]`

---

## 1. Objective

Completely eliminate false-positive "ghost running" projects and conversation nodes by patching:
1. `auto_resume_recent_prompts` to strictly query pending/queued prompts (`WHERE status IN ('queued', 'pending')`) within recent cutoff, preventing resurrecting stale completed tasks.
2. `discover_running_prompts_from_antigravity` to eliminate the `prompts.is_empty()` and `prompts.len() < 5` bypass that forced idle conversations into `backed_up` status.
3. `compute_project_conversation_tree` fallback scanning loops to enforce a strict recency check (`now - last_time <= 120`).
4. `inspect_conversation_transcript` to strictly propagate `is_terminal_done` and suppress running states for completed turns.

---

## 2. Context & Root Cause Analysis

### 2.1 Resurrecting Completed Tasks in `auto_resume_recent_prompts`
- **Location**: `src-tauri/src/modules/repo_db.rs` (~L3965–3975).
- **Flaw**: The SQL query currently reads:
  ```rust
  "SELECT id, prompt_content, model, image_payload 
   FROM active_prompts 
   WHERE (project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3)
     AND (instance_id = ?4 OR (?4 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
   ORDER BY created_at ASC, id ASC LIMIT 1"
  ```
  Because there is no `status` filter, this query selects completed prompts (`status = 'completed'` or `'dispatched'`). When auto-resume runs after a profile switch or window reload, it picks up old completed prompts, re-dispatches them, and marks the project as actively running.
- **Root Cause Fix**: Add `WHERE status IN ('queued', 'pending')`. Furthermore, when reading `.antigravity_resume_task.json`, verify that `status` is explicitly `"queued"` or `"pending"`.

### 2.2 Forced `backed_up` Status Bypass in `discover_running_prompts_from_antigravity`
- **Location**: `src-tauri/src/modules/repo_db.rs` (~L1003–1007).
- **Flaw**:
  ```rust
  let is_running_or_recent =
      not_fully_idle != 0 || status.contains("RUNNING") || prompts.is_empty();
  if !is_running_or_recent && prompts.len() >= 5 {
      continue;
  }
  ```
  When `prompts` is empty or has fewer than 5 items, the loop bypasses the idle check! Even if `not_fully_idle == 0` and `status` is `"IDLE"` or `"COMPLETED"`, the conversation is converted into an `ActivePrompt` with `status: "backed_up".to_string()`. Downstream functions interpret this as an active prompt, generating false-positive running badges.
- **Root Cause Fix**: Remove `prompts.is_empty()` and `prompts.len() < 5`. Only accept conversations where `not_fully_idle != 0 && status.contains("RUNNING")`.

### 2.3 Stale Crashed Sessions in `compute_project_conversation_tree`
- **Location**: `src-tauri/src/modules/repo_db.rs` (~L5014–5046).
- **Flaw**:
  ```rust
  let (ws_uris_raw, _last_time, status, not_fully_idle) = item;
  let is_running = status.contains("RUNNING") && not_fully_idle > 0;
  ```
  `_last_time` is bound to an unused variable! If Antigravity was closed or crashed hours or days ago while in a `RUNNING` state, `conversation_summaries.db` retains `status = "RUNNING"`. Without checking `_last_time`, the fallback scan forever marks the project as running.
- **Root Cause Fix**: Parse `_last_time` into a Unix epoch timestamp and enforce `(now_fb - last_time_epoch) <= 120`.

### 2.4 Terminal State Propagation in `inspect_conversation_transcript`
- **Location**: `src-tauri/src/modules/repo_db.rs` (~L4659–4795).
- **Flaw**: `TranscriptInspection` determines `is_terminal_done`, but in `compute_project_conversation_tree`, certain fallback branches check summary flags without verifying `inspection.is_terminal_done`.
- **Root Cause Fix**: Enforce that when `inspection.is_terminal_done == true`, the conversation status is overridden to `"COMPLETED"` or `"IDLE"`, and `effective_is_run = false`.

---

## 3. Detailed Implementation Steps

### Step 1: Patch `auto_resume_recent_prompts` in `src-tauri/src/modules/repo_db.rs`

1. Open `src-tauri/src/modules/repo_db.rs` around line 3965.
2. Update the query to filter strictly by status and preserve FIFO ordering:
   ```rust
   let mut prompt_stmt = conn
       .prepare(
           "SELECT id, prompt_content, model, image_payload 
            FROM active_prompts 
            WHERE status IN ('queued', 'pending')
              AND (project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3)
              AND (instance_id = ?4 OR (?4 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
            ORDER BY created_at ASC, id ASC LIMIT 1",
       )
       .map_err(|e| format!("Failed to prepare prompt query: {}", e))?;
   ```
3. Update the `.antigravity_resume_task.json` reader (~L3991–4016):
   - Check the `status` field in the JSON payload.
   - Only accept if `status == "queued"` or `status == "pending"`.
   - If `status == "completed"` or missing, ignore the file.

### Step 2: Patch `discover_running_prompts_from_antigravity` in `src-tauri/src/modules/repo_db.rs`

1. Locate `discover_running_prompts_from_antigravity` around line 1003.
2. Replace lines 1003–1007:
   ```rust
   // OLD DEFECTIVE CODE:
   // let is_running_or_recent = not_fully_idle != 0 || status.contains("RUNNING") || prompts.is_empty();
   // if !is_running_or_recent && prompts.len() >= 5 {
   //     continue;
   // }

   // NEW STRICT FILTER:
   let is_genuinely_running = not_fully_idle != 0 && status.contains("RUNNING");
   if !is_genuinely_running {
       continue;
   }
   ```
3. Ensure that only truly active sessions are collected into `prompts`.

### Step 3: Add Recency Check in `compute_project_conversation_tree` Fallback Loop

1. Locate the fallback loop reading `conversation_summaries.db` (~L5014–5046).
2. Parse `_last_time` (or `last_time_str`):
   ```rust
   let (ws_uris_raw, last_time_str, status, not_fully_idle) = item;
   let last_time_epoch = parse_flexible_timestamp(&last_time_str);
   let is_recent = last_time_epoch > 0 && (now_fb - last_time_epoch <= 120);
   let is_running = status.contains("RUNNING") && not_fully_idle > 0 && is_recent;
   ```
3. If `is_recent` is false, `is_running` is strictly `false`.

### Step 4: Enforce `is_terminal_done` in `compute_project_conversation_tree`

1. In `compute_project_conversation_tree` (~L5480–5520), where conversation node properties are mapped:
2. Ensure that if `inspection.is_terminal_done` is true:
   ```rust
   let effective_is_run = if inspection.is_terminal_done {
       false
   } else {
       is_inst_alive && is_non_idle && (inspection.is_recent_active || has_active_worker)
   };
   ```
3. When `inspection.is_terminal_done` is true, set node status to `"COMPLETED"` or `"IDLE"`.

---

## 4. Invariants & Constraints

- **FIFO Queue Invariant**: All prompt restoration queries must enforce `ORDER BY created_at ASC, id ASC`.
- **Positive Booleans Only**: Use `is_running`, `is_queued`, `is_recent`, `is_genuinely_running`.
- **Zero Synthetic Prompts**: Never create fictitious crash recovery prompt records.
- **Recency Upper Bound**: Any background session with no transcript updates in > 120 seconds must not be reported as running.

---

## 5. Verification & Pre-flight Checklist

```bash
# Pre-flight Rust compilation and formatting checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::repo_db
```

---

## 6. Done When

- [ ] `auto_resume_recent_prompts` query includes `WHERE status IN ('queued', 'pending')` and never resurrects completed tasks.
- [ ] `.antigravity_resume_task.json` reader validates `status == 'queued' || status == 'pending'`.
- [ ] `discover_running_prompts_from_antigravity` no longer contains the `prompts.is_empty()` or `prompts.len() < 5` bypass.
- [ ] `compute_project_conversation_tree` fallback loop checks `now_fb - last_time_epoch <= 120`.
- [ ] `inspect_conversation_transcript` terminal state (`is_terminal_done`) strictly forces `effective_is_run = false`.
- [ ] All Rust tests pass with zero clippy warnings.
