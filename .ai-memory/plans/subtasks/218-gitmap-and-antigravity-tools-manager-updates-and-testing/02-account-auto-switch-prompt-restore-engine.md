# Subtask 218-02: Account Auto-Switching & Prompt Restore Resiliency
Task ID: `Imp-716`
Sequence: 218
Step: 302
Status: COMPLETED

## Scope
- Focus on prompt restore reliability during auto-switch and quota exhaustion.
- Transition all uncompleted prompts (`running`, `dispatched`, `in_flight`, `pending`) to `backed_up` without arbitrary time windows.
- Re-attribute unrestored prompts to the target instance during switch.
- Return honest `RestoreOutcome` and enforce unscoped fallback when scoped restore yields 0 records.

## Completion Evidence
- **Incomplete-Prompt Sweep**: Updated `select_incomplete_prompt_ids` and `sweep_incomplete_prompts_for_instance` in `src-tauri/src/modules/repo_db/backup.rs` to select all uncompleted prompts via `WHERE status NOT IN ('completed', 'succeeded', 'backed_up')` and sweep them to `backed_up` without restrictive time windows or age limits, properly handling default instance scoping (`instance_id IN ('default', '__default__') OR instance_id IS NULL OR instance_id = ''`).
- **Backup Window Removal**: In `src-tauri/src/modules/backup_prompts_db/backup.rs`, eliminated the restrictive 2-hour freshness window so long-running or long-queued prompts are never dropped during quota depletion or switches.
- **Re-attribution Resiliency**: In `src-tauri/src/modules/auto_switcher/rotation.rs`, hardened prompt re-attribution queries across `active_prompts` and `prompt_backups` for default instance scopes and routed fallible operations through `crate::error::record_ignored` with justification comments.
- **Honest RestoreOutcome**: Extended `RestoreOutcome` in `src-tauri/src/modules/backup_prompts_db/restore.rs` with `skipped_live_count()` accessor and `with_details(...)` constructor, preserving accurate counts for `{ restored_count, dispatched_count, failed_count, skipped_live, last_error }` with zero fake success.
- **Live-Status Dedup Invariant**: Enforced live-status dedup in both `restore.rs` and `save_or_requeue_prompt` (`src-tauri/src/modules/repo_db/prompts_crud.rs`) preventing overwriting of active `running` or `completed` rows with `backed_up`/`queued`.
- **FIFO Restore Invariant**: Enforced `ORDER BY created_at ASC, id ASC` across all prompt backup restore queries in `restore.rs`.
- **Companion Test Suite**: Added `test_incomplete_prompt_sweep_selection_and_status_transition` and enhanced `test_restore_outcome_honesty_struct_and_accessors` in `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs`.
