# Master Plan: Smart Instance Process Cache & Prompt Dispatch Fix

- **Slug**: `151-smart-instance-process-cache-and-prompt-dispatch-fix`
- **Date**: 2026-10-10
- **Lead Architect**: Md. Alim Ul Karim (@aukgit)
- **Status**: IN_PROGRESS

---

## 1. Executive Summary & Root Cause Architecture

Users reported that sending or enqueueing prompts from the instance section to the Antigravity IDE fails to take effect and repeatedly reopens/restarts running IDE instances instead of communicating with the active window. In addition, finished conversations falsely display the pulsing "RUNNING" state, and the Prompt Tree View is burdened with unnecessary bracket tags `[...]` and badge clutter.

### Core Discoveries:
1. **Unwanted IDE Relaunch & Process Termination Bug**:
   - `handleResendPrompt` in `PromptTreeViewModal.tsx` calls both `sendPromptNow` AND `focusInstanceWorkspace`.
   - `focusInstanceWorkspace` delegates to `ensure_instance_running_smart`, which checks `is_instance_process_running_smart`.
   - When checking PIDs, if the primary launcher PID exited while worker/window PIDs remained, or if `find_pids_for_data_dir` failed to match exact command lines due to Windows 8.3 path discrepancies, it concluded the instance was dead.
   - It then called `launch_instance_with_workspaces`, which invoked `close_instance()`—**explicitly killing all running IDE processes via `taskkill`** and spawning a cold window!
2. **Ghost Running False Positives**:
   - `inspect_conversation_transcript` used a shallow `take(5)` reverse scan, which got exhausted by trailing telemetry lines (`TOKEN_USAGE`, `HEARTBEAT`, etc.) before reaching completion events.
   - `is_terminal_done` required explicit `DONE`/`COMPLETED` strings, missing `MODEL` events with empty/missing status.
   - Fallback loaders in `compute_project_conversation_tree` set `is_running` without verifying if the owning instance process was actually alive on the OS.
   - Stale active prompt TTLs were set to an excessive 600s (10 minutes).
3. **UI Bracket & Badge Clutter**:
   - Literal bracket tokens `[Collapse Full Text]`, `[Expand Full Text]`, word counts `({count}w)` inside buttons, and noisy dual sequence codes (`C001 · 8159abcd`) cluttered the view.

---

## 2. Work Breakdown & Subtask Allocation

| Subtask ID | Title | Owner | Target Files |
| :--- | :--- | :---: | :--- |
| **Subtask-01** | Smart Process Cache, Closed-PID Re-Scan & Zero-Relaunch Guarantee | Worker 01 | `src-tauri/src/modules/instance.rs`, `src-tauri/src/commands/instance.rs` |
| **Subtask-02** | Ghost Running Eradication, Deep Transcript Scan & Terminal State Detection | Worker 01 | `src-tauri/src/modules/repo_db.rs` |
| **Subtask-03** | UI Bracket De-Cluttering, Sequence Badges & Action Bar Compaction | Worker 02 | `src/components/instances/PromptTreeViewModal.tsx`, `src/components/instances/InstanceTable.tsx` |
| **Subtask-04** | Host-Shielded E2E Testing, CLI Parity & Minor Bump Release Ceremony (`v4.172.0`) | Worker 02 | `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`, `package.json`, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md` |

---

## 3. Strict Rules & Quality Invariants
- Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- Strictly relative Git paths.
- Mandatory Multi-Agent Partitioning: `A = 2`, `H = 2` via `invoke_subagent`.
- Zero disruption to host IDE processes during test execution.
- Attribution strictly to `@aukgit` (`(Thanks to @aukgit)`).
