# Application Specification: Sequence 215

## 1. Executive Summary
**Task Slug**: `gitmap-and-antigravity-tools-manager-updates-and-testing`  
**Sequence**: 215  
**Core Objective**:
Synchronize and audit recent updates across `Antigravity-Manager` and `GitMap`, drain pending plan queues, enforce coding guidelines, resolve prompt restore failures following auto-switching / quota exhaustion, implement randomized auto-pick for prompt reruns, and prepare exhaustive deferred end-to-end verification suites.

---

## 2. Problem Statement & Root Cause Synthesis

### A. Prompt Restore Failure on Account Auto-Switch / Quota Depletion
- **RC-1 (120s In-Flight Window)**: `repo_db/backup.rs` `backup_running_prompts` previously only transitioned `dispatched` rows to `backed_up` if dispatched within 120s. Long-running prompts fall out of the restore query window when quota finishes.
  - **Resolution**: Implement comprehensive incomplete-prompt sweep transitioning all `running`, `in_flight`, and `dispatched` rows not marked `completed` (regardless of age) to `backed_up` upon switch or quota expiration.
- **RC-2 (Orphaned Durable Backups on Manual/Auto Switch)**: Unrestored rows in `prompt_backups` keep source instance scoping while restore is scoped to target instance.
  - **Resolution**: Re-attribute unrestored `prompt_backups` rows to target instance in `switch.rs` and fallback to unscoped restore when scoped restore returns 0 records.
- **RC-3 (Reporting Parity & Honesty)**: `restored_prompts_count` was previously written from `backed_up_count` rather than actual dispatched/failed counts.
  - **Resolution**: Propagate true `RestoreOutcome { restored_count, dispatched_count, failed_count }` across audit logs, notifications, and UI toasts.

### B. Rerun Randomizer & Auto-Pick
- **Current Behavior**: Rerun dialog defaults to static `is-it-done` with count 2; context menu rerun bypasses randomized options.
- **Required Behavior**:
  - Default option = `randomized` ("Are you done?", "Is it released?", "Did you verify edge cases?").
  - Enlarge `RERUN_RANDOM_MESSAGE_SET` to at least 12 rich completion-check variations.
  - Persist user option selection and count across mounts and sessions.
  - Parity between DetailHeader button and ConversationContextMenu actions.

### C. GitMap Code Quality & CG Execute Prompts
- Enforce coding guidelines across GitMap:
  - Eliminate explicit `== true` / `== false`.
  - Migrate `fmt.Errorf` to structured `apperror.AppError`.
  - Enforce function size limits (under 100 lines) and file size limits.
  - Drain completed pending plans in `.ai-memory/plans/pending/`.

---

## 3. Work Breakdown & Step Mapping

| Step | Subtask ID | Focus Area | Target File(s) | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Step 286** | `Subtask 01` | Git & Audit | `02-spec/21-app/215-gitmap-and-antigravity-tools-manager-updates-and-testing/audit_report.md` | Run `gitmap pa`, analyze last 50-80 commits across AGM and GitMap, audit prompt/skill changes. |
| **Step 287** | `Subtask 02` | AGM Backend | `src-tauri/src/modules/backup_prompts_db/restore.rs`, `src-tauri/src/modules/instance/switch.rs` | Fix prompt restore on auto-switch and quota completion; implement incomplete-prompt sweep and honest outcome reporting. |
| **Step 288** | `Subtask 03` | AGM Frontend | `src/components/instances/PromptTreeView/DetailHeader.tsx`, `usePromptTreePromptActions.ts` | Implement rerun randomized auto-pick by default, enlarge prompt sets (12+ variations), and persist selections. |
| **Step 289** | `Subtask 04` | GitMap CG | `../gitmap/` | Drain completed pending plans in GitMap, verify coding guideline adherence, eliminate boolean comparisons, and migrate AppError. |
| **Step 290** | `Subtask 05` | E2E Testing | `02-spec/21-app/215-gitmap-and-antigravity-tools-manager-updates-and-testing/e2e_master_plan.md` | Author deferred E2E test catalog covering auto-switch prompt restore, quota drop recovery, rerun randomizer, and GitMap commands. |

---

## 4. Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly DO NOT run builds or tests during implementation. All live test runs remain deferred to the explicit testing phase.
- **Exclusively GitMap for Search**: Strictly use `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Git Paths**: 100% repository-relative Git paths across all specs, code, logs, and commit messages.
- **Strict FIFO Ordering**: Queue restores and queries must strictly enforce `ORDER BY created_at ASC, id ASC`.
- **No Silent Errors Rule**: Every fallible path routes to `AppError` or uses `record_ignored` with an explicit `// Justification:` comment.
