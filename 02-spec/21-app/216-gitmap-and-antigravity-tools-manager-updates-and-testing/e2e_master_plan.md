# Master End-to-End Test Plan (Sequence 216)
Slug: `216-gitmap-and-antigravity-tools-manager-updates-and-testing`

## 1. Scope & Execution Rules
- **Non-Negotiable Invariant**: ZERO builds and ZERO tests during code authoring and refactoring.
- **Execution Timing**: Build, test execution, and end-to-end verification are conducted only after all code improvements are finalized.
- **Pathing**: Exclusively relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).

---

## 2. Test Case Scenarios

### Test Case `E2E-216-01`: Account Auto-Switch Prompt Restore on Quota Exhaustion
- **Target Modules**: `src-tauri/src/modules/auto_switcher/rotation.rs`, `src-tauri/src/modules/repo_db/backup.rs`, `src-tauri/src/modules/backup_prompts_db/restore.rs`
- **Objective**: Verify that prompts in any incomplete state (`running`, `in_flight`, `dispatched`, `pending`, `queued`, `blocked`) survive account rotation when quota is exhausted.
- **Execution Steps**:
  1. Populate `active_prompts` with 4 records having statuses `'running'`, `'queued'`, `'pending'`, and `'blocked'`.
  2. Trigger auto-switch rotation via `execute_profile_rotation_with_context`.
  3. Verify Step 0 sweeps all 4 prompts to `'backed_up'`.
  4. Verify Step 3 schedules delayed restore on the target instance (`schedule_prompt_restore_after_launch`).
  5. In `restore.rs`, verify `is_live_status_preserved` only preserves `completed`, allowing all 4 prompts to be re-injected.
  6. Confirm honest outcome reporting: `restored == 4`, `dispatched >= 1`, `failed == 0`.

---

### Test Case `E2E-216-02`: In-Memory Dispatch Cache Bypass for Resend Candidates
- **Target Module**: `src-tauri/src/modules/repo_db/resend.rs`
- **Objective**: Verify that an interrupted prompt whose signature was previously placed in `DISPATCHED_PROMPTS_CACHE` is not falsely skipped as a duplicate during a resend or post-switch recovery.
- **Execution Steps**:
  1. Add a prompt ID and signature to `get_dispatched_prompts_cache()`.
  2. Set the prompt status in `active_prompts` to `'running'`.
  3. Invoke `resend_running_commands_for_instance`.
  4. Verify that `reset_dispatched_prompts_cache()` clears stale signatures.
  5. Verify that `already_dispatched` treats `"backed_up" | "running" | "pending" | "queued"` as `false`, preventing the prompt from being skipped.
  6. Verify the prompt proceeds to dispatch via `spawn_or_replace_prompt_via_agy` and updates to `'dispatched'`.

---

### Test Case `E2E-216-03`: Rerun Button Randomized Auto-Pick & Option Selection Persistence
- **Target Modules**: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `usePromptTreePromptActions.ts`, `PromptTreeViewModal.tsx`
- **Objective**: Verify that clicking the Rerun button defaults to `'randomized'`, provides discrete choices (`is-it-finished`, `is-it-released`, `is-it-done`, `is-it-published`), and persists user preference across UI views.
- **Execution Steps**:
  1. Mount `PromptTreeViewModal` and inspect `handleStripRerun`.
  2. Verify `handleStripRerun` invokes `loadPersistedRerunMessageOption()` rather than a hardcoded option.
  3. Verify default persisted option is `'randomized'`.
  4. Select option `'is-it-finished'`, verify persistence in localStorage.
  5. Verify `RERUN_RANDOM_MESSAGE_SET` contains 18 distinct completion/release check nudges.

---

### Test Case `E2E-216-04`: GitMap Command Execution Suite
- **Target Subsystem**: GitMap CLI (`d:/work/gitmap`)
- **Objective**: Verify high-speed responsiveness across GitMap CLI commands.
- **Execution Steps**:
  1. Execute `gitmap status`: verify multi-repo scanning across fleet.
  2. Execute `gitmap find 02-spec/21-app/`: verify sub-30ms execution.
  3. Execute `gitmap cat`: verify bounded pagination and content output.
  4. Execute `gitmap aum search`: verify indexed multi-term search.

---

### Test Case `E2E-216-05`: Cross-Repository Coding Guidelines & Invariant Verification
- **Target Repositories**: `Antigravity-Manager` and `gitmap`
- **Objective**: Verify strict compliance with repository-wide coding standards.
- **Execution Steps**:
  1. Verify zero explicit boolean comparisons (`== true` / `== false`).
  2. Verify positive boolean naming throughout.
  3. Verify function line count discipline (8–15 lines target, bounded limits).
  4. Verify 100% repository-relative Git paths in all commits, specs, and plans.
  5. Verify leaf error sovereignty: all fallible paths propagate via structured errors with zero silent suppression.
