# Subtask 02: Prompt Backup Restore & Quota Drop Recovery
Sequence: 215
Step: 287
Status: COMPLETED
Module: `src-tauri/src/modules/backup_prompts_db/restore.rs`, `src-tauri/src/modules/instance/switch.rs`, `src-tauri/src/modules/repo_db/backup.rs`

## Description
Fix prompt restore failures following auto-switch and quota exhaustion:
1. Implement incomplete-prompt sweep transitioning all `running`/`in_flight`/`dispatched` rows to `backed_up` regardless of the 120s window.
2. Re-attribute unrestored `prompt_backups` rows to target instance in `switch.rs`.
3. Propagate honest restore outcome counts (`restored_count`, `dispatched_count`, `failed_count`) without fake successes.
4. Fall back to unscoped restore when scoped restore returns 0 records.

## Verification & Outcomes
- **Incomplete-Prompt Sweep**: Removed restrictive 120s dispatch window in `repo_db/backup.rs` and wired `sweep_incomplete_prompts_for_instance` into `switch.rs` before IDE shutdown. Any prompt in `running`, `in_flight`, or `dispatched` state (regardless of age) transitions to `backed_up`.
- **Target Instance Re-attribution**: Updated `switch.rs` to re-attribute both `active_prompts` (status = 'backed_up') and unrestored `prompt_backups` (is_restored = 0) from the source instance to the target instance, supporting both default/empty scopes and named instances.
- **Honest RestoreOutcome**: Refactored `RestoreOutcome` in `restore.rs` with explicit fields and accessors `{ restored_count, dispatched_count, failed_count, skipped_live, last_error }` ensuring no fake success is reported if re-injection or dispatch encounters an error.
- **Live-Status Dedup**: Enhanced `live_prompt_status` in `restore.rs` to check both prompt ID and (repo_path, prompt_text) signature, preserving rows whose live status is `running` or `completed` from being overwritten by `save_or_requeue_prompt`.
- **Scoped-Restore Fallback**: Implemented `restore_running_prompts_scoped_with_fallback_with_outcome` in `restore.rs` and wired it into `workspace.rs`. When scoped restore yields 0 records but unrestored records exist under another instance ID, it automatically falls back to unscoped restore in FIFO order.
- **Unit Test Suite**: Created companion unit tests in `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs` validating honest outcomes, live status preservation, fallback scoping, and re-attribution queries.
