# Subtask 02: Account Auto-Switch Prompt Restore Engine & Quota Exhaustion Remediation
Status: COMPLETED
Sequence: 220
Step: 312
Improvement: `Imp-726`
Owner: Worker 01

## Objectives
- Resolve prompt restore failure during account auto-switch when an account finishes credit / quota.
- Ensure all prompts in `pending` or `running` mode that were abruptly closed during quota finish are audited and recovered.
- Implement sweep transition in `backup_prompts_db/restore.rs` ensuring orphaned or interrupted prompts are restored to `queued` with proper instance reattribution and strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).
- Add comprehensive honesty unit tests covering quota exhaustion scenarios in `tests_prompt_restore_honesty.rs`.

## Disjoint File Boundaries
- `src-tauri/src/modules/backup_prompts_db/restore.rs`
- `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs`
- `src-tauri/src/modules/auto_switcher/rotation.rs`

## Invariants
- Zero builds / zero tests during implementation.
- Strictly relative Git paths.
