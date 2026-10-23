# End-to-End Master Execution Plan: Sequence 215 (Steps 286–290)

## 1. Scope & Batch Objectives
This document establishes the comprehensive end-to-end and integration verification plan for the 5 architectural, backend restoration, frontend rerun, CLI code quality, and regression gate modules implemented in Sequence 215 (`Imp-700` through `Imp-704`).

In accordance with the **Zero-Build & Zero-Test Invariant** codified in `AGENTS.md` and repository standards, live test runs are strictly deferred during the implementation phase. Any AI agent or developer executing deferred verification must follow the detailed specifications and exact command lines cataloged in this master plan.

The five focus areas verified in this sequence are:
1. **GitMap PA Fleet Audit & Recent Commits Invariant** (`Imp-700`, Step 286): Complete discovery and remote branch reconciliation across all 68 repositories in the developer workspace, verification of recent commit history invariants, and relative path hygiene.
2. **Auto-Switch Prompt Restore & Quota Drop Recovery** (`Imp-701`, Step 287): Removal of the 120-second window clobber, incomplete-prompt sweep across all uncompleted statuses, durable prompt re-attribution on manual/auto switch, honest `RestoreOutcome` reporting, and unscoped fallback.
3. **Rerun Prompt Randomizer, Enlarged Message Set & Storage Persistence** (`Imp-702`, Step 288): Default selection of `'randomized'`, elevated default count of 4, enlarged 14-variation prompt set, dual localStorage persistence keys with error tracking, and full context-menu integration parity.
4. **GitMap Code Quality & Pending Plans Drain** (`Imp-703`, Step 289): Drainage of completed pending plans into `completed/`, complete elimination of explicit `== true` and `== false` boolean comparisons, and migration of Go CLI error handling to structured `AppError`.
5. **Sequence 215 Master Regression Gate & Zero Silent Errors Invariant** (`Imp-704`, Step 290): Full pre-flight gate verification (`cargo fmt`, `cargo clippy`, `npm run build`), strict relative path enforcement, and zero silent errors audit.

---

## 2. Architectural Invariants Enforced
All test cases strictly enforce the non-negotiable architectural invariants codified in `AGENTS.md`:

- **Architecture & Pipeline First**: System maintains strict separation of concerns. Adapters perform normalization, while the pipeline handles payload sanitization, context alignment, and thinking backfilling.
- **FIFO Restore & Queue Invariant**: All prompt backup, restore, and dispatch queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly prohibited.
- **No Silent Errors Invariant**: Every fallible operation routes into the error module. Bare `.ok()`, swallowed errors, and bare `catch {}` blocks are forbidden. Best-effort operations require `crate::error::record_ignored` along with a mandatory `// Justification:` comment in Rust, and `useErrorStore.getState().trackWarning` in TypeScript.
- **Strictly Relative Git Paths Invariant**: Absolute filesystem paths (`D:\...`, `C:\...`, `/home/...`) must never appear in specifications, plans, commit messages, changelogs, READMEs, or release notes. All file references must be repository-relative Git paths.
- **Honest Outcome Reporting (No Fake Success)**: Operations must never report success when errors occur. Restore outcomes propagate actual counts (`restored`, `dispatched`, `failed`, `skipped_live`) rather than claiming pre-restore counts.
- **Contextual UI & Capsule Styling**: Rerun controls, toolbar actions, and buttons follow segmented pill capsule styling (`rounded-full`, dark-glass aesthetics, shared borders, subtle dividers) with transparent inline select dropdowns.

---

## 3. Step-by-Step Task Mapping & Improvement Ledger

| Step | Improvement ID | Category / Subsystem | Primary Target Files | Master Verification Test Case |
| :--- | :--- | :--- | :--- | :--- |
| **Step 286** | `Imp-700` | GitMap Fleet Audit & Commit Hygiene | `02-spec/21-app/215-gitmap-and-antigravity-tools-manager-updates-and-testing/audit_report.md` | `E2E-215-01` |
| **Step 287** | `Imp-701` | AGM Backend (Restore & Switch) | `src-tauri/src/modules/backup_prompts_db/restore.rs`<br>`src-tauri/src/modules/instance/switch.rs`<br>`src-tauri/src/modules/repo_db/backup.rs` | `E2E-215-02` |
| **Step 288** | `Imp-702` | AGM Frontend (Prompt Tree & Rerun) | `src/components/instances/PromptTreeView/DetailHeader.tsx`<br>`src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`<br>`src/components/instances/PromptTreeView/ConversationContextMenu.tsx` | `E2E-215-03` |
| **Step 289** | `Imp-703` | GitMap Code Quality & Plan Management | `.ai-memory/plans/`<br>`cmd/`<br>`internal/apperror/` | `E2E-215-04` |
| **Step 290** | `Imp-704` | Quality Gates & Regression Verification | Workspace Linters<br>Pre-flight check suites | `E2E-215-05` |
>>>>>>> 2a597bf3 (Feature: Feature: instance - implement rerun randomizer auto-pick prompt restore quota recovery and fleet audit (sequence 215))

