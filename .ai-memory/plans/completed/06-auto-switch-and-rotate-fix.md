# Completed Plan: Auto Switch, Rotate, Fast Forward & CLI Revert Fix

## Metadata
- **Slug**: `auto-switch-and-rotate-fix`
- **Parent Plan**: `.ai-memory/plans/auto-switch-and-rotate-fix.md`
- **Canonical Spec**: `02-spec/21-app/auto-switch-and-rotate-fix/01-architecture-spec.md`, `02-spec/21-app/auto-switch-and-rotate-fix/02-component-spec.md`, `02-spec/21-app/auto-switch-and-rotate-fix/03-ai-instructions.md`
- **Execution Run**: `.ai-memory/temp-agents/06-auto-switch-and-rotate-fix/`
- **Status**: `COMPLETED`
- **Result**: `ALL 4 SUBTASKS VERIFIED & PASSED`

---

## 1. Executive Summary & Root Cause Synthesis

### Root Cause 1: Severe Cross-Instance PID Contamination
- **Symptom**: Account switching failed with error: `"Cannot switch instance to account '...': Account is actively bound to running sibling instance '...' on this machine"`. Auto-rotate found 0 candidate profiles. Fast-forward stalled for 10 seconds before failing.
- **Root Cause**: In `src-tauri/src/modules/instance/pid_scan.rs:302`, `instance_root_pids.contains(&p)` checked a global set containing root PIDs of ALL sandbox instances running on the OS. As a result, child processes of any running instance (e.g., renderers, language servers) matched this check and were falsely attributed to all stopped instances. `is_instance_running` returned `true` for every instance on the machine whenever any single instance was active.
- **Fix**: Removed `|| instance_root_pids.contains(&p)` in `src-tauri/src/modules/instance/pid_scan.rs`. Made `matched_set` mutable so descendants strictly match parents belonging to the specific target instance.

### Root Cause 2: 100% Quota Gate & Broken Fallback Candidate Scoring
- **Symptom**: Auto-rotate and Fast-forward abort with: `"No alternative healthy profile found in pool"`.
- **Root Cause**:
  1. In `src-tauri/src/modules/auto_switcher/scoring.rs:205`, `score_candidate_account` returned `0.0` for any account with `< 100%` quota.
  2. In `src-tauri/src/modules/auto_switcher/selection.rs:251-286`, Step 3 fallback gating was blocked when `cooldown_pool` was not empty, and called `score_candidate_account` (returning 0.0) instead of `score_candidate_account_fallback`.
  3. In `src-tauri/src/modules/auto_switcher/selection.rs:389`, the function exited prematurely when the 100% candidate list was empty instead of evaluating fallback accounts above the minimum threshold.
- **Fix**: Unblocked fallback gating, invoked `score_candidate_account_fallback` in Step 3, allowed fallback accounts to be evaluated in `select_and_verify_next_best_profile`, and added comprehensive unit tests.

### Root Cause 3: CLI Update Downgrade / Revert Failure
- **Symptom**: Running `agm update to v4.162` or `agm update 4.162` printed `[UP TO DATE]` and did nothing, or upgraded to latest rather than reverting to `v4.162`.
- **Root Cause**:
  1. `src-tauri/src/bin/agm/update_cmds.rs` never parsed positional versions, `to <version>`, or `--version`.
  2. The update command unconditionally queried `https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest`.
  3. When `current_version == latest`, it short-circuited with `[UP TO DATE]`.
  4. Tags on GitHub Releases use 3-part SemVer (`v4.162.0`). User queries for `4.162` or `v4.162` resulted in 404 unless normalized to `4.162.0`.
  5. `delegate_updater/run.rs` failed to pass `--version` on Unix, and `install.ps1:1455` blocked downgrades via `TargetVersion -le curr` unless `$isPinned` was honored.
- **Fix**: Added target version extraction supporting `to <ver>`, `revert <ver>`, `downgrade <ver>`, `<ver>`, and `--version <ver>` in `update_cmds.rs`. Added SemVer normalization (`4.162` -> `4.162.0`). Routed queries to `releases/tags/v{target}`. Forwarded `--version` through `delegate_updater`, and bypassed downgrade suppression in `install.ps1` for pinned versions.

### Root Cause 4: CI/CD Pipeline Compilation Errors
- **Symptom**: CI pipeline failed on Ubuntu, macOS, and Windows runners with `E0308` (mismatched types) and `E0433` (undeclared type `TokenData`).
- **Fix**:
  1. In `src-tauri/src/modules/process/running.rs:14,21`, passed `&default_dir.to_string_lossy()` to `find_pids_for_data_dir`.
  2. In `src-tauri/src/modules/auto_switcher/selection.rs:531`, added `use crate::models::TokenData;` inside `mod tests`.

---

## 2. Subtask Execution Verification

| Subtask ID | Title | Owner | Result |
| :--- | :--- | :--- | :--- |
| **Task-01** | CI/CD Compiler Fixes & Type Alignment | Worker 01 & 02 | `COMPLETED (PASS)` |
| **Task-02** | Cross-Instance PID Isolation Fix | Worker 01 | `COMPLETED (PASS)` |
| **Task-03** | Auto-Switcher Scoring Fallback & Rotation Gate | Worker 02 | `COMPLETED (PASS)` |
| **Task-04** | CLI Update Downgrade & Revert Support | Worker 02 | `COMPLETED (PASS)` |

---

## 3. Modified Files
- `src-tauri/src/modules/instance/pid_scan.rs`
- `src-tauri/src/modules/process/running.rs`
- `src-tauri/src/modules/auto_switcher/selection.rs`
- `src-tauri/src/modules/auto_switcher/scoring.rs`
- `src-tauri/src/bin/agm/update_cmds.rs`
- `src-tauri/src/modules/delegate_updater/args.rs`
- `src-tauri/src/modules/delegate_updater/run.rs`
- `install.ps1`
