---
name: agm-prompt-heartbeat-and-observer
description: Specialized skill for managing the 5-second real-time prompt goal heartbeat engine, workspace status documentation (AGM_INSTANCE_STATUS.md), prompt liveness verification, detached worker lifecycle (scripts/prompt_heartbeat_runner.py), and live instance observation (agm observe) in Antigravity-Manager.
---

# AGM Prompt Heartbeat & Instance Observation Engine

This skill defines the architectural invariants, background worker lifecycle, credential drift inspection, and operational procedures for Antigravity-Manager's real-time prompt heartbeat verification and live instance observation subsystem.

---

## 1. Subsystem Architecture Overview

The Prompt Heartbeat & Observation engine provides cryptographic proof and operational visibility that an in-flight prompt is running, safely suspended across account rotations, and faithfully resumed post-switch:

```
+----------------------------------------------------------------------------------------------------+
|                                    Project Workspace Directory                                      |
|    - .antigravity_goal_prompt.log (5s real-time tick log)                                          |
|    - .antigravity_goal_prompt.log.pid (detached worker process tracker)                            |
|    - .antigravity_goal_heartbeat.json (state snapshot: prompt_id, instance_id, iteration)          |
|    - AGM_INSTANCE_STATUS.md (operator markdown status document)                                   |
|    - .antigravity_resume_task.json (atomic post-switch task resumption payload)                    |
+-------------------------------------------------+--------------------------------------------------+
                                                  │
                                                  ▼
+----------------------------------------------------------------------------------------------------+
|                     Detached Background Runner (scripts/prompt_heartbeat_runner.py)                |
|    - Windows: CREATE_NEW_PROCESS_GROUP | 0x08000000 (DETACHED_PROCESS) with DEVNULL streams        |
|    - Cadence: Appends UTC timestamp every 5 seconds                                                |
|    - Continuity: Reads previous log Iteration: N and resumes at N+1 post-switch                    |
+-------------------------------------------------+--------------------------------------------------+
                                                  │
                                                  ▼
+----------------------------------------------------------------------------------------------------+
|                          Rust Backend Watchdog (src-tauri/src/modules/repo_db.rs)                  |
|    - inspect_prompt_goal_status: verifies log age against (interval_secs + 4.0s) tolerance        |
|    - ensure_prompt_goals_running_for_instance: auto-heals stalled workers post-switch              |
|    - update_workspace_status_file: updates AGM_INSTANCE_STATUS.md every 5 seconds                  |
+-------------------------------------------------+--------------------------------------------------+
                                                  │
                                                  ▼
+----------------------------------------------------------------------------------------------------+
|                               Live Observation CLI (agm observe / inspect)                          |
|    - Conscious PID resolution (find_pids_for_data_dir)                                             |
|    - Credential Drift Detection: state.vscdb injected email vs instances.json bound_email          |
|    - Active prompt count & real-time heartbeat freshness inspection                                |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Core Architectural Invariants

### Invariant 1: 5-Second Real-Time Heartbeat Cadence
The background worker (`scripts/prompt_heartbeat_runner.py`) writes entries strictly on a 5-second interval:
```text
[2026-09-30 02:15:30 UTC] [PID: 14208] Instance: inst_beta | Iteration: 12 | Status: RUNNING | PromptID: p_9912 | Target: e2e test verification
```
The entry must contain:
1. ISO-8601 UTC timestamp
2. Conscious PID of the worker process
3. Target instance identifier
4. Monotonically increasing iteration counter
5. Prompt identifier and summary snippet

### Invariant 2: Iteration Continuity Across Account Switches
When an instance is rotated (e.g. during auto-switch or fast-forward):
1. The heartbeat worker is cleanly terminated via `scripts/prompt_heartbeat_runner.py stop`.
2. When restarted post-switch, `prompt_heartbeat_runner.py` scans the existing `.antigravity_goal_prompt.log` backwards for `Iteration: (\d+)`.
3. If found, the runner resumes at `Iteration: N + 1`, preserving proof of continuous execution across credential boundaries.

### Invariant 3: Freshness Window & Auto-Healing Watchdog
- `repo_db.rs::inspect_prompt_goal_status` validates log file freshness against:
  $$\text{Tolerance} = \text{interval\_seconds} + 4.0\text{s} = 9.0\text{s}$$
- If `file_age > 9.0s`, the heartbeat is marked `is_fresh: false`.
- Upon post-switch restart, `ensure_prompt_goals_running_for_instance` checks active prompts in `repo_prompts.db`. If any prompt is in `dispatched` or `running` state with a stale heartbeat, it automatically spawns a fresh heartbeat runner.

### Invariant 4: Credential Drift Detection
`agm observe` / `instance.rs::observe_instance` queries:
1. The instance registry `instances.json` for `bound_email`.
2. The SQLite `state.vscdb` `ItemTable` via `db::read_injected_email`.
If `bound_email != injected_email`, `observe` flags `credential_drift: true` with an immediate warning, alerting the operator to an incomplete switch or external profile mutation.

### Invariant 5: Live Status Document (`AGM_INSTANCE_STATUS.md`)
The status document in the workspace root must be maintained with markdown tables showing:
- Active Instance ID & Profile Path
- Verified Process PIDs
- Authenticated Email (from SQLite)
- Running Prompt Queue & Heartbeat Ticks
- Last Verified Timestamp

---

## 3. CLI Command Reference

### Live Instance Observation
```bash
# Human-readable colored status box
agm observe <instance_id_or_path>

# Machine-readable JSON output
agm observe <instance_id_or_path> --json

# Observe default instance
agm observe default
```

### JSON Output Schema (`ObservedInstanceState`)
```json
{
  "instance_id": "test-cli-flow",
  "data_dir": "C:\\Users\\Administrator\\.antigravity_tools\\instances\\test-cli-flow\\data",
  "bound_email": "account-a@gmail.com",
  "injected_email": "account-a@gmail.com",
  "credential_drift": false,
  "pids": [14208, 18392],
  "active_prompts_count": 1,
  "heartbeat": {
    "is_running": true,
    "is_fresh": true,
    "last_iteration": 42,
    "pid": 14208,
    "log_path": "d:\\work\\my-project\\.antigravity_goal_prompt.log"
  }
}
```

### Heartbeat Background Runner Commands
```bash
# Start detached background heartbeat worker
python scripts/prompt_heartbeat_runner.py start --log-path <path> --instance <id> --prompt-id <id>

# Check heartbeat freshness and liveness
python scripts/prompt_heartbeat_runner.py check --log-path <path>

# Stop background worker cleanly
python scripts/prompt_heartbeat_runner.py stop --log-path <path>
```

---

## 4. Troubleshooting & Operational Guidance

1. **Heartbeat marked stale (`FRESH=False`)**:
   - Check if `<log_path>.pid` process is still alive: `tasklist /FI "PID eq <pid>"`.
   - Inspect if the project drive is under high disk I/O lock.
   - Run `agm observe <instance>` to trigger auto-healing watchdog.
2. **Credential Drift Detected (`credential_drift: true`)**:
   - Trigger `agm switch <account> --instance <id>` to re-inject tokens into all 3 SQLite database paths.
   - Verify that no lingering host IDE process locked `state.vscdb`.
3. **Orphaned Heartbeat Processes**:
   - `python scripts/prompt_heartbeat_runner.py stop --log-path <path>` forcibly terminates the worker and deletes the PID file.