---

## 4. Comprehensive End-to-End Test Scenarios

<<<<<<< HEAD
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
=======
### Test Case `E2E-215-01`: GitMap PA Fleet Audit & Recent Commits Invariant
- **Target Module**: GitMap CLI Project Auditor (`gitmap pa`), Repository Scanner, and Git Invariant Validator.
- **Improvement ID**: `Imp-700` (Step 286).
- **Invariant Addressed**: Strictly Relative Git Paths & Commit History Hygiene (`AGENTS.md`).
- **Preconditions**:
  - The developer workspace contains all 68 active and managed repositories.
  - Remote tracking branches (`origin/main`, `origin/beta`) configured for GitMap and Antigravity-Manager.
- **Actions / Execution Steps**:
  1. **Fleet Discovery & Status Reconciliation**:
     - Execute deferred command `gitmap pa`.
     - Verify that all 68 repositories are discovered without timeout or process stalls.
     - Validate output table/JSON categorization into `clean`, `ahead`, `behind`, and `diverged` statuses.
     - Confirm that repositories requiring upstream synchronization are accurately flagged without false positives.
  2. **Recent Commit History Inspection**:
     - Audit the last 50–80 git commits across `Antigravity-Manager` and `gitmap`.
     - Verify that each commit follows conventional atomic formatting with valid imperative commit messages (`fix: ...`, `feat: ...`, `refactor: ...`).
     - Ensure no intermediate debugging churn, temporary broken states, or trial-and-error commits exist.
  3. **Strictly Relative Path Hygiene Check**:
     - Inspect git commit messages, branch references, and generated markdown logs from the commit window.
     - Validate that zero absolute filesystem paths (`D:\...`, `C:\...`, `/home/...`) appear in commit messages, specifications, or audit documentation.
     - Confirm that all repository references use forward-slash, repository-relative notation (`02-spec/...`, `src/...`, `src-tauri/...`).
  4. **Output Formatter Verification**:
     - Test `gitmap pa --json` and verify compliant JSON output adhering to schema standards.
     - Verify that exit code is 0 and no unhandled panics occur.
- **Expected Outcomes**:
  - 100% discovery rate across all 68 repositories in the workspace.
  - Zero absolute filesystem paths detected across all audited commits and logs.
  - Verified atomic commit hygiene across both AGM and GitMap repositories.

---

### Test Case `E2E-215-02`: Auto-Switch Prompt Restore & Quota Drop Recovery
- **Target Module**:
  - `src-tauri/src/modules/backup_prompts_db/restore.rs`
  - `src-tauri/src/modules/instance/switch.rs`
  - `src-tauri/src/modules/repo_db/backup.rs`
  - `src-tauri/src/modules/auto_switcher/rotation.rs`
- **Improvement ID**: `Imp-701` (Step 287).
- **Invariant Addressed**: FIFO Restore & Queue Invariant, No Silent Errors, Honest Outcome Reporting (`AGENTS.md`).
- **Preconditions**:
  - Multi-instance SQLite database setup with test instances `inst-alpha` and `inst-beta`.
  - SQLite tables `active_prompts` and `prompt_backups` seeded with prompts in varied lifecycle states (`running`, `in_flight`, `dispatched`, `queued`, `completed`).
