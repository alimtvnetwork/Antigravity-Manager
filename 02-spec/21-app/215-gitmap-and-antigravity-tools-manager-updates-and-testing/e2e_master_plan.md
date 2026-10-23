# End-to-End Master Execution Plan: Sequence 215

## 1. Batch Objective
Implement and verify updates across GitMap and Antigravity-Manager covering prompt backup and honest restoration on quota auto-switch, rerun randomized option auto-pick and persistence, GitMap code quality cleanup, and deferred end-to-end verification.

---

## 2. Work Breakdown & Subagent Allocations
- **Worker 01 (Task 1 & Task 2)**:
  - Repository audit of last 50-80 commits across AGM and GitMap.
  - AGM Backend: `src-tauri/src/modules/backup_prompts_db/restore.rs`, `src-tauri/src/modules/instance/switch.rs`, `src-tauri/src/modules/repo_db/backup.rs` (incomplete-prompt sweep, honest outcome struct, re-attribution parity).
- **Worker 02 (Task 3 & Task 4)**:
  - AGM Frontend: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `usePromptTreePromptActions.ts`, `ConversationContextMenu.tsx` (rerun randomized default, 18+ prompt set, persistence).
  - GitMap Code Quality: drain completed pending plans in GitMap (`d:/work/gitmap/.ai-memory/plans/pending/`), eliminate boolean `== true` / `== false`, migrate `fmt.Errorf` to AppError.
- **Worker 03 (Task 5 & Master Plan Expansion)**:
  - Author and catalog exhaustive deferred E2E test scenarios (`E2E-215-01` through `E2E-215-05`) for account auto-switch prompt restore, quota depletion recovery, rerun auto-pick, and GitMap command execution.

---

## 3. Invariants
- Zero builds and zero tests during authoring.
- Exclusively relative Git paths.
- No silent errors rule.
- Strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).

---

## 4. Comprehensive End-to-End Test Scenarios

### Test Case `E2E-215-01`: Account Auto-Switch Prompt Restore on Quota Exhaustion
- **Target Modules**: `src-tauri/src/modules/auto_switcher/rotation.rs`, `src-tauri/src/modules/repo_db/backup.rs`, `src-tauri/src/modules/backup_prompts_db/restore.rs`
- **Invariant**: Restore & Queue Invariant (`AGENTS.md`)
- **Execution Steps**:
  1. Enqueue 3 prompts in `active_prompts` with statuses `'running'`, `'queued'`, and `'pending'`.
  2. Simulate quota exhaustion triggering `execute_profile_rotation_with_context`.
  3. Verify Step 0 executes `sweep_incomplete_prompts_for_instance`, transitioning all 3 prompts to `backed_up`.
  4. Verify Step 3 executes `schedule_prompt_restore_after_launch(target_inst_id)`.
  5. In `restore.rs`, verify `is_live_status_preserved` only preserves `completed`, allowing all backed-up prompts to be re-injected and re-dispatched.
  6. Confirm honest outcome reporting: `restored == 3`, `dispatched == 3`, `failed == 0`.

---

### Test Case `E2E-215-02`: Incomplete-Prompt Sweep Beyond 120s Window
- **Target Module**: `src-tauri/src/modules/repo_db/backup.rs`
- **Execution Steps**:
  1. Populate prompts in `active_prompts` with `status = 'dispatched'` and `updated_at` older than 3600 seconds.
  2. Invoke `sweep_incomplete_prompts_for_instance`.
  3. Verify the stale dispatched rows are successfully swept to `backed_up`.
  4. Confirm that post-switch restore successfully queries and dispatches these long-running prompts.

---

### Test Case `E2E-215-03`: Rerun Randomized Auto-Pick by Default & Option Expansion
- **Target Modules**: `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`, `src/components/instances/PromptTreeView/DetailHeader.tsx`
- **Execution Steps**:
  1. Mount `DetailHeader` or open prompt tree context menu with no prior localStorage settings.
  2. Verify `loadPersistedRerunMessageOption()` returns `'randomized'`.
  3. Verify available options in `RERUN_MESSAGE_OPTIONS`:
     - `randomized`: Randomized nudge (random pick)
     - `is-it-done`: Is it done?
     - `is-it-finished`: Is it finished?
     - `is-it-published`: Is it published?
     - `is-it-released`: Is it released?
  4. Verify `RERUN_RANDOM_MESSAGE_SET` contains 18 distinct variations.
  5. Select option `'is-it-released'`, verify persistence in localStorage, reload and confirm persistence.

---

### Test Case `E2E-215-04`: GitMap Command Execution Suite
- **Target Subsystem**: GitMap CLI (`d:/work/gitmap`)
- **Execution Steps**:
  1. Execute `gitmap status` across fleet: verify 69 repositories scanned in parallel.
  2. Execute `gitmap find` across `02-spec/21-app/`: verify sub-30ms execution.
  3. Execute `gitmap cat`: verify bounded pagination and content output.
  4. Execute `gitmap aum search`: verify indexed multi-term search.

---

### Test Case `E2E-215-05`: Cross-Repository Coding Guidelines Compliance
- **Target Modules**: `src-tauri/`, `src/`, `d:/work/gitmap/`
- **Execution Steps**:
  1. Verify zero instances of explicit boolean comparisons (`== true` / `== false`).
  2. Verify function line count discipline (8–15 lines target, max 100 lines).
  3. Verify 100% repository-relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`).
  4. Verify AppError propagation on fallible paths with zero silent errors.
