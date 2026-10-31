# Subtask 02: Prompt Backup Restore & Quota Drop Recovery
Sequence: 216
Step: 292
Status: COMPLETED
Module: `src-tauri/src/modules/backup_prompts_db/restore.rs`, `src-tauri/src/modules/instance/switch.rs`, `src-tauri/src/modules/repo_db/backup.rs`, `src-tauri/src/modules/auto_switcher/rotation.rs`

## Description
Harden account auto-switching, prompt backup, restore, and recovery when account quota is exhausted. Ensure prompts stranded in non-terminal states (`running`, `in_flight`, `dispatched`, `pending`, `queued`, `blocked`) are swept, backed up, and re-dispatched into the new account session.

## Implementation Details
1. Verify `sweep_incomplete_prompts_for_instance` captures all incomplete states (`running`, `in_flight`, `dispatched`, `pending`, `queued`, `blocked`).
2. Guarantee that `is_live_status_preserved` in `restore.rs` strictly skips only `completed` rows, ensuring prompts from terminated IDE sessions are always re-injected.
3. Verify `rotation.rs` Step 0 pre-backup sweep and Step 3 target post-launch restore dispatch schedule.
4. Verify CLI parity for prompt backup and restore (`agm restore-running-prompts`).

## Done When
- Quota depletion auto-switch prompt continuity is structurally verified with zero prompt loss paths.
