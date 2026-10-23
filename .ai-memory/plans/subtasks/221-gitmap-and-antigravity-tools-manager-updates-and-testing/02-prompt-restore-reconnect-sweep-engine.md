# Subtask 02: Prompt Restore Reconnect Sweep & Quota Interruption Recovery
Parent: `221-gitmap-and-antigravity-tools-manager-updates-and-testing.md`
Step: 317
Improvement: `Imp-731`
Status: COMPLETED

## Scope & Objective
In `src-tauri/src/prompts/restore.rs` (facade for `src-tauri/src/modules/backup_prompts_db/restore.rs`) and `src-tauri/src/modules/auto_switcher/rotation.rs`:
- Implement `sweep_and_reattribute_orphaned_backups` to safely detect prompts interrupted by quota finish or process crash and reattribute them to the current instance ID.
- Ensure uncompleted prompt backups are reattributed to the active instance ID and queued with strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).
- Ensure `reinject_single_record` sets `status = 'pending'` or `in_progress` according to restore protocol and logs recovery via `tracing::info!`.
- Ensure rotation and reconnect handovers in `rotation.rs` and `workspace.rs` invoke `sweep_and_reattribute_orphaned_backups`.
- Add test case `test_sweep_and_reattribute_orphaned_backups` in both `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs` and `src-tauri/tests/tests_prompt_restore_honesty.rs`.

## Invariants
- `ORDER BY created_at ASC, id ASC` strictly enforced.
- Zero builds / zero tests during authoring strictly observed.
- Worker Git Ban strictly observed (no commits, pushes, or adds).
- No silent errors: all fallible operations mapped to `AppError` or logged with `crate::error::record_ignored`.

## Verified File Changes
1. `src-tauri/src/modules/backup_prompts_db/restore.rs`:
   - Added `SqlitePool` alias for `rusqlite::Connection`.
   - Implemented `sweep_and_reattribute_orphaned_backups(pool: &SqlitePool, current_instance_id: &str) -> Result<usize, AppError>`.
   - Implemented `sweep_and_reattribute_orphaned_backups_with_file`.
   - Updated `reattribute_unrestored_prompt_backups` to invoke `sweep_and_reattribute_orphaned_backups`.
   - Updated `reinject_single_record` to set restored status ('pending' or 'in_progress') and log via `tracing::info!`.
   - Updated `prompt_status_after_restore` to map 'running' | 'in_progress' to 'in_progress', 'queued' to 'queued', and other statuses to 'pending'.
2. `src-tauri/src/prompts/restore.rs` & `src-tauri/src/prompts/mod.rs`:
   - Canonical facade re-exporting `crate::modules::backup_prompts_db::restore::*`.
3. `src-tauri/src/lib.rs`:
   - Registered `pub mod prompts;`.
4. `src-tauri/src/modules/auto_switcher/rotation.rs`:
   - Invoked `sweep_and_reattribute_orphaned_backups` during cross-instance rotation and during Step 3 restore handover.
5. `src-tauri/src/modules/instance/workspace.rs`:
   - Invoked `sweep_and_reattribute_orphaned_backups` on prompt channel ready in `push_backed_up_prompts_after_channel`.
6. `src-tauri/src/modules/repo_db/dispatch.rs`:
   - Updated `dispatch_running_prompts` query and duplicate check to include statuses `backed_up`, `pending`, and `in_progress`.
7. `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs` & `src-tauri/tests/tests_prompt_restore_honesty.rs`:
   - Added unit and integration test `test_sweep_and_reattribute_orphaned_backups` validating out-of-order orphaned records are reattributed and restored in strict FIFO order (`created_at ASC, id ASC`).
