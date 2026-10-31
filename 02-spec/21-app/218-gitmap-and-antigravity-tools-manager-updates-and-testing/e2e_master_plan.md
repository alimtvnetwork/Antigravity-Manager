# End-to-End Master Execution Plan: Sequence 218 (Steps 301–305)

## 1. Scope & Batch Objectives
This document establishes the comprehensive end-to-end and integration verification plan for the 5 architectural, backend restoration, frontend rerun, CLI code quality, and regression gate modules implemented in Sequence 218 (`Imp-715` through `Imp-719`).

In accordance with the **Zero-Build & Zero-Test Invariant** codified in `AGENTS.md` and repository standards, live test runs are strictly deferred during the implementation phase. Any AI agent or developer executing deferred verification must follow the detailed specifications and exact command lines cataloged in this master plan.

The five focus areas verified in this sequence are:
1. **GitMap PA Fleet Audit & Recent Commits Invariant** (`Imp-715`, Step 301): Complete discovery and remote branch reconciliation across all 68 repositories in the developer workspace, verification of recent commit history invariants, and relative path hygiene.
2. **Auto-Switch Prompt Restore & Quota Drop Recovery** (`Imp-716`, Step 302): Elimination of arbitrary dispatch window clobbers, incomplete-prompt sweep across all uncompleted statuses (`running`, `in_flight`, `dispatched`, `pending`, `queued`, `blocked`), durable prompt re-attribution on manual/auto switch, honest `RestoreOutcome` reporting, and unscoped fallback.
3. **Rerun Prompt Randomizer, Enlarged Message Set & Storage Persistence** (`Imp-717`, Step 303): Default selection of `'randomized'`, elevated default count of 4, enlarged 16-variation prompt set, dual `localStorage` persistence keys with error tracking, and full context-menu integration parity.
4. **GitMap Code Quality & Pending Plans Drain** (`Imp-718`, Step 304): Drainage of completed pending plans into `completed/`, complete elimination of explicit `== true` and `== false` boolean comparisons, and migration of Go CLI error handling to structured `AppError`.
5. **Sequence 218 Master Regression Gate & Zero Silent Errors Invariant** (`Imp-719`, Step 305): Full pre-flight gate verification (`cargo fmt`, `cargo clippy`, `npm run build`), strict relative path enforcement, and zero silent errors audit.

---

## 2. Architectural Invariants Enforced
All test cases strictly enforce the non-negotiable architectural invariants codified in `AGENTS.md`:

- **Architecture & Pipeline First**: The system maintains strict separation of concerns. Adapters perform protocol normalization, while the pipeline handles payload sanitization, context alignment, and thinking backfilling.
- **FIFO Restore & Queue Invariant**: All prompt backup, restore, and dispatch queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly prohibited in dispatch and recovery logic.
- **No Silent Errors Invariant**: Every fallible operation routes into the error module. Bare `.ok()`, swallowed errors, and bare `catch {}` blocks are strictly forbidden. Best-effort operations require `crate::error::record_ignored` along with a mandatory `// Justification:` comment in Rust, and `useErrorStore.getState().trackWarning` in TypeScript.
- **Strictly Relative Git Paths Invariant**: Absolute filesystem paths must never appear in specifications, plans, commit messages, changelogs, READMEs, or release notes. All file references must be repository-relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **Honest Outcome Reporting (No Fake Success)**: Operations must never report success when errors occur. Restore outcomes propagate actual counts (`restored`, `dispatched`, `failed`, `skipped_live`) rather than claiming pre-restore counts or treating failures as success.
- **Contextual UI & Capsule Styling**: Rerun controls, toolbar actions, and buttons follow segmented pill capsule styling (`rounded-full`, dark-glass aesthetics, shared borders, subtle dividers) with transparent inline select dropdowns.

---

## 3. Step-by-Step Task Mapping & Improvement Ledger

