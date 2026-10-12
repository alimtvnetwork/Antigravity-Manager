# Actionable Subtask Plan: Auto-Switch, Auto-Rotate, Fast-Forward, and Instance Restarting Remediation

> **Subtask Target:** `.ai-memory/plans/subtasks/auto-switch-and-rotate-fix/01-auto-switch-and-fast-forward-remediation.md`  
> **Parent Plan:** `.ai-memory/plans/auto-switch-and-rotate-fix.md`  
> **Associated Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/01-auto-switch-and-fast-forward-spec.md`  
> **Status:** READY_FOR_EXECUTION  
> **Author:** @aukgit  
> **Initiative Slug:** `auto-switch-and-rotate-fix`  
> **Execution Gate:** Rule 1 Zero Local Full Builds / Tests. Strict Relative Git Paths. No Raw Git Commands.

---

## 1. Subtask Objective & Context

This subtask defines the discrete, surgical remediation steps to restore full operational stability to account switching, profile rotation, Fast-Forward shortcuts, and deterministic instance restarts.

Between the baseline release `v4.162.0` and current development branches, four critical defects degraded these subsystems:
1. **Defect A1**: Process table scanning in `src-tauri/src/modules/instance/pid_scan.rs` contaminated child process attribution across instances, causing offline sibling instances to be falsely marked as running, which blocked account switching in `switch.rs:59`.
2. **Defect A2**: Candidate profile scoring in `src-tauri/src/modules/auto_switcher/selection.rs` invoked primary scoring instead of fallback scoring in Step 3, returning `0.0` for any candidate account with < 100% 4-hour quota and freezing rotation.
3. **Defect A4 & Compilation**: In `src-tauri/src/modules/instance/close.rs`, calling `super::pid_scan::force_refresh_process_cache()` broke compilation because `force_refresh_process_cache` resides in `pid_util.rs` (re-exported via `super`).
4. **Lifecycle Parity**: Ensuring the execution call chain from Fast-Forward (`trigger_manual_rotation_for_instance`) through to unified switching (`switch_account_to_instance`), and verified restart (`close_instance_verified` -> `launch_instance`).

---

## 2. Affected Files Matrix

| Target File | Component / Subsystem | Modification Type | Description |
| :--- | :--- | :--- | :--- |
| `src-tauri/src/modules/instance/pid_scan.rs` | Instance PID Discovery | Surgical Bugfix | Eliminate `instance_root_pids.contains(&p)` in child process traversal; check only `matched_set.contains(&p)` (Defect A1). |
| `src-tauri/src/modules/auto_switcher/selection.rs` | Auto-Switcher Selection | Surgical Bugfix | Call `score_candidate_account_fallback(acc, target_model, now_sec)` in Step 3 fallback pool (Defect A2). |
| `src-tauri/src/modules/instance/close.rs` | Instance Lifecycle Close | Namespace Repair | Replace `super::pid_scan::force_refresh_process_cache()` with `force_refresh_process_cache()` (Defect A4). |
| `src-tauri/src/modules/instance/lifecycle.rs` | Instance Restart | Parity Verification | Ensure `restart_instance` executes verified close followed by launch. |
| `src-tauri/src/modules/auto_switcher/manual.rs` | Fast-Forward & Manual Rotation | Parity Verification | Ensure `trigger_manual_rotation_for_instance` delegates to `switch_account_to_instance`. |

---

## 3. Surgical Work Packages (WP)

### WP-01: Fix Defect A1 in `src-tauri/src/modules/instance/pid_scan.rs`

- **Location**: `src-tauri/src/modules/instance/pid_scan.rs:291-315`
- **Issue**: Checking `matched_set.contains(&p) || instance_root_pids.contains(&p)` causes child processes of active instances to match stopped sibling instances, poisoning `matched_pids`.
- **Target Invariant**: A process is a child of an instance if and only if its ancestor walk terminates at a PID already contained within that specific instance's `matched_set`.
- **Exact Code Change**:
```rust
<<<<
            for _ in 0..10 {
                if let Some(&p) = parent_map.get(&curr) {
                    // Strictly match ancestors belonging to this specific instance
                    if matched_set.contains(&p) || instance_root_pids.contains(&p) {
                        matched_pids.push(pid_u32);
                        matched_set.insert(pid_u32);
                        break;
                    }
                    curr = p;
                } else {
                    break;
                }
            }
