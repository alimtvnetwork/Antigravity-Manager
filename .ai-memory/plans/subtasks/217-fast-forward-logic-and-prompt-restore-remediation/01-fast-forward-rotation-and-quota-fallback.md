# Subtask 01: Fast-Forward Rotation & Quota Fallback Harmonization
Status: COMPLETED

## Objectives
1. Eliminate instance hijacking in `src/stores/useInstanceStore/smart.ts:314-325`:
   - [x] Never override `effectiveInstanceId` with another instance ID.
   - [x] Never close `instId` and switch another instance just because an account was previously used there.
2. Unify Fast-Forward execution:
   - [x] In `smartRotateProfileAccount`, delegate directly to `instanceService.fastForwardInstance(instId)` so all UI buttons and CLI commands trigger the exact same backend workflow.
3. Fix sub-threshold candidate filtering in `src-tauri/src/modules/auto_switcher/selection.rs`:
   - [x] In Step 3 fallback pass of `select_candidate_profiles`, when no accounts meet `quota > threshold`, allow any healthy account with `quota > 5.0` to be eligible.
   - [x] In `select_and_verify_next_best_profile`, when live quota refresh exceeds 5000ms or fails, fall back to the account's existing cached quota if `> 5.0`, recording a warning rather than skipping the candidate.
4. Clean active in-use account locking in `src-tauri/src/modules/auto_switcher/instances.rs`:
   - [x] Restrict `get_active_in_use_account_ids()` to accounts bound to instances that are actively running (`is_running == true`).

## Verification Criteria
- [x] Bounded functions (8-15 lines).
- [x] Positive booleans.
- [x] AppError propagation or `crate::error::record_ignored` with `// Justification:`.
- [x] Zero builds and zero tests during code modification.
