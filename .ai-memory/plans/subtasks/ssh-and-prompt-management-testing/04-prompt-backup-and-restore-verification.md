# Subtask Implementation Plan: Prompt Backup & Restore Verification with Image Preservation & Audit Logs

> **Subtask ID:** `04-prompt-backup-and-restore-verification`  
> **Parent Slug:** `ssh-and-prompt-management-testing`  
> **Spec Reference:** [02-spec/21-app/ssh-and-prompt-management-testing/01-architecture-spec.md](02-spec/21-app/ssh-and-prompt-management-testing/01-architecture-spec.md)  
> **Target Files:**
> - `src-tauri/src/modules/backup_prompts_db/backup.rs`
> - `src-tauri/src/modules/backup_prompts_db/restore.rs`
> - `src-tauri/src/modules/backup_prompts_db/db.rs`
> - `src-tauri/src/modules/backup_prompts_db/types.rs`
> - `src-tauri/src/modules/instance/switch.rs`
> - `src-tauri/src/modules/instance/post_switch_resume.rs`
> - `src-tauri/src/modules/instance/workspace.rs`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

When Antigravity-Manager executes an account switch or instance profile rotation (e.g. triggered automatically by 98% quota exhaustion or manually via UI/CLI), any running or in-flight prompt turns must not be dropped. Furthermore, prompts containing multimodal image payloads (such as UI mockups, terminal screenshots, or KaTeX diagrams) must preserve their image attachments across the restart boundary.

Antigravity-Manager employs a dual-database snapshot and rehydration architecture:
1. **Primary Operational DB (`repo_prompts.db`)**: Manages live `active_prompts` with columns `id`, `project_id`, `instance_id`, `repo_path`, `prompt_content`, `status`, `image_payload`, and timestamps.
2. **Dedicated Split Backup DB (`backup-prompts.db`)**: Persists structured snapshots across `backup_batches` and `prompt_backups` with retention tracking, per-instance scoping, sequence IDs, and full image payloads.
3. **Post-Switch Resume Timer (`post_switch_resume.rs`)**: Replaces hardcoded blind sleeps with a configurable, cancellable timer (`post_switch_resume_delay_seconds`, default 10s) that waits for the newly spawned IDE prompt channel before rehydrating prompts.

The objective of this subtask is to verify end-to-end prompt preservation, multimodal image integrity, database state transitions, and audit log generation during instance switches.

---

## 2. Database Schemas & Status State Machine

### 2.1 SQLite Database Contracts

#### `prompt_backups` in `data/backup-prompts/backup-prompts.db`
```sql
CREATE TABLE IF NOT EXISTS prompt_backups (
    id TEXT PRIMARY KEY,
    backup_batch_id TEXT NOT NULL,
    prompt_id TEXT NOT NULL,
    project_name TEXT NOT NULL,
    project_path TEXT NOT NULL,
    project_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    conversation_name TEXT,
    sequence_id INTEGER NOT NULL,
    prompt_text TEXT NOT NULL,
    has_images BOOLEAN NOT NULL DEFAULT 0,
    images_payload TEXT,
    status TEXT NOT NULL DEFAULT 'queued',
    created_at INTEGER NOT NULL,
    is_restored BOOLEAN NOT NULL DEFAULT 0,
    restored_at INTEGER,
    instance_id TEXT DEFAULT 'default',
    FOREIGN KEY(backup_batch_id) REFERENCES backup_batches(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_prompt_backups_batch ON prompt_backups(backup_batch_id);
CREATE INDEX IF NOT EXISTS idx_prompt_backups_restored ON prompt_backups(is_restored, restored_at);
CREATE INDEX IF NOT EXISTS idx_prompt_backups_instance ON prompt_backups(instance_id);
```

#### `active_prompts` in `data/repo_prompts.db`
```sql
CREATE TABLE IF NOT EXISTS active_prompts (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    instance_id TEXT NOT NULL DEFAULT 'default',
    repo_path TEXT NOT NULL,
    prompt_content TEXT NOT NULL,
    model TEXT,
    session_id TEXT,
    status TEXT NOT NULL DEFAULT 'queued',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    image_payload TEXT
);
```

---

### 2.2 Prompt Lifecycle & Status Transition Machine

