---
plan: 230-running-prompt-restore-after-account-switch
subtask: "01"
title: Running Prompt Restore Architecture, Dual-Store Synchronization & Gating Contracts
domain: backend-rust
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/230-running-prompt-restore-after-account-switch/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/instance/switch.rs
  - src-tauri/src/modules/integration/system_impl.rs
  - src-tauri/src/modules/repo_db/backup.rs
  - src-tauri/src/modules/backup_prompts_db/restore.rs
status: pending
---

# Subtask 01: Running Prompt Restore Architecture, Dual-Store Synchronization & Gating Contracts

## 1. Context & Motivation

In Antigravity Manager, switching accounts (manually via GUI or automatically when quota is exhausted) must adhere to the **Zero Prompt Loss** invariant. When an account switch occurs:
1. In-flight prompts and active agent work must be captured prior to terminating the IDE process.
2. Credentials and OS keyring entries are rewritten while the process is verified dead.
3. The IDE is relaunched with the new account identity.
4. Following a 10-second post-launch settling period (to allow IDE IPC and webview channels to boot), backed-up prompts must be re-injected into the dispatch queue in strict First-In-First-Out (FIFO) order.

However, three distinct defects broke this lifecycle:
- **Defect A (`switch.rs:676`)**: `list_prompt_backups(Some(&target_inst_id), None)` was invoked with an `instance_id` instead of a `batch_id`. In SQLite, `WHERE backup_batch_id = ?` returned 0 rows because batch IDs are UUID-based strings (`"batch_..."`). As a consequence, `has_unrestored_backups` evaluated to `false`, causing `channel_wait` to evaluate to `false` and skipping Stage 5 restoration.
- **Defect B (`system_impl.rs:104, 279`)**: Manual desktop account switch discarded the return value of `backup_active_running_prompts_for_instance`, checking only `repo_db::count_backed_up_prompts("default")`. If prompts were stored in `backup_prompts.db`, `needs_reinject` evaluated to `false`, logging `"No backed-up prompt; skipping prompt restoration"`.
- **Defect C (`repo_db/backup.rs:400`)**: `count_backed_up_prompts` omitted `instance_id = '' OR instance_id IS NULL` for the default instance, rendering default prompts with blank or null instance IDs invisible to the restore trigger.

