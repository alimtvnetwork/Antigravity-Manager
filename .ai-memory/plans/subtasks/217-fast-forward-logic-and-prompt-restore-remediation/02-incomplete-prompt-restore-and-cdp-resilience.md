# Subtask 02: Incomplete Prompt Restore & CDP Resilience
Status: COMPLETED

## Objectives
1. Update `src-tauri/src/modules/backup_prompts_db/restore.rs:351-353`:
   - [x] `is_live_status_preserved(status)` must ONLY return true for `Some("completed")`.
   - [x] Prompts left in `running` or `dispatched` status when the IDE terminated or quota died must be re-injected as `backed_up` and dispatched.
2. Update unit test `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs`:
   - [x] Verify `!is_live_status_preserved(Some("running"))` and `is_live_status_preserved(Some("completed"))`.
3. In `src-tauri/src/modules/repo_db/backup.rs`:
   - [x] Update `sweep_incomplete_prompts_for_instance` default instance query to match `(instance_id IN ('default', '__default__', '') OR instance_id IS NULL)`.
4. In `src-tauri/src/modules/repo_db/auto_resume.rs`:
   - [x] Expand query to `status IN ('queued', 'pending', 'backed_up', 'running', 'in_flight', 'dispatched', 'blocked')`.
5. In `src-tauri/src/modules/instance/cdp_onboarding.rs:626`:
   - [x] Add an adaptive retry ladder in `submit_prompt_via_cdp_for_instance` (up to 4 attempts with 400ms sleep) so CDP waits for the Electron DevTools port and conversation webview to stabilize.

## Verification Criteria
- [x] AppError propagation or `crate::error::record_ignored` with `// Justification:`.
- [x] Zero builds and zero tests during code modification.
