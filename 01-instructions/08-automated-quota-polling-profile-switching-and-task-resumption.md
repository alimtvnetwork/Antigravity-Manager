# Automated Quota Polling, Profile Auto-Switching, and Pending Task Resumption Architecture

> **Document ID:** `01-instructions/08-automated-quota-polling-profile-switching-and-task-resumption.md`  
> **Status:** Approved Architectural Specification  
> **Target Platforms:** Ubuntu / Debian Linux, Windows 10/11, macOS  

---

## Executive Feasibility Verdict

### 1. Is It Possible?
**YES, 100% technically feasible.**

There are two complementary layers to achieve this with zero work interruption:
1. **IDE Profile Level (Process Level)**: Automatically check the quota of the account currently bound to the active IDE instance on a configurable timer (e.g. 60s, 120s). If token quota drops below a threshold (e.g. `< 10%`), select the next best healthy profile, snapshot any pending task state, inject new credentials into SQLite `state.vscdb`, restart the IDE window, and resume the task.
2. **Proxy Token Pool Level (Zero-Downtime Hot Failover)**: If Antigravity routes through the built-in local proxy (`127.0.0.1:8045`), in-flight streaming LLM requests that hit rate limits or low quota can fail over to the next account *without restarting the IDE window at all*, keeping the active prompt completely seamless.

Combining both strategies provides maximum reliability: the proxy prevents in-flight prompt crashes, while the background profile supervisor ensures the IDE's local workspace credentials always reflect a healthy, high-quota account.

---

## System Architecture Overview

