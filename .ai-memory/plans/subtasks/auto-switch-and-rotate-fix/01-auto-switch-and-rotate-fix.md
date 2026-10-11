# Subtask Implementation Plan: Auto-Switch Quota Fallback & Multi-Instance PID Decontamination

> **Subtask ID:** `01-auto-switch-and-rotate-fix`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Parent Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/01-architecture-spec.md`  
> **Remediation Spec:** `02-spec/21-app/auto-switch-and-rotate-fix/02-rca-and-remediation-spec.md`  
> **Target Files:**
> - `src-tauri/src/modules/instance/pid_scan.rs`
> - `src-tauri/src/modules/auto_switcher/selection.rs`
> - `src-tauri/src/modules/auto_switcher/scoring.rs`
> - `src-tauri/src/modules/instance/switch.rs`
> - `src-tauri/src/modules/auto_switcher/rotation.rs`
> **Status:** `[READY_FOR_WORKER]`  
> **Author:** @aukgit  
> **Assigned Role:** Worker Agent (Backend / Systems Specialist)

---

## 1. Context & Objectives

During production operation, two critical defects degrade Antigravity Manager instance isolation and autonomous account rotation:
1. **Cross-Instance PID Contamination (`pid_scan.rs:302`)**: Child process resolution checks `instance_root_pids.contains(&p)`, which contains root PIDs across **all** running instances. This causes stopped instances to falsely report as running and causes account switches on instance A to kill language servers or terminal shells running in instance B.
2. **Auto-Switcher 100% Quota Gate Lockup (`selection.rs:294`)**: When all accounts in the pool have sub-100% quota, Step 3 Fallback Pool invokes `score_candidate_account` instead of `score_candidate_account_fallback`. Because `score_candidate_account` strictly returns `0.0` for accounts with $< 100\%$ quota, all fallback candidates receive zero scores, causing the auto-switcher to abort rotation and leave depleted instances frozen indefinitely.
3. **Sibling Running Guard False Positive (`switch.rs:59`)**: Due to PID contamination, `is_instance_running` reports offline sibling instances as active, blocking account switches with *"Account is actively bound to running sibling instance"*.

This plan provides exact, surgical implementation instructions for Worker agents to resolve these issues with zero regression risk.

---

## 2. File Boundaries & Invariant Guarantees

Worker agents must touch **only** the designated target files. Do not modify unrelated proxy handlers, database schemas, or frontend UI components.

- **Invariant I1 (Strict Target PID Descent)**: For any non-default instance, child process descent in `find_pids_for_data_dir` must check `matched_set.contains(&p)` ONLY. Under no circumstances may `instance_root_pids` be queried in the child resolution loop.
- **Invariant I2 (Proportional Fallback Scoring)**: Any account with $q_{\text{4h}} > 0\%$ and weekly quota $\ge 8\%$ evaluated in Step 3 of `select_candidate_profiles` must receive a strictly positive score ($> 0.0$) proportional to its remaining credit headroom.
- **Invariant I3 (Zero Git Commands)**: Workers must NEVER execute raw `git` commands (`git add`, `git commit`, `git status`, `git push`, etc.).
- **Invariant I4 (No Search Commands)**: Workers must NEVER execute `grep`, `ripgrep`, `rg`, `Select-String`, or `findstr`. Use `gitmap find` or `view_file`.

---

## 3. Step-by-Step Implementation Instructions

### Step 1: Decontaminate Child PID Walk in `src-tauri/src/modules/instance/pid_scan.rs`
- Locate lines 291–315 in `src-tauri/src/modules/instance/pid_scan.rs`.
- In the `else` branch (handling instance data dirs where `!is_default`):
  ```rust
  // Line 301-308:
  for _ in 0..10 {
      if let Some(&p) = parent_map.get(&curr) {
          // REMOVE: || instance_root_pids.contains(&p)
          // REPLACE WITH:
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
  ```
- **Rationale**: `matched_set` is initialized with root PIDs matching the target instance's specific `data_dir` or cloned binary marker. Confining the check to `matched_set` guarantees that only genuine descendants of the target instance are captured.

### Step 2: Fix Fallback Candidate Scoring in `src-tauri/src/modules/auto_switcher/selection.rs`
- Locate Step 3 (Fallback Pool) in `src-tauri/src/modules/auto_switcher/selection.rs` (lines 254–308).
- Inspect the scoring call at line 294:
  ```rust
  // OLD (DEFECTIVE):
  let score = score_candidate_account(acc, target_model, now_sec);
  
  // NEW (REMEDIATED):
  let score = score_candidate_account_fallback(acc, target_model, now_sec);
  ```
- Verify that `score_candidate_account_fallback` is imported and in scope (it is declared `pub fn score_candidate_account_fallback` in `scoring.rs`).
- **Rationale**: `score_candidate_account` checks `if q_4h < 100.0 && !is_period_finished { return 0.0; }`. In fallback mode, accounts have $< 100\%$ quota by definition, so primary scoring evaluates to `0.0`. `score_candidate_account_fallback` correctly scales weekly score by the available 4-hour percentage.

### Step 3: Validate Sibling Running Guard in `src-tauri/src/modules/instance/switch.rs`
- Inspect `src-tauri/src/modules/instance/switch.rs` lines 54–68.
- Confirm that the sibling check:
  ```rust
  if inst.id != target_id && is_instance_running(&inst.id, &inst.data_dir, inst.pid)
  ```
  benefits from the decontaminated `find_pids_for_data_dir`.
- Ensure logging clearly displays the offending instance name and data directory if a genuine sibling conflict occurs.

### Step 4: Verify Prompt Database Connection Export in `src-tauri/src/modules/backup_prompts_db/mod.rs`
- Confirm `connect_db()` is properly re-exported with `pub fn` or `pub(crate) fn` in `src-tauri/src/modules/backup_prompts_db/mod.rs` so that `rotation.rs:238` builds without visibility errors across all platforms.

---

## 4. Verification & Quality Gates

Worker agents must run the following checks locally:

```pwsh
# 1. Rust Formatting
cd src-tauri && cargo fmt -- --check

# 2. Rust Clippy Gate
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Targeted Unit Tests
cd src-tauri && cargo test modules::instance::pid_scan
cd src-tauri && cargo test modules::auto_switcher::selection
cd src-tauri && cargo test modules::auto_switcher::scoring

# 4. Pipeline Telemetry Check
gitmap pe -t
```

---

## 5. Acceptance Criteria

- [ ] `find_pids_for_data_dir` returns `[]` for an offline instance even when sibling instances are running with multiple child workers.
- [ ] Account rotation succeeds when all available accounts in the pool have 80%–95% quota, selecting the candidate with the highest non-zero score.
- [ ] No `taskkill` or `SIGKILL` signals are sent to processes belonging to sibling instances during an account switch.
- [ ] `cargo clippy --all-targets --all-features` completes with zero errors and zero warnings.
