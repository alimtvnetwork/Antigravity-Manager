# Fleet Audit & Commitment Ingestion Report (Sequence 216)
Date: `2026-10-23`
Slug: `216-gitmap-and-antigravity-tools-manager-updates-and-testing`

---

## 1. Executive Summary
This report documents the fleet-wide status following `gitmap pa`, analyzes recent commits across **Antigravity-Manager** and **GitMap**, presents a 5-point root-cause analysis (RCA) for the long-standing prompt loss and restore defect during quota-exhaustion auto-switches, and catalogs code quality improvements across both codebases.

---

## 2. Fleet Synchronization (`gitmap pa`)
- **Fleet Scope**: 68 managed repositories in workspace.
- **Execution Command**: `gitmap pa` (`gitmap pull-all`).
- **Outcome**: All core developer repositories (`Antigravity-Manager`, `gitmap`, `core-v9`, `enum-v10`, `errorwrapper-v4`, etc.) are up to date with remote tracking branches.
- **Working Tree Cleanliness**: Both target repositories (`Antigravity-Manager` and `gitmap`) maintain clean working trees with zero uncommitted or untracked drifts.

---

## 3. Commit History Review (Last 50–80 Commits)

### 3.1 Antigravity-Manager
- **Recent Milestones**:
  - `b0dbb38c`: Sequence 215 completion (quota exhaustion recovery, Step 0 sweep, Step 3 scheduling guarantee).
  - `2b26268a`: Full rustc and rustfmt alignment across 434 files.
  - `17e510b3`: Sequence 214 implementation (streaming payload accumulator, capsule button styler, FIFO prompt restore cursor stepper, batch execution interceptor, contributor authorship guard).
  - `c592b25c` – `398f421b`: Sequences 210–214 alignment and master test catalog expansions.
- **Coding Guideline Conformance**:
  - Strict FIFO order queries (`ORDER BY created_at ASC, id ASC`) enforced across `active_prompts`, `prompt_backups`, and `running_projects`.
  - Zero silent errors: all fallible paths propagate via `AppError` or record justifications.
  - Caller IDE self-preservation: verified protected PIDs and folder paths before any instance operation.

### 3.2 GitMap
- **Recent Milestones**:
  - `67154f1d4`: Completion of Plan 310 (`310-relative-paths.md`).
  - Strict relative Git path enforcement across all commands and documentation.
  - 0 pending plans in `.ai-memory/plans/pending/`.
- **Coding Guideline Conformance**:
  - Elimination of explicit boolean comparisons (`== true` / `== false`).
  - Structured error handling with `*appfault.AppError`.

---

## 4. Root Cause Analysis (RCA): Prompt Loss on Auto-Switch & Quota Finish

### Problem Statement
When an account exhausted its quota, active and queued prompts were frequently closed, dropped, or marked dispatched without ever re-executing on the target account.

### The 5 Compounding Root Causes

#### 1. Incomplete Prompt Query Scope
- **File**: `src-tauri/src/modules/repo_db/backup.rs`
- **Defect**: `backup_running_prompts` and `select_incomplete_prompt_ids` only queried `status IN ('running', 'in_flight', 'dispatched')`. Prompts with status `'pending'`, `'queued'`, or `'blocked'` were never transitioned to `'backed_up'`, so when the IDE closed, they were permanently abandoned.
- **Fix**: Expanded queries across all modules to: `status IN ('running', 'in_flight', 'dispatched', 'pending', 'queued', 'blocked')`.

#### 2. Stale Session False-Positive Live Guard
- **File**: `src-tauri/src/modules/backup_prompts_db/restore.rs`
- **Defect**: On IDE shutdown, prompts remaining in SQLite had `status = 'running'`. When the new account opened, `is_live_status_preserved` checked `matches!(status, Some("running") | Some("completed"))`. It assumed `'running'` meant actively executing in the new IDE session and skipped re-injection (`outcome.skipped_live += 1`).
- **Fix**: Constrained `is_live_status_preserved` strictly to `Some("completed")`. Stale uncompleted prompts from killed sessions are now always re-injected.

#### 3. Permanent In-Memory Cache Lockout in `resend.rs`
- **File**: `src-tauri/src/modules/repo_db/resend.rs`
- **Defect**: In `resend_single_pass`, the query fetched `status IN ('backed_up', 'queued', 'pending', 'running')`. However, the deduplication check was:
  ```rust
  let already_dispatched = if prompt.status == "backed_up" { false } else {
      let mut cache = get_dispatched_prompts_cache().lock().unwrap();
      cache.contains(&prompt.id) || cache.contains(&sig)
  };
  ```
  For prompts with status `'running'`, `'pending'`, or `'queued'`, this checked `cache`. Because the prompt was previously dispatched before the switch, its signature was already present in `cache`! As a result, `already_dispatched` was evaluated to `true`, the prompt was logged as `"Skipping duplicate dispatch"`, its DB status was updated to `'dispatched'` without sending to `agy`, and it was skipped with `continue;`!
- **Fix**: Changed `resend_running_commands_for_instance` to invoke `reset_dispatched_prompts_cache()`, and exempted all incomplete statuses (`"backed_up" | "running" | "pending" | "queued"`) from cache lockout.

#### 4. Auto-Switch Target Restore Scheduling
- **File**: `src-tauri/src/modules/auto_switcher/rotation.rs`
- **Defect**: Cross-instance and same-instance rotations did not consistently schedule `schedule_prompt_restore_after_launch(target_inst_id)`.
- **Fix**: Explicitly invoked Step 0 `sweep_incomplete_prompts_for_instance` and Step 3 `schedule_prompt_restore_after_launch(target_inst_id)` in `rotation.rs`.

#### 5. Hardcoded Rerun Message Option in Header Strip
- **File**: `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- **Defect**: `handleStripRerun` in the "Now Running + Up Next" strip hardcoded `'is-it-done'` instead of honoring the user's default/persisted preference (`'randomized'`).
- **Fix**: Updated `handleStripRerun` to pass `loadPersistedRerunMessageOption()`.

---

## 5. Coding Guidelines & CG Execute Verification
- Verified all 39 prompt modules in `01-prompts/15-cg-execute/`.
- Verified strictly relative Git paths across all documentation, commits, and plans.
- Verified bounded line limits and positive boolean names.
