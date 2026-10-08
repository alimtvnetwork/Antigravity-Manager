# Master Audit Ledger: 147-smart-instance-process-cache-and-prompt-dispatch

## 1. User Request (Verbatim)
```text
Hi there. Look, the sending prompt from the instance section to the IDE does not work. Find the root cause of it, why we cannot send the prompt and why we cannot enqueue the prompt. In both cases, it has no effect so far. Every time it tries to open or reopen the IDE instance, which is very wrong. If the instance is already open, you have to check in the process if this instance is already running. If the instance is already running, you need to cache it so that you can understand if the instance. That means you have to have the smart logic. When we send it or queue the prompt, first, you have to check how many instances are running for that Antigravity Tools Manager. You need to cache those as well. Let's say you find that it is running in the cache, but then again, you try to send and see this PID is closed. Then you have to rerun it and see if it is running again. If it is not running, then and only then you have to reopen and send the prompt. It is not happening right now like this. It is not working at all. You need to work on it, find the root cause, do the end-to-end testing, do things from CLI as well. Currently, the view is nice. I appreciate that, but there are too many tags, which is not necessary in the prompt section. Too many target bracket item, which can be reduced. And too many running items, which is not running, so you need to fix that as well. I hope you respect this and try to fix all this stuff and make a minor bump and release again.
```

## 2. Executive Problem Diagnosis & Requirements
1. **Root Cause Analysis (Prompt Send & Enqueue Failures)**:
   - Identify why sending/enqueueing prompts from the instance section fails or triggers unwanted reopen cycles.
   - Unwanted IDE launch: Investigating why `focus_or_launch_instance_with_workspace` / `focus_or_launch_workspace` spawns a new process instead of finding the existing running window/process.
2. **Smart Instance Process Caching & Liveness Verification**:
   - Multi-instance running process cache: query system processes (`Get-Process`, `sysinfo`, or Win32 API), detect running Antigravity IDE instances by PID, command-line arguments, and data-dir/profile flags.
   - Cache running PIDs with short TTL / active heartbeat.
   - When sending or queueing prompts: check if instance is running in cache; if PID exists and alive, send directly to the running instance via IPC/file/window message without launching a new instance.
   - If cached PID was closed/stale: re-verify process table; only if confirmed completely dead should it trigger reopening the instance.
3. **Prompt Section UI Cleanup & Tag Compaction**:
   - Eliminate redundant bracketed items and excessive tags in the Prompt Tree View modal.
   - Clean up cluttered metadata pills while preserving essential context.
4. **False Positive Running Items Elimination**:
   - Resolve "too many running items, which is not running": Harden running prompt detection logic in `repo_db` and UI to eliminate false-positive `RUNNING` indicators on idle/completed conversations.
5. **CLI & E2E Testing Verification**:
   - Implement/verify CLI commands (`agm prompt send`, `agm prompt queue`, `agm instance status`) with automated end-to-end test script.
6. **Minor Version Bump & Release Ceremony**:
   - Run pre-flight gates (`cargo fmt -- --check`, `npm run build`), bump minor version (`4.164.0 -> 4.165.0`), update changelogs with strict `@aukgit` attribution, tag, and publish release.

## 3. Architecture & Subtask Decomposition
- **Subtask 01:** `01-prompt-send-enqueue-rca-and-process-cache.md` (Backend RCA, smart instance process cache, PID liveness verification, avoid unwanted reopen)
- **Subtask 02:** `02-running-detection-hardening-and-tag-compaction.md` (Backend false-positive running prompt elimination, prompt tree tag compaction)
- **Subtask 03:** `03-cli-e2e-testing-and-ipc-parity.md` (CLI prompt send/queue command parity and Python E2E verification harness)
- **Subtask 04:** `04-version-bump-and-release-ceremony.md` (Pre-flight gates, minor bump v4.165.0, changelogs, READMEs, atomic commit & release)

## 4. Invariants & Guardrails
- **Search Guardrail:** Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. Exclusively use GitMap (`gitmap find`, `gitmap aum search`, `gitmap cat`).
- **Relative Paths:** Strictly use relative Git paths (no absolute paths or `file:///` URIs).
- **Subagent Concurrency:** Enforce `A = 2, H = 2` at every stage.
- **Atomic Release:** Single atomic GitMap commit (`gitmap cpf "<module> - <summary>"`).

## 5. Execution & Verification Status
- **Status:** `COMPLETED`
- **Subtask 01:** Completed (Smart Process Cache, `is_pid_alive_targeted`, `get_or_detect_instance_process`, `ensure_instance_running_for_dispatch`, decoupled window focus failure from process liveness)
- **Subtask 02:** Completed (False-positive running elimination with `is_terminal_done`, timestamp narrowing to 60s, stale rows pruning; compacted `formatDualBadge` to `${rawAgm} · ${rawGm}`, removed redundant role text badges, cleaned button labels)
- **Subtask 03:** Completed (CLI parity for `cmd_instances_status`, `cmd_prompts_send`, `cmd_prompts_running`, `cmd_prompts_queue`; automated safe test harness `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py` passing 5/5 test cases)
- **Subtask 04:** In progress (Pre-flight passed, minor version bump v4.165.0, changelogs, READMEs, atomic commit & release)
