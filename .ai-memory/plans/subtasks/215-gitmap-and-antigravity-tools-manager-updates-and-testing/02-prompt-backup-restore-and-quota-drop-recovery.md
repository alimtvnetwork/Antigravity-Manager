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
