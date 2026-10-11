# Subtask 02: Prompt Pipeline Dispatch & Queue Engine (QE) Fixes

> **Status:** Pending Execution  
> **Owner:** `@aukgit`  
> **Parent Spec:** `02-spec/21-app/update-cli-command-and-pipeline-fixes/01-architecture-spec.md`  
> **Estimated Scope:** 4 files (`src-tauri/src/bin/agm/prompt_cmds.rs`, `src-tauri/src/modules/repo_db/queue.rs`, `src-tauri/src/modules/repo_db/backup.rs`, `src-tauri/src/modules/repo_db/dispatch.rs`)  

---

## 1. Objectives & Context

Reconnect the prompt dispatch and queueing pipeline across the AGM CLI and backend database modules by:
1. Grounding repository directory resolution in `cmd_prompts_send` and `cmd_prompts_queue` to use `std::env::current_dir()` instead of dummy `"scratch/test-repo"`.
2. Resolving authentic session and conversation identifiers rather than generating random UUIDs or inserting `session_id: NULL`.
3. Wiring `cmd_prompts_send` to trigger the execution bridge via `repo_db::spawn_prompt_via_agy`.
4. Preventing unrecoverable prompt loss in `check_and_dispatch_enqueued_prompts`: keeping prompts in `'queued'` state if execution bridge dispatch fails.
5. Expanding `backup_running_prompts` transition query to capture all active prompt statuses: `WHERE status IN ('running', 'in_flight', 'dispatched', 'queued')`.
6. Resolving cross-instance scoping mismatch during auto-switch by providing workload migration from the source instance to the newly active target instance before FIFO dispatch.

---

## 2. Target Files & Line-by-Line Changes

### 2.1 File: `src-tauri/src/bin/agm/prompt_cmds.rs`
- **Location:** Lines 106–110, 164, 170–190 (`cmd_prompts_send`) and Lines 288–292, 326–337 (`cmd_prompts_queue`).
- **Current Defect:**
  - `cmd_prompts_send` line 109 defaults `repo_path` to `"scratch/test-repo"`.
  - Line 164 sets `session_id: Some(format!("conv-{}", uuid::Uuid::new_v4()))`.
  - Lines 170–190 write `.antigravity_resume_task.json` but omit calling `repo_db::spawn_prompt_via_agy(&active_prompt)`.
  - `cmd_prompts_queue` line 291 defaults `repo_path` to `"scratch/test-repo"`.
  - Line 327 inserts `session_id: NULL`.
- **Required Modification:**
  ```rust
  // In cmd_prompts_send:
  let repo_path = ctx.repo_path.clone().unwrap_or_else(|| {
      std::env::current_dir()
          .map(|p| p.to_string_lossy().to_string())
          .unwrap_or_else(|_| ".".to_string())
  });
  let slug = crate::common::derive_current_repo_slug();

  // Resolve active conversation session or derive from slug
  let session_id = repo_db::lookup_conversation_title(&repo_path)
      .map(|_| format!("conv-{}", slug))
      .unwrap_or_else(|| format!("conv-{}", slug));

  let active_prompt = ActivePrompt {
      id: prompt_id.clone(),
      project_id: slug.clone(),
      instance_id: resolved_inst.clone(),
      repo_path: repo_path.clone(),
      prompt_content: final_prompt.clone(),
      model: Some(model),
      session_id: Some(session_id.clone()),
      status: "in_flight".to_string(),
      created_at: now,
      updated_at: now,
      image_payload: None,
  };

  let _ = repo_db::insert_active_prompt(&active_prompt);

  // Trigger execution bridge
  let is_spawn_successful = repo_db::spawn_prompt_via_agy(&active_prompt);
  if is_spawn_successful {
      let _ = repo_db::connect_db().and_then(|conn| {
          conn.execute(
              "UPDATE active_prompts SET status = 'running', updated_at = ?1 WHERE id = ?2",
              rusqlite::params![now, &prompt_id],
          ).map_err(|e| crate::error::AppError::Sqlite(e))
      });
  }

  // In cmd_prompts_queue:
  let repo_path = ctx.repo_path.clone().unwrap_or_else(|| {
      std::env::current_dir()
          .map(|p| p.to_string_lossy().to_string())
          .unwrap_or_else(|_| ".".to_string())
  });
  let slug = crate::common::derive_current_repo_slug();
  let session_id = format!("conv-{}", slug);

  if let Ok(ref conn) = conn_res {
      let _ = conn.execute(
          "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'queued', ?8, ?9)",
          rusqlite::params![
              &prompt_id,
              &slug,
              &resolved_inst,
              &repo_path,
              &final_prompt,
              &model,
              &session_id,
              now,
              now,
          ],
      );
  }
  ```

### 2.2 File: `src-tauri/src/modules/repo_db/queue.rs`
- **Location:** Lines 191–201 (`check_and_dispatch_enqueued_prompts`).
- **Current Defect:** Unconditionally marks the prompt as `'dispatched'` even when `spawn_prompt_via_agy` fails, causing the prompt to be permanently lost from the queue.
- **Required Modification:**
  ```rust
  // Spawn prompt via agy
  let is_spawn_successful = spawn_prompt_via_agy(&prompt);
  let has_valid_conversation = prompt
      .session_id
      .as_deref()
      .map(|id| !id.trim().is_empty())
      .unwrap_or(false);

  // Only transition status to 'dispatched' if spawn succeeded or valid resume task was created
  if is_spawn_successful || (is_resume_file_written && has_valid_conversation) {
      crate::error::record_ignored(
          conn.execute(
              "UPDATE active_prompts SET status = 'dispatched', updated_at = ?1 WHERE id = ?2",
              params![now, &prompt.id],
          ),
          "mark prompt dispatched after agy send",
      );
      dispatched_count += 1;
  } else {
      crate::modules::logger::log_warn(&format!(
          "[PromptQueueScheduler] Prompt '{}' dispatch failed; keeping in 'queued' status for next retry cycle",
          prompt.id
      ));
      // Keep prompt in 'queued' status and update timestamp to avoid immediate thrashing
      crate::error::record_ignored(
          conn.execute(
              "UPDATE active_prompts SET updated_at = ?1 WHERE id = ?2",
              params![now, &prompt.id],
          ),
          "update retry timestamp for queued prompt",
      );
  }
  ```