```
[Prompt Dispatched / Running]
             │
             │ Account Switch Triggered (STAGE_1_SNAPSHOT)
             ▼
[status = 'backed_up'] ──► Saved to prompt_backups (has_images = 1, images_payload intact)
             │
             │ Cross-instance re-attribution (if target != source)
             ▼
[instance_id updated to target_id]
             │
             │ Pre-switch PID eviction & IDE process relaunch (STAGE_2 to STAGE_4)
             ▼
[PostSwitchResume Timer: 10s wait for prompt channel]
             │
             │ STAGE_5_RESTORE: restore_running_prompts_for_instance
             ▼
[prompt_status_after_restore]
   - If previously 'queued'   ──► Remains 'queued'
   - If previously 'running'  ──► Set to 'backed_up' for re-dispatch
             │
             ▼
[dispatch_running_prompts] ──► Pushed to IDE via prompt channel
             │
             ▼
[prompt_backups.is_restored = 1, restored_at = timestamp]
```

---

## 3. Concrete Verification & Test Steps

### Step 1: Verify Multimodal Image Payload Preservation in `backup.rs`
- **Target File:** `src-tauri/src/modules/backup_prompts_db/backup.rs`.
- **Test Scenarios:**
  1. Create a prompt in `active_prompts` containing image payloads:
     - `has_images: true`
     - `image_payload: Some("[\"data:image/png;base64,iVBORw0KGgo...\"]")`
  2. Invoke `backup_active_running_prompts_for_instance(Some("default"), None)`.
  3. Query `prompt_backups` table:
     - Verify `has_images == 1`.
     - Verify `images_payload` matches the exact string without JSON truncation or byte loss.
     - Verify `sequence_id` is assigned in strict FIFO order (`ORDER BY created_at ASC, id ASC`).

### Step 2: Verify Cross-Instance Re-Attribution in `switch.rs`
- **Target File:** `src-tauri/src/modules/instance/switch.rs`.
- **Test Scenarios:**
  1. Trigger switch from `instance_A` to `instance_B`.
  2. Verify Stage 1 Snapshot execution in `switch_account_to_instance`:
     - Checks backed-up prompts from `source_instance_id`.
     - Executes SQL:
       ```sql
       UPDATE active_prompts 
       SET instance_id = ?1, updated_at = ?2 
       WHERE instance_id = ?3 AND status = 'backed_up'
       ```
     - Emits log trace:
       ```
       [INSTANCE_SWITCH:STAGE_1_SNAPSHOT] Backed up N prompts from active source instance 'instance_A' for target 'instance_B'
       ```
  3. Verify that prompts originally attributed to `instance_A` now have `instance_id = 'instance_B'` so the new instance does not suffer from prompt amnesia.

### Step 3: Verify Post-Switch Resume Scheduling in `post_switch_resume.rs`
- **Target File:** `src-tauri/src/modules/instance/post_switch_resume.rs`.
- **Test Scenarios:**
  1. Verify `schedule_post_switch_resume("instance_B")`:
     - Reads configured delay from `auto_profile_switcher.post_switch_resume_delay_seconds` (default 10s).
     - Queries `collect_pending_prompt_previews`: verifies `PendingPromptPreview` records `has_image = true` when image payloads exist.
     - Spawns asynchronous tokio sleep task.
     - Emits frontend event `prompts://resume-scheduled`.
     - Emits log trace:
       ```
       [PostSwitchResume] Scheduling prompt resume for instance 'instance_B' in 10s (N prompt(s))
       ```
  2. Test cancellation / superseding:
     - Schedule a new switch before the 10s delay elapses.
     - Verify `cancel_post_switch_resume` aborts the previous tokio task handle, emits `prompts://resume-cancelled`, and prevents duplicate re-injections.

