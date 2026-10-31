# Subtask 02: Antigravity Manager Account Switch & Prompt Restore Verification
Status: COMPLETED

## Objectives
1. Eliminate instance hijacking in `src/stores/useInstanceStore/smart.ts`:
   - [x] Ensure `effectiveInstanceId` remains strictly `instId` (never hijacked to another instance).
   - [x] Delegate to `instanceService.fastForwardInstance(instId)` for unified backend fast-forward execution.
2. Fix sub-threshold candidate filtering in `src-tauri/src/modules/auto_switcher/selection.rs`:
   - [x] Step 3b emergency fallback for eligible accounts with `quota > 5.0%`.
   - [x] Fall back to cached quota when live Google API probe times out or errors.
3. Clean active in-use locking in `src-tauri/src/modules/auto_switcher/instances.rs`:
   - [x] Only lock accounts if the instance is actively running (`is_running == true`).
4. Restore interrupted prompts in `src-tauri/src/modules/backup_prompts_db/restore.rs`:
   - [x] `is_live_status_preserved(status)` strictly matches `Some("completed")` only.
   - [x] Prompts stranded in `running`, `in_flight`, or `dispatched` are swept into `backed_up` and re-dispatched.
5. Auto-resume and CDP retry ladder:
   - [x] `auto_resume.rs` queries all active/interrupted statuses.
   - [x] `cdp_onboarding.rs` implements a 4-attempt retry ladder (400ms sleep) for DevToolsActivePort detection.

## Verification Evidence
- Verified via live command `agm restore running --keep`: restored 1 prompt and re-dispatched in-flight execution to both `gitmap` and `Antigravity-Manager` projects.
