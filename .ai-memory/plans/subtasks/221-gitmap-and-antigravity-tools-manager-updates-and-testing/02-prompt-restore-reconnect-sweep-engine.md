# Subtask 02: Prompt Restore Reconnect Sweep & Quota Interruption Recovery
Parent: `221-gitmap-and-antigravity-tools-manager-updates-and-testing.md`
Step: 317
Improvement: `Imp-731`
Status: PENDING

## Scope & Objective
In `src-tauri/src/prompts/restore.rs` and `src-tauri/src/auto_switcher/rotation.rs`:
- Implement `reconcile_and_sweep_interrupted_prompts` to safely detect prompts interrupted by quota finish or process crash.
- Ensure uncompleted prompt backups are reattributed to the active instance ID and queued with strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).
- Add tests in `src-tauri/tests/tests_prompt_restore_honesty.rs` for reconnect sweeping and reattribution without running tests yet.

## Invariants
- `ORDER BY created_at ASC, id ASC` strictly enforced.
- Zero builds / zero tests during authoring.
