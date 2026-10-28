# Subtask 04: Instance Switching & Prompt Restoring E2E Verification

## Target Files
- `src-tauri/src/modules/account/switch.rs`
- `src-tauri/src/modules/repo_db/backup.rs`
- `src-tauri/src/modules/repo_db/restore.rs`
- `tests/test_instance_switch_fifo_restore.rs` or integration test

## Objectives
1. Verify instance switching and FIFO prompt restore behavior:
   - Ensure `restore_backed_up_prompts_for_instance` preserves `ORDER BY created_at ASC, id ASC`.
   - Ensure settling delay (10s) and restore status progression behave reliably.
2. Verify cross-instance prompt isolation:
   - Switching instance `A` does not mutate or restore prompts for instance `B`.
   - Caller IDE `v3-7391` is untouched.
