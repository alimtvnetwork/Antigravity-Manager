# Subtask 219-02: Account Auto-Switching & Prompt Restore Resiliency
Task ID: `Imp-721`
Sequence: 219
Step: 307
Status: COMPLETED

## Scope
- Focus on prompt restore reliability during auto-switch and quota exhaustion.
- Transition all uncompleted prompts (`running`, `dispatched`, `in_flight`, `pending`) to `backed_up` without arbitrary time windows.
- Re-attribute unrestored prompts to the target instance during switch.
- Return honest `RestoreOutcome` and enforce unscoped fallback when scoped restore yields 0 records.

## Execution Evidence
1. **Incomplete-Prompt Sweep Engine (`repo_db/backup.rs`)**:
   - Verified `sweep_incomplete_prompts_for_instance`: queries all active prompts whose status is NOT in `('completed', 'succeeded', 'backed_up')` in strict FIFO order (`ORDER BY created_at ASC, id ASC`) and transitions them to `backed_up` with zero arbitrary time windows.
   - Sweep is triggered automatically in `auto_switcher/rotation.rs` (pre-switch), `instance/switch.rs` (pre-switch stage 1 snapshot), and `instance/workspace.rs` (post-launch stage 5 restore).
2. **Cross-Instance Prompt Re-Attribution (`instance/switch.rs` & `auto_switcher/rotation.rs`)**:
   - Both manual and automated instance switch routines update unrestored rows in `active_prompts` (`WHERE status = 'backed_up'`) and `prompt_backups` (`WHERE is_restored = 0`), re-attributing them from source instance to target instance ID.
3. **Honest `RestoreOutcome` Telemetry (`backup_prompts_db/restore.rs`)**:
   - Verified `RestoreOutcome` struct provides `{ restored_count, dispatched_count, failed_count, skipped_live, last_error }` with complete accessor methods and honest reporting without swallowing failures.
4. **Live-Status Deduplication Invariant (`backup_prompts_db/restore.rs` & `repo_db/prompts_crud.rs`)**:
   - In `restore.rs`, `is_live_status_preserved` skips overwriting live rows that are already `completed` (`outcome.record_skipped_live()`).
   - In `prompts_crud.rs`, `save_or_requeue_prompt` verifies live status in SQLite and preserves live prompts in `running` and `completed` states from being overwritten by incoming `backed_up` or `queued` records.
5. **Scoped-Restore Fallback (`backup_prompts_db/restore.rs`)**:
   - Verified `restore_running_prompts_scoped_with_fallback_with_outcome`: if scoped restore returns 0 records and unrestored records exist under other instances, it falls back to unscoped restore with informative logging.
6. **Unit Test Suite Integrity (`backup_prompts_db/tests_prompt_restore_honesty.rs`)**:
   - Verified comprehensive coverage across all 5 test scenarios:
     - `test_restore_outcome_honesty_struct_and_accessors`
     - `test_live_status_dedup_preserves_completed_only`
     - `test_scoped_restore_fallback_when_scoped_returns_zero_and_other_exists`
     - `test_reattribution_of_unrestored_prompt_backups_during_switch`
     - `test_incomplete_prompt_sweep_selection_and_status_transition`