====
            for _ in 0..10 {
                if let Some(&p) = parent_map.get(&curr) {
                    // Strictly match ancestors belonging to this specific instance
                    if matched_set.contains(&p) {
                        matched_pids.push(pid_u32);
                        matched_set.insert(pid_u32);
                        break;
                    }
                    curr = p;
                } else {
                    break;
                }
            }
>>>>
```

### WP-02: Fix Defect A2 in `src-tauri/src/modules/auto_switcher/selection.rs`

- **Location**: `src-tauri/src/modules/auto_switcher/selection.rs:291-307`
- **Issue**: In Step 3 fallback candidate evaluation, calling `score_candidate_account(acc, target_model, now_sec)` strictly evaluates to `0.0` for any account with < 100% 4-hour quota, starving the rotation pool.
- **Target Invariant**: Step 3 fallback candidate evaluation must invoke `score_candidate_account_fallback(acc, target_model, now_sec)` to calculate proportional headroom-weighted scores.
- **Exact Code Change**:
```rust
<<<<
            let quota = calculate_candidate_quota(acc, target_model).unwrap_or(0.0);
            if quota > threshold {
                seen_account_ids.insert(acc.id.clone());
                let score = score_candidate_account(acc, target_model, now_sec);
                let candidate = ProfileCandidate {
                    instance_id: current_instance_id.to_string(),
                    account_id: acc.id.clone(),
                    email: acc.email.clone(),
                    quota_percent: quota,
                    score,
                };
====
            let quota = calculate_candidate_quota(acc, target_model).unwrap_or(0.0);
            if quota > threshold {
                seen_account_ids.insert(acc.id.clone());
                let score = score_candidate_account_fallback(acc, target_model, now_sec);
                let candidate = ProfileCandidate {
                    instance_id: current_instance_id.to_string(),
                    account_id: acc.id.clone(),
                    email: acc.email.clone(),
                    quota_percent: quota,
                    score,
                };
>>>>
```

### WP-03: Fix Defect A4 in `src-tauri/src/modules/instance/close.rs`

- **Location**: `src-tauri/src/modules/instance/close.rs` lines 75, 86, 117, 154, 160
- **Issue**: Calling `super::pid_scan::force_refresh_process_cache()` fails to compile because `force_refresh_process_cache` is defined in `pid_util.rs` and re-exported at the module level in `src-tauri/src/modules/instance.rs`.
- **Target Invariant**: Clean compilation by calling `force_refresh_process_cache()` directly (which is in scope via `use super::*;`).
- **Exact Code Changes**:
  - Line 75: `force_refresh_process_cache();`
  - Line 86: `force_refresh_process_cache();`
  - Line 117: `force_refresh_process_cache();`
  - Line 154: `force_refresh_process_cache();`
  - Line 160: `force_refresh_process_cache();`

### WP-04: Fast-Forward and Instance Restarting Parity Verification

- **Fast-Forward Call Chain**:
  1. Trigger: `trigger_manual_rotation_for_instance(target_instance_id)` in `src-tauri/src/modules/auto_switcher/manual.rs`.
  2. Backup: `crate::modules::repo_db::backup_running_prompts(&inst_id)`.
  3. Selection: `select_and_verify_next_best_profile(...)`.
  4. Execution: `execute_profile_rotation_with_context(...)` in `src-tauri/src/modules/auto_switcher/rotation.rs`.
  5. Switching: `crate::modules::instance::switch_account_to_instance(&target.account_id, Some(&target.instance_id)).await?`.
  6. Verified: Fast-Forward delegates directly to the unified account switch pipeline.

- **Verified Restart Call Chain**:
  1. Trigger: `restart_instance(instance_id)` in `src-tauri/src/modules/instance/lifecycle.rs`.
  2. Token Refresh: Account token refreshed prior to process termination.
  3. Close: `close_instance_verified(&resolved_id, Duration::from_millis(5000))`.
  4. Cache Invalidation: `crate::modules::repo_db::invalidate_prompt_tree_cache(Some(&resolved_id))`.
  5. Launch: `launch_instance(&resolved_id)`.
  6. Verification: `is_instance_process_running_smart(&resolved_id)` polling with an 8s ceiling.
  7. Verified: Restart verifies process death before relaunch and confirms process liveness after launch.

---

## 4. Step-by-Step Execution Checklist

- [ ] **Step 1: Workspace Hygiene & Pre-flight Inspection**
  - Verify target files exist and inspect current lines.
  - Confirm zero uncommitted working tree pollution.

- [ ] **Step 2: Apply WP-01 in `src-tauri/src/modules/instance/pid_scan.rs`**
  - Locate line 302: `if matched_set.contains(&p) || instance_root_pids.contains(&p)`.
  - Replace with `if matched_set.contains(&p)`.
  - Verify that only `matched_set.contains(&p)` is checked during instance child traversal.

- [ ] **Step 3: Apply WP-02 in `src-tauri/src/modules/auto_switcher/selection.rs`**
  - Locate line 294 in Step 3 fallback candidate loop.
  - Ensure `score_candidate_account_fallback(acc, target_model, now_sec)` is called.
  - Verify that partial 4-hour quota accounts receive non-zero candidate scores.

- [ ] **Step 4: Apply WP-03 in `src-tauri/src/modules/instance/close.rs`**
  - Inspect lines 75, 86, 117, 154, and 160.
  - Replace all occurrences of `super::pid_scan::force_refresh_process_cache()` with `force_refresh_process_cache()`.
  - Confirm clean module resolution.

- [ ] **Step 5: Verify Fast-Forward and Restart Call Chains**
  - Confirm `manual.rs:131` calls `execute_profile_rotation_with_context`.
  - Confirm `rotation.rs:275, 309` calls `switch_account_to_instance`.
  - Confirm `lifecycle.rs:47` calls `close_instance_verified` followed by `launch_instance`.

- [ ] **Step 6: Invariant & Code Quality Audit**
  - Audit all modified files for strictly relative Git paths.
  - Audit all boolean identifiers for positive naming (`isApplied`, `hasCompleted`, `isPeriodFinished`).
  - Audit error handling to ensure proper `AppError` routing with no silent error swallowing.
  - Verify no git commands were executed.

---

## 5. Acceptance Criteria

- **AC-01 (Process Sandboxing)**: Stopped sibling instances have zero PIDs returned by `find_pids_for_data_dir` when active instances are running child processes.
- **AC-02 (Switching Unblocked)**: `switch_account_to_instance` does not fail with false "actively bound to running sibling instance" errors.
- **AC-03 (Fallback Scoring)**: In `select_candidate_profiles`, candidate accounts with < 100% 4-hour quota receive non-zero scores when evaluated in Step 3 fallback pool.
- **AC-04 (Compilation Cleanliness)**: `close.rs` resolves `force_refresh_process_cache()` without compiler errors.
- **AC-05 (Fast-Forward Flow)**: Fast-forward triggers manual rotation and safely switches credentials in `state.vscdb`.
- **AC-06 (Restart Flow)**: Instance restart enforces verified termination before launching the new process tree.
- **AC-07 (Relative Paths)**: All paths across code, spec, and plan are strictly relative Git paths.
- **AC-08 (Positive Booleans)**: Boolean variables and flags strictly follow positive naming conventions.