| Step | Improvement ID | Category / Subsystem | Primary Target Files | Master Verification Test Case |
| :--- | :--- | :--- | :--- | :--- |
| **Step 301** | `Imp-715` | GitMap Fleet Audit & Commit Hygiene | `02-spec/21-app/218-gitmap-and-antigravity-tools-manager-updates-and-testing/audit_report.md` | `E2E-218-01` |
| **Step 302** | `Imp-716` | AGM Backend (Restore & Switch) | `src-tauri/src/modules/backup_prompts_db/restore.rs`<br>`src-tauri/src/modules/repo_db/backup.rs`<br>`src-tauri/src/modules/auto_switcher/rotation.rs`<br>`src-tauri/src/modules/instance/switch.rs` | `E2E-218-02` |
| **Step 303** | `Imp-717` | AGM Frontend (Prompt Tree & Rerun) | `src/components/instances/PromptTreeView/DetailHeader.tsx`<br>`src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`<br>`src/components/instances/PromptTreeView/ConversationContextMenu.tsx`<br>`src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | `E2E-218-03` |
| **Step 304** | `Imp-718` | GitMap Code Quality & Plan Management | `.ai-memory/plans/`<br>`cmd/`<br>`internal/apperror/` | `E2E-218-04` |
| **Step 305** | `Imp-719` | Master Deferred E2E Plan & Gate | `02-spec/21-app/218-gitmap-and-antigravity-tools-manager-updates-and-testing/e2e_master_plan.md`<br>Workspace Linters & Pre-flight Gates | `E2E-218-05` |

---

## 4. Comprehensive End-to-End Test Scenarios

### Test Case `E2E-218-01`: GitMap PA Fleet Audit & Recent Commits Invariant
- **Target Module**: GitMap CLI Project Auditor (`gitmap pa`), Repository Scanner, and Git Invariant Validator.
- **Improvement ID**: `Imp-715` (Step 301).
- **Invariant Addressed**: Strictly Relative Git Paths & Commit History Hygiene (`AGENTS.md`).
- **Preconditions**:
  - The developer workspace contains all 68 active and managed repositories.
  - Remote tracking branches (`origin/main`, `origin/beta`) configured for GitMap and Antigravity-Manager.
- **Actions / Execution Steps**:
  1. **Fleet Discovery & Remote Reconciliation**:
     - Execute deferred command `gitmap pa`.
     - Verify that all 68 repositories are discovered without timeout or process stalls within 30 seconds.
     - Validate output table/JSON categorization into `clean`, `ahead`, `behind`, and `diverged` statuses.
     - Confirm that 66 repositories are cleanly up-to-date and 2 active repositories (`coding-guidelines-v24` and `repo-cache`) are accurately tracked.
  2. **Recent Commit History Inspection**:
     - Audit the last 50–80 git commits across `Antigravity-Manager` and `gitmap`.
     - Verify that each commit follows conventional atomic formatting with valid imperative commit messages (`fix: ...`, `feat: ...`, `refactor: ...`).
     - Verify strict attribution to `@aukgit` (`(Thanks to @aukgit)`) with zero unauthorized user handles.
     - Ensure no intermediate debugging churn, temporary broken states, or trial-and-error commits exist.
  3. **Strictly Relative Path Hygiene Check**:
     - Inspect git commit messages, branch references, and generated markdown logs from the commit window.
     - Validate that zero absolute filesystem paths appear in commit messages, specifications, or audit documentation.
     - Confirm that all repository references use forward-slash, repository-relative notation (`02-spec/...`, `src/...`, `src-tauri/...`).
  4. **Coding Guidelines Ingestion Verification**:
     - Verify recent coding guidelines updates: installer script signal handling fixes and Python boolean naming conventions.
     - Confirm ingestion into architectural documentation in `02-spec/21-app/218-gitmap-and-antigravity-tools-manager-updates-and-testing/audit_report.md`.
- **Expected Outcomes**:
  - 100% discovery rate across all 68 repositories in the workspace.
  - Zero absolute filesystem paths detected across all audited commits and logs.
  - Verified atomic commit hygiene across both AGM and GitMap repositories.

---

### Test Case `E2E-218-02`: Auto-Switch Prompt Restore & Quota Drop Recovery
- **Target Module**:
  - `src-tauri/src/modules/backup_prompts_db/restore.rs`
  - `src-tauri/src/modules/repo_db/backup.rs`
  - `src-tauri/src/modules/auto_switcher/rotation.rs`
  - `src-tauri/src/modules/instance/switch.rs`
  - `src-tauri/src/modules/backup_prompts_db/tests_prompt_restore_honesty.rs`
- **Improvement ID**: `Imp-716` (Step 302).
- **Invariant Addressed**: FIFO Restore & Queue Invariant, No Silent Errors, Honest Outcome Reporting (`AGENTS.md`).
- **Preconditions**:
  - Multi-instance SQLite database setup with test instances `inst-alpha` and `inst-beta`.
  - SQLite tables `active_prompts` and `prompt_backups` seeded with prompts in varied lifecycle states (`running`, `in_flight`, `dispatched`, `pending`, `queued`, `blocked`, `completed`).
- **Actions / Execution Steps**:
  1. **Incomplete-Prompt Sweep & Elimination of 120s Window Clobber (RC-1)**:
     - Insert a prompt into `active_prompts` with status `dispatched` and `updated_at = now - 7200` (dispatched 2 hours ago, simulating a long-running prompt during quota depletion).
     - Insert prompts with statuses `running`, `in_flight`, `pending`, and `blocked` under `inst-alpha`.
     - Trigger `sweep_incomplete_prompts_for_instance("inst-alpha")`.
     - Query `active_prompts` for `inst-alpha`:
       - Assert that the 2-hour-old `dispatched` prompt transitioned to `backed_up` (proving removal of the arbitrary 120-second recency clobber).
       - Assert that `running`, `in_flight`, `pending`, and `blocked` prompts transitioned to `backed_up`.
       - Assert that prompts with status `completed` remained untouched.
  2. **Durable Prompt Re-attribution on Switch (RC-2)**:
     - Insert unrestored rows into `prompt_backups` scoped to `instance_id = 'inst-alpha'` with `is_restored = 0`.
     - Execute instance switch from `inst-alpha` to `inst-beta` via `switch_account_to_instance` or `execute_profile_rotation_with_context`.
     - Verify that `switch.rs` and `rotation.rs` execute the re-attribution update:
       `UPDATE prompt_backups SET instance_id = 'inst-beta' WHERE instance_id = 'inst-alpha' AND is_restored = 0`.
     - Assert that unrestored records in `prompt_backups` now belong to `inst-beta`.
     - Assert that already-restored records (`is_restored = 1`) remain unchanged under `inst-alpha`.
  3. **Honest `RestoreOutcome` Struct Verification (RC-3)**:
     - Invoke `restore_running_prompts_for_instance_with_outcome(Some("inst-beta"), false, custom_db)`.
     - Validate that the returned tuple `(records, outcome)` contains accurate counts:
       - `outcome.restored` matches the exact number of prompts re-injected into `active_prompts`.
       - `outcome.dispatched` matches the number of prompts actually sent by `resend_running_commands_for_instance`.
       - `outcome.failed == 0` on healthy execution.
       - `outcome.is_success() == true`.
     - Simulate a database or dispatch error on one record:
       - Assert that `outcome.failed` increments to 1.
       - Assert that `outcome.last_error` captures the error message without swallowing.
       - Assert that `outcome.is_success() == false`.
       - Assert that callers record real counts instead of claiming fake successes.
  4. **Unscoped Fallback Mechanism (RC-2 Fallback)**:
     - Create an isolated test backup DB where prompts exist for `inst-gamma` with `is_restored = 0`, but none exist for `inst-delta`.
     - Invoke `restore_running_prompts_scoped_with_fallback("inst-delta", true, custom_db)`.
     - Verify that the initial scoped query for `inst-delta` returns 0 records.
     - Verify that `count_unrestored_prompts_for_other_instances("inst-delta", custom_db)` returns > 0.
     - Verify that the function logs the fallback and executes an unscoped restore, successfully restoring the orphaned prompts under their respective instances in FIFO order.
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

### Test Case `E2E-218-03`: Rerun Prompt Randomizer, Enlarged Message Set & Storage Persistence
- **Target Module**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
  - `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`
  - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
  - `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`
- **Improvement ID**: `Imp-717` (Step 303).
- **Invariant Addressed**: Minimalist & Contextual UI, Zero Silent Errors (`AGENTS.md`).
- **Preconditions**:
  - Browser DOM environment with mocked or real `window.localStorage`.
  - Prompt Tree View mounted with an active conversation selected.
- **Actions / Execution Steps**:
  1. **Default Option & Elevated Count Verification**:
     - Clear `window.localStorage` keys `agm:prompt-tree:rerun-option` and `agm:prompt-tree:rerun-count`.
     - Mount `DetailHeader` component.
     - Assert `loadPersistedRerunMessageOption()` returns `'randomized'` (`RERUN_DEFAULT_MESSAGE_OPTION`).
     - Assert `loadPersistedRerunCount()` returns `4` (`RERUN_DEFAULT_COUNT`).
     - Assert that the UI rerun capsule highlights the "Randomized nudge" picker option by default.
     - Assert that the count capsule displays `4` as the active pill button.
  2. **16+ Prompt Set Integrity & Random Sampling**:
     - Inspect `RERUN_RANDOM_MESSAGE_SET`.
     - Assert `RERUN_RANDOM_MESSAGE_SET.length >= 16`.
     - Validate that all 16 prompt variations are non-empty, distinct, and represent polite, professional completion checks:
       1. "Is it completed?"
       2. "Is it finished?"
       3. "Is it released?"
       4. "Has this been fully implemented and verified?"
       5. "Can you check the pending tasks and continue?"
       6. "Is the implementation done and pushed?"
       7. "Please verify and finish the remaining items."
       8. "Did all tests pass and is everything green?"
       9. "Are all requirements satisfied?"
       10. "Please double check completion and finalize."
       11. "Is there any pending work left?"
       12. "Please confirm completion status and next steps."
       13. "Quick status check — is it done yet?"
       14. "Please continue where you left off and confirm when finished."
       15. "Checking in: how is it going? Any blockers or pending items?"
       16. "Status update please: what is done and what is left to complete?"
     - Trigger 100 iterations of random message selection for `'randomized'`.
     - Assert that all 16 variations are reachable and distribution shows uniform randomization.
  3. **LocalStorage Persistence & Hydration Lifecycle**:
     - Select message option `'is-it-published'` in the UI.
     - Assert `window.localStorage.getItem('agm:prompt-tree:rerun-option') === 'is-it-published'`.
     - Assert legacy fallback key `agm_prompt_tree_rerun_message_option` is also updated.
     - Select rerun count `6`.
     - Assert `window.localStorage.getItem('agm:prompt-tree:rerun-count') === '6'`.
     - Unmount `DetailHeader` and remount.
     - Assert that the component hydrates with `'is-it-published'` and count `6`.
     - Test clamping:
       - Set count `0` -> asserts clamped to `1` (`RERUN_MIN_COUNT`).
       - Set count `999` -> asserts clamped to `8` (`RERUN_MAX_COUNT`).
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
  - Zero state drift between DetailHeader toolbar actions, modal strips, and context menu actions.
  - Full error reporting into the error store if localStorage access fails.

---

### Test Case `E2E-218-04`: GitMap Code Quality & Pending Plans Drain
- **Target Module**:
  - GitMap CLI repository (`../gitmap/`)
  - Go CLI command handlers (`cmd/`, `internal/`)
  - Plan directories (`.ai-memory/plans/pending/`, `.ai-memory/plans/completed/`)
- **Improvement ID**: `Imp-718` (Step 304).
- **Invariant Addressed**: Code Quality, Boolean Improvements, AppError Migration (`AGENTS.md`).
- **Preconditions**:
  - GitMap repository checked out in parallel workspace directory.
  - Go toolchain configured.
- **Actions / Execution Steps**:
  1. **Pending Plans Drain Audit**:
     - Inspect `.ai-memory/plans/pending/` across GitMap.
     - Identify all plan files whose corresponding tasks, features, or fixes have been fully implemented.
     - Verify that completed plan files are cleanly moved to `.ai-memory/plans/completed/` with updated status headers (`Status: COMPLETED`).
     - Assert that zero stale completed plans linger in `pending/`.
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

### Test Case `E2E-218-05`: Sequence 218 Master Regression Gate & Zero Silent Errors Invariant
- **Target Module**:
  - Entire `Antigravity-Manager` workspace.
  - Rust cargo toolchain (`src-tauri`).
  - Vite / TypeScript build pipeline (`src`).
- **Improvement ID**: `Imp-719` (Step 305).
- **Invariant Addressed**: Formatting & CI Discipline, No Silent Errors Invariant (`AGENTS.md`).
- **Preconditions**:
  - Working tree in clean or staged state for Sequence 218 changes.
  - Node.js, Go, and Rust environments available.
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
  4. **Frontend Unit Tests Gate**:
     - Execute deferred command `npm test`.
     - Verify that all test suites pass with 0 failures (including `rerunActions.test.ts` and `treeSequence.test.ts`).
  5. **Zero Silent Errors Invariant Audit**:
     - Perform an exhaustive static audit on all newly added or modified lines in Rust and TypeScript:
       - In Rust: confirm that every `Result` is either propagated via `?` or tracked via `crate::error::record_ignored(..., "<operation>")` with an explicit `// Justification:` comment. Verify NO bare `.ok()`, `let _ = <fallible>`, or unhandled `unwrap_or_default()`.
       - In TypeScript: confirm that every `try/catch` block routes errors to `useErrorStore.getState().trackWarning` or `captureError`. Verify NO empty `catch {}` or bare `catch (e) {}` without error store routing.
  6. **Absolute Path Leakage Audit**:
     - Scan all modified files for Windows drive letters or unix home paths.
     - Confirm 0 occurrences in production code, comments, specs, or commit messages.