This subtask defines the exact contracts, symbol signatures, and surgical code modifications required to eliminate these defects and establish permanent safeguards.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Functions | Planned Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/instance/switch.rs` | `switch_instance` (Stage 5 restore trigger) | Replace mismatched `list_prompt_backups(Some(&target_inst_id))` call with `count_unrestored_prompts_for_instance(Some(&target_inst_id))` and `count_unrestored_prompts_for_other_instances(&target_inst_id, None)`. Compute `total_pending_restore` using dual-store maximum. |
| `src-tauri/src/modules/integration/system_impl.rs` | `SystemManager::Desktop::switch_account` (Step 1 & Step 5) | Capture return tuple from `backup_active_running_prompts_for_instance`. Query both `count_unrestored_prompts_for_instance(Some("default"))` and `repo_db::count_backed_up_prompts("default")` to compute `needs_reinject`. Dynamically re-evaluate in Step 5 before scheduling restoration. |
| `src-tauri/src/modules/repo_db/backup.rs` | `count_backed_up_prompts` | Update SQL query predicate for default instance to explicitly include `OR instance_id = '' OR instance_id IS NULL`. |
| `src-tauri/src/modules/backup_prompts_db/restore.rs` | `count_unrestored_prompts_for_instance`, `restore_running_prompts_for_instance_with_outcome` | Verify FIFO order queries (`ORDER BY created_at ASC, id ASC`) and ensure honest counts (`RestoreOutcome`) are logged and surfaced cleanly. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Repair Restore Gating in `src-tauri/src/modules/instance/switch.rs`

In `switch.rs` around lines 672–687:
1. Replace the call to `list_prompt_backups(Some(&target_inst_id), None)` with direct count helpers:
   ```rust
   let target_inst_id = instance.id.clone();
   let target_backed_up = crate::modules::repo_db::count_backed_up_prompts(&target_inst_id);
   let target_unrestored = crate::modules::backup_prompts_db::count_unrestored_prompts_for_instance(
       Some(&target_inst_id),
   )
   .unwrap_or(0);
   let other_unrestored = crate::modules::backup_prompts_db::count_unrestored_prompts_for_other_instances(
       &target_inst_id,
       None,
   )
   .unwrap_or(0);

   let total_pending_restore = backed_up_count.max(target_backed_up).max(target_unrestored);
   let has_unrestored_backups = target_unrestored > 0 || other_unrestored > 0;
   let channel_wait = crate::modules::repo_db::needs_prompt_channel_wait(total_pending_restore)
       || has_unrestored_backups;
   ```
2. Ensure logging clearly outputs the breakdown:
   ```rust
   crate::modules::logger::log_info(&format!(
       "[INSTANCE_SWITCH:STAGE_5_RESTORE] Evaluation for instance '{}': total_pending={}, target_unrestored={}, other_unrestored={}, channel_wait={}",
       target_inst_id, total_pending_restore, target_unrestored, other_unrestored, channel_wait
   ));
   ```

### 3.2 Step 2: Establish Dual-Store Awareness in `src-tauri/src/modules/integration/system_impl.rs`

In `system_impl.rs`:
1. In Step 1 (lines 88–106), retain error visibility without swallowing backup results:
   ```rust
   let newly_backed_up = match crate::modules::repo_db::backup_running_prompts("default") {
       Ok(c) => c,
       Err(e) => {
           crate::modules::logger::log_warn(&format!(
               "[Desktop] Failed to backup running prompts from repo_db: {}", e
           ));
           0
       }
   };

   let db_backup_count = match crate::modules::backup_prompts_db::backup_active_running_prompts_for_instance(
       Some("default"),
       None,
   ) {
       Ok((_info, recs)) => recs.len(),
       Err(e) => {
           crate::modules::logger::log_warn(&format!(
               "[Desktop] Failed to backup active prompts into backup DB: {}", e
           ));
           0
       }
   };

   let repo_backed_up = crate::modules::repo_db::count_backed_up_prompts("default");
   let unrestored_count = crate::modules::backup_prompts_db::count_unrestored_prompts_for_instance(
       Some("default"),
   )
   .unwrap_or(0);

   let total_pending = newly_backed_up.max(db_backup_count).max(repo_backed_up).max(unrestored_count);
   let needs_reinject = crate::modules::repo_db::needs_prompt_channel_wait(total_pending)
       || unrestored_count > 0;

   crate::modules::logger::log_info(&format!(
       "[Desktop] [Step 1/5] Prompt backup completed: repo_db={}, backup_db={}, unrestored={}, needs_reinject={}",
       repo_backed_up, db_backup_count, unrestored_count, needs_reinject
   ));
   ```

2. In Step 5 (lines 276–290), re-verify unrestored counts prior to deciding whether to schedule restoration:
   ```rust
   let final_unrestored = crate::modules::backup_prompts_db::count_unrestored_prompts_for_instance(Some("default"))
       .unwrap_or(0);
   let final_repo_backed_up = crate::modules::repo_db::count_backed_up_prompts("default");
   let final_needs_reinject = needs_reinject || final_unrestored > 0 || final_repo_backed_up > 0;

   if final_needs_reinject {
       crate::modules::logger::log_info(&format!(
           "[Desktop] [Step 5/5] Scheduling post-launch prompt restore for instance default (repo: {}, backup_db: {})",
           final_repo_backed_up, final_unrestored
       ));
       crate::modules::instance::schedule_prompt_restore_after_launch("default".to_string());
       note_prompt_reinjected(true);
   } else {
       crate::modules::logger::log_info(
           "[Desktop] [Step 5/5] No backed-up prompt; skipping prompt restoration",
       );
       note_prompt_reinjected(false);
   }
   ```

### 3.3 Step 3: Normalize Default Instance Query in `src-tauri/src/modules/repo_db/backup.rs`

In `repo_db/backup.rs:393-405`:
Update `count_backed_up_prompts` to properly account for empty string and null instance IDs:
```rust
pub fn count_backed_up_prompts(instance_id: &str) -> usize {
    let Ok(conn) = connect_db() else {
        return 0;
    };
    let is_default = instance_id == "default" || instance_id == "__default__" || instance_id.is_empty();
    let sql = if is_default {
        "SELECT COUNT(*) FROM active_prompts
         WHERE status = 'backed_up'
           AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL)"
    } else {
        "SELECT COUNT(*) FROM active_prompts
         WHERE status = 'backed_up' AND instance_id = ?1"
    };

    if is_default {
        conn.query_row(sql, [], |row| row.get::<_, i64>(0)).unwrap_or(0) as usize
    } else {
        conn.query_row(sql, [instance_id], |row| row.get::<_, i64>(0)).unwrap_or(0) as usize
    }
}
```

---

## 4. Invariants & Operational Rules

1. **Pre-Switch Snapshot Precedence**: Snapshot creation (`sweep_incomplete_prompts_for_instance`, `backup_active_running_prompts_for_instance`) MUST strictly execute to completion before any call to `close_instance_verified`, `force_terminate_pid_win32`, or `terminate_agy_workers`.
2. **FIFO Restore Ordering**: All queries re-injecting prompts into `active_prompts` MUST use:
   ```sql
   ORDER BY created_at ASC, id ASC
   ```
3. **No Silent Errors Gate**:
   - Swallowing `Result` errors via unlogged `.ok()` or empty `catch {}` is forbidden.
   - Any best-effort operation must route through `crate::error::record_ignored(result, "context")` with an accompanying `// Justification:` comment.
4. **Zero Path Leaks**:
   - Absolute filesystem paths (`D:\...`, `C:\...`, `/home/...`) are forbidden in code, logs, and documentation. All paths must be repository-relative.
5. **Zero Intermediate Builds/Tests During Spec Authoring**:
   - Adhere strictly to the pre-execution planning phase without invoking `cargo build`, `cargo test`, or `npm test`.

---

## 5. Verification & Acceptance Criteria

- [ ] `switch.rs`: Evaluates `count_unrestored_prompts_for_instance` and `count_unrestored_prompts_for_other_instances` directly, with zero calls to `list_prompt_backups` with an instance ID.
- [ ] `system_impl.rs`: Checks both `repo_db` and `backup_prompts_db` counts before deciding `needs_reinject`.
- [ ] `repo_db/backup.rs`: `count_backed_up_prompts("default")` returns accurate counts for rows where `instance_id` is empty string or `NULL`.
- [ ] Code compiles cleanly under `cargo check` during execution phase.
- [ ] Error handling complies with `AppError` and "No Silent Errors" rules.