### 2.3 File: `src-tauri/src/modules/repo_db/backup.rs`
- **Location:** Lines 65–83 (`backup_running_prompts`).
- **Current Defect:** Only updates prompts `WHERE status = 'running'`, dropping `in_flight`, `dispatched`, and `queued` prompts.
- **Required Modification:**
  ```rust
  let transitioned = if instance_id == "all" {
      conn.execute(
          "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 
           WHERE status IN ('running', 'in_flight', 'dispatched', 'queued')",
          params![now],
      )
      .unwrap_or(0)
  } else if is_all_or_default {
      conn.execute(
          "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 
           WHERE status IN ('running', 'in_flight', 'dispatched', 'queued') 
             AND instance_id IN ('default', '__default__')",
          params![now],
      )
      .unwrap_or(0)
  } else {
      conn.execute(
          "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 
           WHERE status IN ('running', 'in_flight', 'dispatched', 'queued') 
             AND instance_id = ?2",
          params![now, instance_id],
      )
      .unwrap_or(0)
  };
  ```

### 2.4 File: `src-tauri/src/modules/repo_db/dispatch.rs`
- **Location:** Lines 59–92 (`dispatch_running_prompts`).
- **Current Defect:** Filters prompts by exact instance match (`prompt_inst == target_inst`), discarding prompts backed up on a different source instance prior to profile switch.
- **Required Modification:**
  1. Add cross-instance migration helper:
  ```rust
  pub fn reassign_backed_up_prompts_to_instance(source_instance: &str, target_instance: &str) -> Result<usize, String> {
      let conn = connect_db()?;
      let now = Utc::now().timestamp();
      let updated = conn.execute(
          "UPDATE active_prompts SET instance_id = ?1, updated_at = ?2 
           WHERE status = 'backed_up' AND (instance_id = ?3 OR (?3 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))",
          rusqlite::params![target_instance, now, source_instance],
      ).map_err(|e| format!("Failed to reassign backed up prompts: {}", e))?;
      Ok(updated)
  }
  ```
  2. In `dispatch_running_prompts`, enforce strict FIFO ordering:
  ```rust
  let mut stmt = conn
      .prepare(
          "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
           FROM active_prompts 
           WHERE status = 'backed_up'
           ORDER BY created_at ASC, id ASC",
      )
      .map_err(|e| format!("Failed to prepare dispatch query: {}", e))?;
  ```

---

## 3. Input & Output Structures

### Prompt Record Structure (`ActivePrompt`):
```rust
pub struct ActivePrompt {
    pub id: String,
    pub project_id: String,
    pub instance_id: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub image_payload: Option<String>,
}
```

### Resume Task Document Structure (`.antigravity_resume_task.json`):
```json
{
  "prompt_id": "prompt-c917b2...",
  "project_id": "Antigravity-Manager",
  "conversation_id": "conv-Antigravity-Manager",
  "instance_id": "default",
  "repo_path": ".",
  "prompt_content": "Example prompt text",
  "dispatched_at": 1775908000,
  "status": "dispatched"
}
```

---

## 4. Acceptance Criteria

1. **AC-01 (Path Grounding):** `agm prompt send` and `agm prompt queue` record the caller's actual current directory (`std::env::current_dir()`) and repo slug instead of `"scratch/test-repo"`.
2. **AC-02 (Session Continuity):** Enqueued and sent prompts contain valid conversation session IDs (`conv-<slug>`) rather than `NULL` or detached random UUIDs.
3. **AC-03 (Execution Bridge Activation):** `agm prompt send` initiates `spawn_prompt_via_agy` immediately, transitioning prompt status to `'running'` upon success.
4. **AC-04 (Queue Loss Prevention):** When `spawn_prompt_via_agy` fails in `check_and_dispatch_enqueued_prompts`, the prompt status remains `'queued'` and is retried on subsequent cycles.
5. **AC-05 (Broad Backup Transition):** `backup_running_prompts` transitions all `in_flight`, `dispatched`, `queued`, and `running` prompts to `'backed_up'`.
6. **AC-06 (Cross-Instance Restore):** Prompts backed up during an instance switch can be reassigned to the target instance and successfully dispatched in strict FIFO sequence (`ORDER BY created_at ASC, id ASC`).

---

## 5. Verification Plan

1. **Unit & Module Testing:**
   ```powershell
   cd src-tauri
   cargo test modules::repo_db::tests
   ```
2. **CLI Prompt Send Smoke Test:**
   ```powershell
   target\debug\agm.exe prompt send "Verification test prompt"
   target\debug\agm.exe prompts ls 1 --json
   ```
3. **Queue Scheduler Single Cycle Test:**
   ```powershell
   target\debug\agm.exe queue-scheduler --once
   ```
4. **Backup & Restore Simulation:**
   ```powershell
   target\debug\agm.exe prompts backup
   target\debug\agm.exe prompts restore
   ```