- **Actions / Execution Steps**:
  1. **Incomplete-Prompt Sweep & Elimination of 120s Window Clobber (RC-1)**:
     - Insert a prompt into `active_prompts` with status `dispatched` and `updated_at = now - 7200` (dispatched 2 hours ago, simulating a long-running prompt during quota depletion).
     - Insert prompts with statuses `running` and `in_flight` under `inst-alpha`.
     - Trigger `backup_running_prompts("inst-alpha")`.
     - Query `active_prompts` for `inst-alpha`:
       - Assert that the 2-hour-old `dispatched` prompt transitioned to `backed_up` (proving removal of the 120-second recency clobber).
       - Assert that `running` and `in_flight` prompts transitioned to `backed_up`.
       - Assert that prompts with status `completed` remained untouched.
  2. **Durable Prompt Re-attribution on Switch (RC-2)**:
     - Insert unrestored rows into `prompt_backups` scoped to `instance_id = 'inst-alpha'` with `is_restored = 0`.
     - Execute instance switch from `inst-alpha` to `inst-beta` via `switch_to_instance`.
     - Verify that `switch.rs` executes the re-attribution update:
       `UPDATE prompt_backups SET instance_id = 'inst-beta' WHERE instance_id = 'inst-alpha' AND is_restored = 0`.
     - Assert that unrestored records in `prompt_backups` now belong to `inst-beta`.
  3. **Honest `RestoreOutcome` Struct Verification (RC-3)**:
     - Invoke `restore_running_prompts_for_instance_with_outcome(Some("inst-beta"), false, custom_db)`.
     - Validate that the returned tuple `(records, outcome)` contains accurate counts:
       - `outcome.restored` matches the exact number of prompts re-injected into `active_prompts`.
       - `outcome.dispatched` matches the number of prompts actually sent by `resend_running_commands_for_instance`.
       - `outcome.failed == 0` on healthy execution.
     - Simulate a database error on one record:
       - Assert that `outcome.failed` increments to 1.
       - Assert that `outcome.last_error` captures the error message without swallowing.
       - Assert that `switch.rs` audit log records real counts instead of copying `backed_up_count`.
  4. **Unscoped Fallback Mechanism (RC-2 Fallback)**:
     - Create an isolated test backup DB where prompts exist for `inst-gamma` with `is_restored = 0`, but none exist for `inst-delta`.
     - Invoke `restore_running_prompts_scoped_with_fallback("inst-delta", true, custom_db)`.
     - Verify that the initial scoped query for `inst-delta` returns 0 records.
     - Verify that `count_unrestored_prompts_for_other_instances("inst-delta")` returns > 0.
     - Verify that the function logs the fallback and executes an unscoped restore, successfully restoring the orphaned prompts.
  5. **Live Status Preservation & Double-Dispatch Guard (RC-6)**:
     - Set a prompt in `active_prompts` with status `running`.
     - Attempt to restore a backup record with the same prompt ID.
     - Assert that `is_live_status_preserved` returns `true`, `outcome.skipped_live` increments by 1, and the live `running` prompt is not overwritten.
     - Set thread-local `RESEND_IN_PROGRESS = true`.
     - Attempt to call `restore_running_prompts_for_instance_with_outcome`.
     - Assert that the call fails immediately with the double-dispatch guard error.
- **Expected Outcomes**:
  - Long-running prompts are never dropped during auto-switch or quota exhaustion.
  - Zero orphaned prompts across manual or automatic profile switches.
  - 100% honesty in restore reporting without fake successes.
  - Strict FIFO order (`ORDER BY created_at ASC, sequence_id ASC`) preserved throughout.

---

### Test Case `E2E-215-03`: Rerun Prompt Randomizer, Enlarged Message Set & Storage Persistence
- **Target Module**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
  - `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`
  - `src/components/instances/PromptTreeView/PromptTreeContextMenu.tsx`
- **Improvement ID**: `Imp-702` (Step 288).
- **Invariant Addressed**: Minimalist & Contextual UI, Zero Silent Errors (`AGENTS.md`).
- **Preconditions**:
  - Browser DOM environment with mocked or real `window.localStorage`.
  - Prompt Tree View mounted with an active conversation selected.
- **Actions / Execution Steps**:
  1. **Default Option & Elevated Count Verification**:
     - Clear `window.localStorage` keys `agm_prompt_tree_rerun_message_option` and `agm_prompt_tree_rerun_count`.
     - Mount `DetailHeader` component.
     - Assert `loadPersistedRerunMessageOption()` returns `'randomized'` (`RERUN_DEFAULT_MESSAGE_OPTION`).
     - Assert `loadPersistedRerunCount()` returns `4` (`RERUN_DEFAULT_COUNT`).
     - Assert that the UI rerun capsule highlights the "Randomized nudge" picker option by default.
     - Assert that the count capsule displays `4` as the active pill button.
  2. **12+ Prompt Set Integrity & Random Sampling**:
     - Inspect `RERUN_RANDOM_MESSAGE_SET`.
     - Assert `RERUN_RANDOM_MESSAGE_SET.length >= 12` (verified at 14 items).
     - Validate that all prompt variations are non-empty, distinct, and represent polite, professional completion checks (e.g. "Quick status check — is it done yet?", "Please continue where you left off and confirm when finished.", "Are we there yet? Let me know the current state.").
     - Trigger 100 iterations of random message selection for `'randomized'`.
     - Assert that all 14 variations are reachable and distribution shows uniform randomization.
  3. **LocalStorage Persistence & Hydration Lifecycle**:
     - Select message option `'is-it-published'` in the UI.
     - Assert `window.localStorage.getItem('agm_prompt_tree_rerun_message_option') === 'is-it-published'`.
     - Select rerun count `5`.
     - Assert `window.localStorage.getItem('agm_prompt_tree_rerun_count') === '5'`.
     - Unmount `DetailHeader` and remount.
     - Assert that the component hydrates with `'is-it-published'` and count `5`.
     - Test localStorage failure scenario (mock `localStorage.setItem` throwing an exception):
       - Assert that `savePersistedRerunMessageOption` catches the error.
       - Assert that `useErrorStore.getState().trackWarning` is invoked with source `'usePromptTreePromptActions.savePersistedRerunMessageOption'`.
       - Assert that the in-memory state continues functioning without crashing the UI.
  4. **Right-Click Context Menu Integration Parity**:
     - Open `ConversationContextMenu` on a conversation node.
     - Inspect the "Rerun conversation" menu item label and tooltip.
     - Assert that the menu action reads `loadPersistedRerunMessageOption()` and dispatches using the persisted option and count.
     - Trigger rerun from context menu:
       - Verify that the generated prompt payload uses the persisted message mode.
       - Verify that attached images (both prior conversation images and newly staged media) are preserved.
