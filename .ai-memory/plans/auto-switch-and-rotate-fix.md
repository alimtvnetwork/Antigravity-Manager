# Execution Plan: Auto Switch, Rotate, Fast Forward & CLI Revert Fix

## User Request (Verbatim)
```text
# Auto Switch and Rotate Fix: high priority instruction, non-negotiable task

Yeah, I think it's been quite some time that you have been trying to fix the auto switch and rotate. None of this is working. The biggest problem here is that the switch of the account and fast forward, both of this does not work at all. So you can compare with 162, v4.162 version. That's auto-rotate. Auto-rotate and fast forward, restarting instance, all of these functionalities work. So try to go deep, understand very well what is the difference between your code and there, and also fix the CI/CD issues by checking the gitmap.pe. Don't try to run anything. Check the issues, try to find the files, the root cause, and try to fix it. Again, send it to GitHub CI/CD to fix it, okay? Don't try to be over smart at this moment. And try to figure out and write the root cause for me why your account switching is failing. I tried to revert back to the old version, version 1.62, and why I cannot do that automatically using the `Antigravity Manager (`Antigravity Manager (AGM)`)` update to v4.162 or just 4.162. Why I cannot? So that update option from the CLI also does not revert the version. Make sure you fix it properly everything. Okay? It's very crucial. I've been doing this for several time. You have been breaking stuff. This is stupid. Write the root cause, write the problem first, and then try to create the instruction for the AI first, okay? And then you start doing the task. So before doing that, plan it properly, write the instruction properly, find the root problem, go deep inside the code, do the comparison of this, what you have, what is there that is working. It is kind of stupid that you tried several times and it is not working.

## slug: auto-switch-and-rotate-fix

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/auto-switch-and-rotate-fix/ and enqueue plan task in .ai-memory/plans/auto-switch-and-rotate-fix.md (subtasks in .ai-memory/plans/subtasks/auto-switch-and-rotate-fix/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Compare current code with version 162, v4.162 to identify differences in auto-rotate and fast forward functionalities.
7. Investigate and fix CI/CD issues by checking gitmap.pe.
8. Identify and document the root cause of account switching failure.
9. Attempt to revert to version 1.62 using `Antigravity Manager (`Antigravity Manager (AGM)`)` and document why the CLI update option fails to revert the version.
10. Plan and write detailed instructions for AI before starting the task.
```

---

## 1. Problem Statement & Root Cause Summary

### A. Root Cause 1: Severe Cross-Instance PID Contamination (`pid_scan.rs`)
- In `src-tauri/src/modules/instance/pid_scan.rs:302`:
  Child process traversal checks `if matched_set.contains(&p) || instance_root_pids.contains(&p)`.
  Because `instance_root_pids` contains the root PIDs of **all** sandbox instances on the OS, children of any running instance (e.g., Instance B) are falsely attributed to all other stopped instances (e.g., Instance A).
- **Consequences**:
  1. `is_instance_running(&inst.id, ...)` returns `true` for all instances whenever any instance runs.
  2. Account switching in `switch.rs:59-67` rejects switching with `"Account is actively bound to running sibling instance..."`.
  3. Auto-switcher candidate pool skips all candidates in `selection.rs:123, 221, 281`.
  4. `close_instance_verified` loops checking `live_pids()`, stalling 10 seconds on restart.

### B. Root Cause 2: 100% Quota Gate & Broken Fallback Scoring (`selection.rs` & `scoring.rs`)
- In `src-tauri/src/modules/auto_switcher/scoring.rs:205-207`, `score_candidate_account` returns `0.0` if `q_4h < 100.0`.
- In `src-tauri/src/modules/auto_switcher/selection.rs:286`, Step 3 fallback erroneously calls `score_candidate_account` instead of `score_candidate_account_fallback`.
- In `selection.rs:383-395`, if no accounts have 100% quota, `select_and_verify_next_best_profile` immediately returns `Ok(None)`, failing auto-rotate and manual fast-forward with `"No alternative healthy profile found in pool"`.

### C. Root Cause 3: CLI Update Downgrade Revert Failure (`agm update to v4.162`)
- In `src-tauri/src/bin/agm/update_cmds.rs:16-31`, positional versions, `to <ver>`, and `--version` are never parsed.
- The command always queries GitHub `releases/latest`, sees `VERSION == latest`, prints `[UP TO DATE]`, and exits.
- Semantic versions like `4.162` or `v4.162` require normalization to `v4.162.0`.
- `delegate_updater/run.rs` and `install.ps1:1455` fail to pass/honor target pinned versions for downgrades.

### D. Root Cause 4: CI/CD Pipeline Compilation Errors (`gitmap pe -t`)
- `src-tauri/src/modules/process/running.rs:14, 21`: `find_pids_for_data_dir(&default_dir, true)` passes `&PathBuf` instead of `&str`.
- `src-tauri/src/modules/auto_switcher/selection.rs:538, 575`: Unit tests use `TokenData` without importing `crate::models::TokenData`.

---

## 2. Work Breakdown & Subtask Waves

| Task-ID | Subtask Title | Assigned Files | Status |
| :--- | :--- | :--- | :--- |
| **Task-01** | CI/CD Compiler Fixes & Type Alignment | `src-tauri/src/modules/process/running.rs`, `src-tauri/src/modules/auto_switcher/selection.rs` | QUEUED |
| **Task-02** | Cross-Instance PID Isolation Fix | `src-tauri/src/modules/instance/pid_scan.rs`, `src-tauri/src/modules/process/running.rs` | QUEUED |
| **Task-03** | Auto-Switcher Scoring Fallback & Rotation Gate | `src-tauri/src/modules/auto_switcher/selection.rs`, `src-tauri/src/modules/auto_switcher/scoring.rs` | QUEUED |
| **Task-04** | CLI Update Downgrade & Revert Support | `src-tauri/src/bin/agm/update_cmds.rs`, `src-tauri/src/modules/delegate_updater/args.rs`, `src-tauri/src/modules/delegate_updater/run.rs`, `install.ps1` | QUEUED |
| **Task-05** | Master AI Instruction & Verification | `02-spec/21-app/auto-switch-and-rotate-fix/03-ai-instructions.md`, tests | QUEUED |

---

## 3. Strict Execution Protocol

- **Search Primacy**: All searches via `gitmap` exclusively. No `rg`, `ripgrep`, `grep`, `Select-String`.
- **Relative Paths**: Only relative paths from repository root.
- **Zero Builds / Tests**: Do not run local compilation or test suites (R1).
- **Subagents**: Spawn subagents using `invoke_subagent` (A=2, H=2).
- **Git Safety**: Workers never run git commands. Lead commits atomically using `gitmap cpb`.