- **Expected Outcomes**:
  - 100% green pre-flight verification across Rust, TypeScript, and GitMap.
  - Complete compliance with the No Silent Errors house rule.
  - Flawless relative path hygiene across the entire repository.

---

## 5. Verification Matrix & Edge Cases Summary

| Test ID | Subsystem | Happy Path Scenario | Primary Failure / Edge Case Mode | Invariant / Rule Checked |
| :--- | :--- | :--- | :--- | :--- |
| `E2E-218-01` | GitMap Fleet & Audit | Instant status report across 68 repos | Diverged branches, dirty trees, unpushed commits | Strictly Relative Paths & Git Hygiene |
| `E2E-218-02` | AGM Backend Restore | Long-running prompts restored post auto-switch | >120s prompt clobber, orphaned backups, fake success counts | FIFO Restore & Honest Outcome Reporting |
| `E2E-218-03` | AGM Frontend Rerun | Defaults to randomized nudge (count 4, 16 options) | Blocked localStorage, missing conversation images, out-of-range counts | Minimalist & Contextual UI, Error Visibility |
| `E2E-218-04` | GitMap Code Quality | All completed plans drained to `completed/` | Explicit `== true`, unstructured `fmt.Errorf` | Code Quality & AppError Architecture |
| `E2E-218-05` | Workspace Regression | `cargo fmt`, `clippy`, and `npm run build` pass | Swallowed `Result`, bare catch blocks, absolute path leak | Formatting & CI Discipline, No Silent Errors |