- **Expected Outcomes**:
  - The UI defaults out-of-the-box to randomized follow-up nudges with count 4.
  - Zero state drift between DetailHeader toolbar actions and context menu actions.
  - Full error reporting into the error store if localStorage access fails.

---

### Test Case `E2E-215-04`: GitMap Code Quality & Pending Plans Drain
- **Target Module**:
  - GitMap CLI repository (`../gitmap/`)
  - Go CLI command handlers (`cmd/`, `internal/`)
  - Plan directories (`.ai-memory/plans/pending/`, `.ai-memory/plans/completed/`)
- **Improvement ID**: `Imp-703` (Step 289).
- **Invariant Addressed**: Code Quality, Boolean Improvements, AppError Migration (`AGENTS.md`).
- **Preconditions**:
  - GitMap repository checked out in parallel workspace directory.
  - Go toolchain configured.
- **Actions / Execution Steps**:
  1. **Pending Plans Drain Audit**:
     - Inspect `.ai-memory/plans/pending/` across GitMap.
     - Identify all plan files whose corresponding tasks, features, or fixes have been fully implemented.
     - Verify that completed plan files are cleanly moved to `.ai-memory/plans/completed/` with updated status headers (`Status: COMPLETED`).
     - Assert that no stale, completed plans linger in `pending/`.
  2. **Boolean Comparison Elimination (`== true` / `== false`)**:
     - Perform a static scan across all `.go` files in GitMap for occurrences of `== true` or `== false`.
     - Assert 0 matches found.
     - Validate that all conditions use direct boolean evaluation (`if condition` or `if !condition`).
  3. **Structured `AppError` Migration**:
     - Audit error returns across CLI command implementations (`cmd/`) and core packages.
     - Verify that raw `fmt.Errorf("...")` calls without codes or context have been migrated to structured `apperror.AppError` (or `apperror.New(...)` / `appfault.New(...)`).
     - Confirm that error messages maintain clear operational context, caller attribution, and proper wrapping.
  4. **Function and File Size Caps**:
     - Audit touched Go files against repository limits (functions targeted at 8–15 lines, decomposed modules).
     - Verify clean imports and absence of unused declarations or dead code.
- **Expected Outcomes**:
  - 100% clean pending plan backlog with verified completed archives.
  - Zero forbidden boolean equality patterns across the Go codebase.
  - Consistent, structured error propagation across all GitMap CLI commands.

---

### Test Case `E2E-215-05`: Sequence 215 Master Regression Gate & Zero Silent Errors Invariant
- **Target Module**:
  - Entire `Antigravity-Manager` workspace.
  - Rust cargo toolchain (`src-tauri`).
  - Vite / TypeScript build pipeline (`src`).
- **Improvement ID**: `Imp-704` (Step 290).
- **Invariant Addressed**: Formatting & CI Discipline, No Silent Errors Invariant (`AGENTS.md`).
- **Preconditions**:
  - Working tree in clean or staged state for Sequence 215 changes.
  - Node.js and Rust environments available.
