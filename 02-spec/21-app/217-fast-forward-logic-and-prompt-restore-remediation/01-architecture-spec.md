# Sequence 217 Specification: Fast-Forward Logic Review & Prompt Restore Quota Recovery Remediation

## 1. Executive Summary & Problem Classification
- **Domain**: Antigravity Account Rotation, Fast-Forward (`ff`), Post-Switch Prompt Restore, and Rerun Dispatch Engine.
- **Problem Statement**:
  1. **Fast-Forward Logic Malfunction**:
     - Frontend `smartRotateProfileAccount` (`src/stores/useInstanceStore/smart.ts:314-325`) actively hijacks and terminates the working instance if the candidate account was previously bound to another instance config (`candidateBoundInstance`). It closes the current working IDE and switches the other instance instead of rotating the active instance in place.
     - Divergent execution paths: `fastForwardInstance` (backend IPC) and `smartRotateProfileAccount` (TypeScript client) run disparate candidate selection algorithms.
     - Backend candidate over-filtering: In `src-tauri/src/modules/auto_switcher/selection.rs:383` and `line 767`, fallback candidates are required to have `quota > low_quota_threshold_percent` (e.g. 80%). When no 100% account exists and remaining accounts have 50%–75% quota, all accounts are rejected with `"No alternative healthy profile found in pool"`.
     - In `selection.rs:690-724`, if the 5000ms live quota API refresh times out or encounters transient network failure, the candidate was skipped rather than falling back to cached quota.
     - In `src-tauri/src/modules/auto_switcher/instances.rs:69`, `get_active_in_use_account_ids()` treated non-running instances as in-use simply because `inst.id == active_instance_id`, locking healthy accounts.
  2. **Prompt Restore & Quota Recovery Failure**:
     - In `src-tauri/src/modules/backup_prompts_db/restore.rs:351-353`, `is_live_status_preserved(status)` returned `true` for `Some("running")`. When an account ran out of quota and the IDE terminated, prompts left in `'running'` status were skipped by the restore pass (`outcome.record_skipped_live()`), stranding uncompleted prompts forever.
     - In `src-tauri/src/modules/repo_db/backup.rs:449`, `is_all_or_default` filtered `AND instance_id IN ('default', '__default__')`, ignoring records with `instance_id = ''` or `instance_id IS NULL`.
     - In `src-tauri/src/modules/repo_db/auto_resume.rs:90`, SQL queries only matched `status IN ('queued', 'pending', 'backed_up')`, ignoring interrupted prompts in `'running'`, `'in_flight'`, and `'dispatched'`.
     - In `src-tauri/src/modules/instance/cdp_onboarding.rs:626`, `submit_prompt_via_cdp_for_instance` failed immediately on a single millisecond port probe if Electron had not yet finalized `DevToolsActivePort`.
  3. **Rerun Randomizer Auto-Pick & Template Recognition**:
     - `buildRerunPrompt` in `src/components/instances/PromptTreeView/verificationTemplates.ts:848-882` only recognized `"is it done?"` and `"is it published?"`, failing to recognize `"is it finished?"`, `"is it released?"`, `"is it completed?"`, etc., misclassifying randomized completion nudges as original prompt bodies.
     - Default rerun message option was not guaranteed to be `'randomized'` across all invocations.
     - In `src-tauri/src/commands/cmd_running_up_next.rs`, `query_up_next` and `query_up_next_total` only filtered for `status IN ('queued', 'pending')`, leaving `'backed_up'` prompts invisible in the "Up Next" panel.

---

## 2. Technical Architecture & Surgical Remediation Contract

### 2.1 Fast-Forward Candidate Selection & Rotation Unification
1. **Frontend Fast-Forward Harmonization (`src/stores/useInstanceStore/smart.ts`)**:
   - Eliminate instance hijacking: Delete lines 314-325 where `candidateBoundInstance` overrides `instId`. `effectiveInstanceId` MUST strictly equal `instId`.
   - Harmonize `smartRotateProfileAccount` to invoke the unified backend Fast-Forward command (`instanceService.fastForwardInstance(instId)`), ensuring all UI buttons (Top Navigation Bar `>>`, Global Shortcut `Ctrl+Shift+F`, Settings "Rotate Now", Instance Card, Account Table) and CLI `agm ff` execute the exact same battle-tested backend pipeline.
2. **Backend Quota Fallback & Timeout Resilience (`src-tauri/src/modules/auto_switcher/selection.rs`)**:
   - In Step 3 fallback pass of `select_candidate_profiles`, when no accounts meet `quota > threshold`, allow any healthy account with `quota > 5.0` (headroom > 5%) to be selected.
   - In `select_and_verify_next_best_profile`, when live quota refresh exceeds 5000ms or fails, gracefully fall back to the candidate's existing cached quota `calculate_candidate_quota` if `> 5.0`, recording a warning rather than skipping the candidate.
   - In `src-tauri/src/modules/auto_switcher/instances.rs:69`, restrict `get_active_in_use_account_ids()` to accounts bound to instances that are actively running (`is_running == true`).

### 2.2 Uncompleted Prompt Restoration & CDP Resilience
1. **Durable Incomplete Prompt Recovery (`src-tauri/src/modules/backup_prompts_db/restore.rs`)**:
   - Update `is_live_status_preserved(status)` to ONLY match `Some("completed")`.
   - Stale `running` and `dispatched` rows left by terminated IDE sessions or expired quotas MUST be re-injected into `active_prompts` as `backed_up` and dispatched.
2. **Database Null/Empty Instance ID Hygiene (`src-tauri/src/modules/repo_db/backup.rs` & `auto_resume.rs`)**:
   - In `sweep_incomplete_prompts_for_instance`, update default instance predicates to `(instance_id IN ('default', '__default__', '') OR instance_id IS NULL)`.
   - In `auto_resume.rs`, expand the query to `status IN ('queued', 'pending', 'backed_up', 'running', 'in_flight', 'dispatched', 'blocked')`.
3. **Adaptive CDP Prompt Submission (`src-tauri/src/modules/instance/cdp_onboarding.rs`)**:
   - Add an adaptive retry ladder to `submit_prompt_via_cdp_for_instance` (up to 4 attempts with 400ms sleep) to allow newly spawned Electron instances to initialize `DevToolsActivePort` and conversation webview DOM before giving up.

### 2.3 Rerun Randomizer & Up-Next Visibility
1. **Rerun Nudge Pattern Expansion (`src/components/instances/PromptTreeView/verificationTemplates.ts`)**:
   - Expand `isNudge` check to recognize `"is it finished?"`, `"is it released?"`, `"is it completed?"`, `"finished?"`, `"released?"`, `"completed?"`, etc.
2. **Up-Next Panel Visibility (`src-tauri/src/commands/cmd_running_up_next.rs`)**:
   - Expand `query_up_next` and `query_up_next_total` to `status IN ('queued', 'pending', 'backed_up')` so prompts awaiting execution after an auto-switch appear in the UI strip.

---

## 3. Invariants & Verification Standards
- **Zero Builds / Zero Tests During Code Modifications**: All code modifications are strictly completed without running builds or tests. Verification is deferred to the final phase.
- **No Silent Errors**: All fallible paths propagate via `AppError` or record using `crate::error::record_ignored` with a `// Justification:` comment.
- **Strict FIFO Ordering**: All prompt restore and dispatch queries enforce `ORDER BY created_at ASC, id ASC`.
- **Relative Paths Invariant**: All referenced paths use strictly repository-relative Git paths.