---

## 6. Exact Targeted Deferred Test Commands

When an explicit testing prompt is issued at the completion of all code changes, execute the following commands in order:

```bash
# ==============================================================================
# 1. Step 301 (Imp-715): GitMap PA Fleet Audit & Git Hygiene
# ==============================================================================
# Run fleet discovery and status audit across all 68 repositories
gitmap pa

# Audit commit messages for strictly relative paths and conventional formatting
gitmap log -n 50


# ==============================================================================
# 2. Step 302 (Imp-716): Auto-Switch Prompt Restore & Quota Drop Recovery
# ==============================================================================
# Unit tests for prompt restore honesty, scoping, and re-attribution
cd src-tauri && cargo test --package antigravity-tools-lib --lib modules::backup_prompts_db::tests_prompt_restore_honesty

# Unit tests for backup DB lifecycle and general storage info
cd src-tauri && cargo test --package antigravity-tools-lib --lib modules::backup_prompts_db::tests

# End-to-end integration tests for auto-switcher, prompt snapshotting, and restoration
cd src-tauri && cargo test --test auto_switcher_e2e_test -- --ignored


# ==============================================================================
# 3. Step 303 (Imp-717): Rerun Prompt Randomizer & UI Persistence
# ==============================================================================
# Frontend test suite for rerun actions, randomized picker, and persistence
node scripts/run-frontend-tests.mjs rerunActions
npm test -- src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts
npm test -- src/components/instances/PromptTreeView/__tests__/treeSequence.test.ts


# ==============================================================================
# 4. Step 304 (Imp-718): GitMap Code Quality & Structured Errors
# ==============================================================================
# Verify Go CLI tests and AppError handling in GitMap
cd ../gitmap && go test ./...
cd ../gitmap && go test ./internal/apperror/... ./cmd/...


# ==============================================================================
# 5. Step 305 (Imp-719): Master Pre-Flight Regression Gate
# ==============================================================================
# Rust code formatting check
cd src-tauri && cargo fmt -- --check

# Rust comprehensive clippy lint gate (includes compilation)
cd src-tauri && cargo clippy --all-targets --all-features

# Frontend TypeScript compilation & Vite production build
npm run build

# Master frontend test runner
npm test
```

---
*End of Sequence 218 Master Execution Plan.*