- **Actions / Execution Steps**:
  1. **Rust Pre-flight Formatting Gate**:
     - Execute deferred command `cd src-tauri && cargo fmt -- --check`.
     - Assert that all Rust source files adhere strictly to rustfmt guidelines with 0 formatting violations.
  2. **Rust Clippy Gate**:
     - Execute deferred command `cd src-tauri && cargo clippy --all-targets --all-features`.
     - Assert 0 errors and 0 warnings.
     - Verify that all touched modules compile cleanly under strict linting rules.
  3. **TypeScript / Frontend Build Gate**:
     - Execute deferred command `npm run build`.
     - Verify that TypeScript type-checking (`tsc`) passes without errors.
     - Verify that Vite bundle compilation completes successfully with 0 module resolution failures.
  4. **Zero Silent Errors Invariant Audit**:
     - Perform an exhaustive static audit on all newly added or modified lines in Rust and TypeScript:
       - In Rust: confirm that every `Result` is either propagated via `?` or tracked via `crate::error::record_ignored(..., "<operation>")` with an explicit `// Justification:` comment. Verify NO bare `.ok()`, `let _ = <fallible>`, or unhandled `unwrap_or_default()`.
       - In TypeScript: confirm that every `try/catch` block routes errors to `useErrorStore.getState().trackWarning` or `captureError`. Verify NO empty `catch {}` or bare `catch (e) {}` without error store routing.
  5. **Absolute Path Leakage Audit**:
     - Scan all modified files for Windows drive letters (`D:\`, `C:\`) or unix home paths (`/home/`).
     - Confirm 0 occurrences in production code, comments, specs, or commit messages.
- **Expected Outcomes**:
  - 100% green pre-flight verification across Rust, TypeScript, and GitMap.
  - Complete compliance with the No Silent Errors house rule.
  - Flawless relative path hygiene across the entire repository.

---

## 5. Verification Matrix & Edge Cases Summary

| Test ID | Subsystem | Happy Path Scenario | Primary Failure / Edge Case Mode | Invariant / Rule Checked |
| :--- | :--- | :--- | :--- | :--- |
| `E2E-215-01` | GitMap Fleet & Audit | Instant status report across 68 repos | Diverged branches, dirty trees, unpushed commits | Strictly Relative Paths & Git Hygiene |
| `E2E-215-02` | AGM Backend Restore | Long-running prompts restored post auto-switch | >120s prompt clobber, orphaned backups, fake success counts | FIFO Restore & Honest Outcome Reporting |
| `E2E-215-03` | AGM Frontend Rerun | Defaults to randomized nudge (count 4) | Blocked localStorage, missing conversation images | Minimalist & Contextual UI, Error Visibility |
| `E2E-215-04` | GitMap Code Quality | All completed plans drained to `completed/` | Explicit `== true`, unstructured `fmt.Errorf` | Code Quality & AppError Architecture |
| `E2E-215-05` | Workspace Regression | `cargo fmt`, `clippy`, and `npm run build` pass | Swallowed `Result`, bare catch blocks, absolute path leak | Formatting & CI Discipline, No Silent Errors |

---

## 6. Exact Targeted Deferred Test Commands

When an explicit testing prompt is issued, execute the following commands in order:

```bash
# ==============================================================================
# 1. Step 286 (Imp-700): GitMap PA Fleet Audit & Git Hygiene
# ==============================================================================
# Run fleet discovery and status audit across all 68 repositories
gitmap pa

# Audit commit messages for strictly relative paths and conventional formatting
gitmap log -n 50


# ==============================================================================
# 2. Step 287 (Imp-701): Auto-Switch Prompt Restore & Quota Drop Recovery
# ==============================================================================
# Unit & integration tests for backup DB lifecycle, scoping, and restore outcomes
cd src-tauri && cargo test --package antigravity-tools-lib --lib modules::backup_prompts_db::tests

# End-to-end integration tests for auto-switcher, prompt snapshotting, and restoration
cd src-tauri && cargo test --test auto_switcher_e2e_test -- --ignored


# ==============================================================================
# 3. Step 288 (Imp-702): Rerun Prompt Randomizer & UI Persistence
# ==============================================================================
# Frontend test suite for prompt tree actions, randomized picker, and persistence
npm test -- src/components/instances/PromptTreeView/__tests__/treeSequence.test.ts


# ==============================================================================
# 4. Step 289 (Imp-703): GitMap Code Quality & Structured Errors
# ==============================================================================
# Verify Go CLI tests and AppError handling in GitMap
cd ../gitmap && go test ./internal/apperror/... ./cmd/...


# ==============================================================================
# 5. Step 290 (Imp-704): Master Pre-Flight Regression Gate
# ==============================================================================
# Rust code formatting check
cd src-tauri && cargo fmt -- --check

# Rust comprehensive clippy lint gate (includes compilation)
cd src-tauri && cargo clippy --all-targets --all-features

# Frontend TypeScript compilation & Vite production build
npm run build
```

---
*End of Sequence 215 Master Execution Plan.*