```
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                Background Supervisor Daemon (modules::auto_switcher)        │
 ├─────────────────────────────────────────────────────────────────────────────┤
 │                                                                             │
 │  ┌───────────────────────────────────────────────────────────────────────┐  │
 │  │ Timer Loop (Tick every T seconds, default: 60s, range: 15s - 600s)     │  │
 │  └───────────────────────────────────┬───────────────────────────────────┘  │
 │                                      │                                      │
 │                                      ▼                                      │
 │  ┌───────────────────────────────────────────────────────────────────────┐  │
 │  │ 1. Identify Active IDE Instance & Bound Account                        │  │
 │  │    - Read instance config from instances.json                          │  │
 │  │    - Query live quota for evaluated models (e.g. Gemini Pro / Flash)  │  │
 │  └───────────────────────────────────┬───────────────────────────────────┘  │
 │                                      │                                      │
 │                           Is Quota < Threshold?                             │
 │                           (default: < 10%)                                  │
 │                                      │                                      │
 │                 ┌────────────────────┴────────────────────┐                 │
 │                 ▼ No                                      ▼ Yes             │
 │          ┌─────────────┐             ┌───────────────────────────────────┐  │
 │          │ Continue    │             │ 2. Select Next Best Profile       │  │
 │          │ Sleep Loop  │             │    - Healthy quota (> threshold)  │  │
 │          └─────────────┘             │    - Longest time to reset        │  │
 │                                      └─────────────────┬─────────────────┘  │
 │                                                        │                    │
 │                                                        ▼                    │
 │                                      ┌───────────────────────────────────┐  │
 │                                      │ 3. Snapshot Pending QE Task State │  │
 │                                      │    - Parse conversation transcript│  │
 │                                      │    - Backup uncompleted subtasks  │  │
 │                                      │    - Record workspace memento     │  │
 │                                      └─────────────────┬─────────────────┘  │
 │                                                        │                    │
 │                                                        ▼                    │
 │                                      ┌───────────────────────────────────┐  │
 │                                      │ 4. Graceful Rotate & Token Inject │  │
 │                                      │    - Close active instance PID    │  │
 │                                      │    - Inject tokens into state.vscdb│ │
 │                                      │    - Update registry active target│  │
 │                                      └─────────────────┬─────────────────┘  │
 │                                                        │                    │
 │                                                        ▼                    │
 │                                      ┌───────────────────────────────────┐  │
 │                                      │ 5. Relaunch IDE & Resume Task     │  │
 │                                      │    - Spawn with --user-data-dir   │  │
 │                                      │    - Trigger task resumption loop │  │
 │                                      └───────────────────────────────────┘  │
 └─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Configurable Timer & Low-Quota Evaluation

### A. Configuration Schema
Stored in `gui_config.json` under `auto_profile_switcher`:

```json
{
  "auto_profile_switcher": {
    "is_enabled": true,
    "check_interval_seconds": 60,
    "low_quota_threshold_percent": 10.0,
    "target_model": "gemini-pro",
    "fallback_models": ["gemini-flash"],
    "auto_resume_pending_tasks": true,
    "cooldown_after_switch_seconds": 180,
    "notification_enabled": true
  }
}
```

### B. Parameters Specification
| Parameter | Default | Valid Range | Description |
| :--- | :--- | :--- | :--- |
| `is_enabled` | `false` | Boolean | Master switch for background quota supervision. |
| `check_interval_seconds` | `60` | `15` – `600` | Polling interval in seconds (e.g. 60s, 120s, 300s). |
| `low_quota_threshold_percent` | `10.0` | `1.0` – `50.0` | Quota percentage considered "exhausted / low". |
| `target_model` | `"gemini-pro"` | Model slug | Primary model evaluated for quota health. |
| `auto_resume_pending_tasks` | `true` | Boolean | Whether to automatically snapshot and resume pending tasks. |
| `cooldown_after_switch_seconds`| `180` | `30` – `900` | Minimum delay before another auto-switch can trigger. |

### C. Quota Scoring & "Next Best Profile" Selection
When the active profile drops below `low_quota_threshold_percent`:
1. **Candidate Pool**: Filter all registered profiles where:
   - Profile has a bound Google account.
   - Account is enabled (`is_disabled == false`).
   - Profile is not in cooldown.
2. **Scoring Function**:
   $$\text{Score} = (\text{Remaining Quota \%} \times 0.6) + (\text{Time Until Reset in Hours} \times 0.2) - (\text{Recent Error Count} \times 10)$$
3. **Winner Selection**:
   The candidate profile with the highest positive score is selected as the switch target.

---

## 2. Technical Feasibility of Pending QE Task Backup & Resumption

### A. Where Pending Tasks Live in Antigravity / VS Code
1. **Brain Transcripts & JSONL Logs**:
   - Location: `<appDataDir>/brain/<conversation-id>/.system_generated/logs/transcript.jsonl`
   - Contains completed steps, pending user prompts, and planner actions.
2. **Antigravity Plan Subtasks**:
   - Location: `.lovable/plans/subtasks/` or `.lovable/plans/pending/`
   - Lists numbered atomic tasks with checkboxes (`[ ]` uncompleted, `[x]` completed).
3. **VS Code Workspace Memento**:
   - Location: `<user-data-dir>/User/workspaceStorage/<hash>/state.vscdb`
   - Stores active open tabs, unsaved text buffers, and extension session keys.

### B. Task State Snapshot Protocol
Before terminating an IDE instance during a quota failover:
1. **Locate Active Workspace Storage**:
   Inspect `<user-data-dir>/User/workspaceStorage/` for the most recently modified database.
2. **Extract Unfinished Goals**:
   Read `.lovable/plans/01-index.md` and `.lovable/plans/pending/` to retrieve the current `/goal` text and uncompleted checklist numbers.
3. **Write Snapshot Artifact**:
   Save a recovery snapshot to `<config_dir>/task_recovery/snapshot_<instance_id>.json`:
   ```json
   {
     "instance_id": "profile-alpha",
     "account_id": "prev-account-1",
     "timestamp": 1773558000,
     "workspace_path": "/work/Antigravity-Manager",
     "pending_goal": "/goal Autonomously orchestrate parent task...",
     "next_subtask_path": ".lovable/plans/subtasks/14-multi/02-dispatch.md",
     "is_recovered": false
   }
   ```

### C. Safe Restart & Credential Swap
1. Send `SIGTERM` (`kill -15` on Linux/macOS, graceful close message on Windows) to allow VS Code to flush disk buffers to SQLite.
2. Wait up to 3 seconds for clean exit; if process lingers, force terminate.
3. Inject the target account's OAuth credentials into `<user-data-dir>/User/globalStorage/state.vscdb`.
4. Relaunch Antigravity with:
   ```bash
   antigravity --user-data-dir="<target_data_dir>" "<workspace_path>"
   ```

### D. Automated Task Resumption Mechanism
To resume the pending task automatically after the window opens:
- **Approach 1 (Agent Self-Resumption via Hook)**:
  When Antigravity boots, the master orchestrator checks `<config_dir>/task_recovery/` on startup. If an uncompleted recovery snapshot exists, it reads `pending_goal` and `next_subtask_path` and immediately continues execution without waiting for user input.
- **Approach 2 (Proxy Re-Dispatch)**:
  If the request was queued in the proxy middleware, the proxy simply retries the buffered request using the new account's token pool credentials.

---

## 3. Implementation Roadmap

### Phase 1: Configuration & UI Controls
- [ ] Add `AutoSwitchConfig` struct to `src-tauri/src/models/config.rs`.
- [ ] Add timer interval slider (15s–300s) and quota threshold input (5%–25%) in `src/pages/Settings.tsx` and `src/pages/Instances.tsx`.
- [ ] Expose IPC commands `get_auto_switch_config` and `save_auto_switch_config`.

### Phase 2: Background Supervisor Daemon
- [ ] Create `src-tauri/src/modules/auto_switcher.rs`.
- [ ] Spawn async loop inside `src-tauri/src/lib.rs` alongside `scheduler.rs`.
- [ ] Implement `check_active_instance_quota()`.
- [ ] Implement `select_best_fallback_profile()`.

### Phase 3: Task Snapshot & Recovery
- [ ] Create `snapshot_active_task(instance_id)` in `modules/task_recovery.rs`.
- [ ] Save recovery state in `<config_dir>/task_recovery/`.
- [ ] Integrate recovery inspection into startup CLI and GUI initialization.

---

## Summary

| Feature | Technical Feasibility | Recommended Default |
| :--- | :--- | :--- |
| **Timer Polling** | **100% Feasible** | Every 60 seconds (configurable 15s–600s) |
| **Low Quota Trigger** | **100% Feasible** | When remaining quota < 10% |
| **Profile Selection** | **100% Feasible** | Highest available quota with earliest reset |
| **Credential Injection** | **100% Feasible** | Direct atomic write to SQLite `state.vscdb` |
| **Task State Backup** | **100% Feasible** | JSON snapshot of uncompleted `/goal` & subtasks |
| **Task Resumption** | **100% Feasible** | Auto-detected on profile relaunch |