### Step 4: Verify Rehydration & Re-Dispatch in `restore.rs` and `workspace.rs`
- **Target Files:** `src-tauri/src/modules/backup_prompts_db/restore.rs`, `src-tauri/src/modules/instance/workspace.rs`.
- **Test Scenarios:**
  1. Upon timer expiration, `push_backed_up_prompts_after_channel("instance_B")` executes:
     - Waits for IDE prompt channel readiness (`wait_for_instance_prompt_channel`).
     - Rehydrates records from `prompt_backups` into `active_prompts` via `repo_db::save_or_requeue_prompt`.
     - Emits restoration log:
       ```
       [BackupDB] Restore prompt <id> on instance_B: saved 'running' -> 'backed_up'
       ```
     - Dispatches rehydrated prompts to the IDE.
     - Emits completion log:
       ```
       [INSTANCE_SWITCH:STAGE_5_RESTORE] Re-pushed N prompt(s) for instance 'instance_B' (N backed up before, 0 still backed up)
       ```
     - Updates `prompt_backups` table: `is_restored = 1`, `restored_at = <timestamp>`.
     - Emits frontend event `prompts://restored`.
  2. Single Retry Guarantee:
     - If the first push yields 0 prompts (e.g. channel initial handshake lag), verify `run_resume_with_retry` waits `RESTORE_RETRY_DELAY_SECONDS` (7s) and retries once before giving up.

---

## 4. Concrete Audit Log Verification Matrix

During execution, verify that the application logs and SQLite tables record the following exact lines:

| Stage | Subsystem | Expected Log Pattern |
|---|---|---|
| **1. Switch Start** | `instance::switch` | `[INSTANCE_SWITCH:START] Target instance: '{id}' (default: {bool}), Target account: '{email}'` |
| **2. Prompt Snapshot** | `instance::switch` | `[INSTANCE_SWITCH:STAGE_1_SNAPSHOT] Backed up {count} prompts from active source instance '{src}' for target '{tgt}'` |
| **3. PID Eviction** | `instance::switch` | `[INSTANCE_SWITCH:STAGE_2_TERMINATE] Pre-switch PID eviction for instance '{id}' (running: {bool}, PIDs: [...], agy workers: {n})` |
| **4. Resume Schedule**| `instance::post_switch_resume` | `[PostSwitchResume] Scheduling prompt resume for instance '{id}' in {delay}s ({count} prompt(s))` |
| **5. Restore Fired** | `instance::workspace` | `[INSTANCE_SWITCH:STAGE_5_RESTORE] Post-launch delay elapsed for instance '{id}'` |
| **6. DB Rehydration** | `backup_prompts_db::restore`| `[BackupDB] Restore prompt {prompt_id} on {id}: saved '{old}' -> '{new}'` |
| **7. Final Re-Push** | `instance::workspace` | `[INSTANCE_SWITCH:STAGE_5_RESTORE] Re-pushed {n} prompt(s) for instance '{id}' ({b} backed up before, {a} still backed up)` |

---

## 5. Invariants & Guardrails

- **Strict FIFO Ordering Invariant**: All queries selecting prompt backups for restoration MUST use `ORDER BY created_at ASC, sequence_id ASC` (or `ORDER BY created_at ASC, id ASC`). LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden.
- **Multimodal Image Payload Preservation**: `has_images` and `images_payload` columns in `prompt_backups` and `active_prompts` must never be stripped, truncated, or overwritten with null during backup or restore.
- **Zero Silent Errors**: Rusqlite executions and file operations must route errors through `AppError` or `crate::error::record_ignored(..., "name")` with a mandatory `// Justification:` comment.
- **Attribution Invariant**: Strictly attribute `@aukgit` in changelogs and git commits.

---

## 6. Verification Checkpoints

1. **Rust Unit Tests for Backup Prompts DB:**
   ```powershell
   cargo test --lib modules::backup_prompts_db
   ```
2. **Rust Compilation & Clippy Pre-flight Gate:**
   ```powershell
   cargo clippy --all-targets --all-features
   ```
3. **Database State Verification (PowerShell SQLite Query):**
   ```powershell
   sqlite3 "$env:USERPROFILE\.antigravity_tools\data\backup-prompts\backup-prompts.db" "SELECT id, prompt_id, has_images, is_restored, instance_id FROM prompt_backups ORDER BY created_at DESC LIMIT 5;"
   ```
4. **End-to-End Log Inspection:**
   - Review log output in `~/.antigravity_tools/logs/antigravity-manager.log` during a simulated account rotation.
   - Confirm all 7 stages in Section 4 are logged without gaps or panic traces.
