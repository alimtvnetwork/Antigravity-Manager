# Application Specification: 102 - CLI Prompt Scheduler, Audit Trail & IDE Reconnect

**Version:** 1.0.0  
**Status:** Completed & Verified  
**Author:** Antigravity Engineering  
**Scope:** `src-tauri/src/modules/scheduler.rs`, `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/task_history_db.rs`, `src-tauri/src/modules/audit_action.rs`, `src-tauri/src/modules/instance.rs`, `src/pages/Audit.tsx`

---

## 1. Executive Summary & Problem Definition

In autonomous development workflows, developers enqueue complex tasks and prompts across Antigravity IDE instances. However, two critical operational gaps exist:
1. **Queue Stalling during Idle Windows:** If an IDE finishes a prompt or encounters an idle gap, enqueued prompts remain dormant in `repo_prompts.db` unless manually triggered. There is no automated bookkeeping scheduler that polls every 10 minutes to verify whether target projects are idle, and if so, automatically dispatches the first enqueued prompt.
2. **Conversation Dropping on IDE Reconnect:** When an Antigravity IDE instance is launched, restarted, or switches accounts, extension hosts and IPC channels require several seconds to boot. If prompt injection occurs too early (or without re-enqueuing), in-flight conversations are dropped or opened into blank sessions instead of continuing in the active thread.
3. **Audit Visibility Gaps:** Operators lack a dedicated trail log in the Audit UI showing what the 10-minute scheduler evaluated, which prompts were pushed, and how reconnect re-enqueuing took place.

---

## 2. Technical Architecture & Invariants

```
┌────────────────────────────────────────────────────────────────────────┐
│                   CLI / GUI 10-Minute Scheduler                        │
│             (agm queue-scheduler / background daemon)                  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ (Every 10 min / 600s)
                                    ▼
                 ┌──────────────────────────────────────┐
                 │  Scan Enqueued / Backed Up Prompts   │
                 │        in repo_prompts.db            │
                 └──────────────────┬───────────────────┘
                                    │
                         Is any prompt running
                         in target project?
                                ├── YES ──► Log "Project Busy" (No-op)
                                │
                                └── NO ───► Auto-Push First Enqueued Prompt
                                                 │
                                                 ├── 1. Write .antigravity_resume_task.json
                                                 ├── 2. Spawn via agy / prompt channel
                                                 ├── 3. Mark status = 'dispatched'
                                                 └── 4. Write AuditTask (Action: SchedulePrompt)
                                                         │
                                                         ▼
                                             Persist in Split-DB
                                           & Surface in Audit UI
```

### Invariant 1: 10-Minute Periodic Cadence
The scheduler must execute periodic bookkeeping every 10 minutes (600 seconds) both as a background Tokio task in the running app and as a dedicated CLI subcommand (`agm queue-scheduler` with optional `--once` flag for one-off runs and scripting).

### Invariant 2: Idle Safety Gate (No Collision)
A prompt must NEVER be pushed to a project if another prompt is currently executing (`status = 'running'` or `conversation_summaries.db` reports active execution). The scheduler must strictly evaluate project idleness before selecting the first FIFO enqueued prompt (`status IN ('backed_up', 'queued', 'pending') ORDER BY created_at ASC LIMIT 1`).

### Invariant 3: Audit Trail Immutability
Every scheduler run and prompt dispatch must record an immutable row in the Split-SQLite audit log (`tasks` table) with `action_code = 4` (`SchedulePrompt`), detailing the target instance, repository, prompt ID, prompt preview, and trail facts.

### Invariant 4: IDE Reconnect Stabilization Delay (5-10s)
When an IDE instance reconnects or starts:
1. Wait for `DevToolsActivePort` or conscious PID detection.
2. Enforce a mandatory 5-second settle delay to allow Electron and extension hosts to fully initialize.
3. Automatically re-enqueue any running/interrupted conversations into `repo_prompts.db` and write `.antigravity_resume_task.json` with `auto_boot: true` and the preserved `session_id`/`conversation_id`.
4. Inject the running conversation back into the IDE.

---

## 3. Data Schemas & Contracts

### 3.1 Audit Action Code Extension (`audit_action.rs`)
```rust
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    AddAccount = 1,
    UpdateAccount = 2,
    SwitchAccount = 3,
    SchedulePrompt = 4,
    RequeueConversation = 5,
}
```

### 3.2 Scheduler Audit Payload Schema (`SchedulerFacts`)
Stored in `tasks.payload_json`:
```json
{
  "scheduler_run_id": "sched-20261003-120000",
  "project_name": "Antigravity-Manager",
  "repo_path": "d:\\work\\Antigravity-Manager",
  "instance_id": "default",
  "prompt_id": "prompt-abcd1234",
  "prompt_preview": "Refactor scheduler loop...",
  "conversation_id": "conv-5678efgh",
  "action_taken": "dispatched",
  "reason": "Project idle; enqueued prompt pushed automatically",
  "idle_check_passed": true,
  "timestamp": 1727932000
}
```

---

## 4. Execution Plan & Implementation Steps

1. **Step 1 - Core Action & DB Extension:**
   - Update `AuditAction` enum with `SchedulePrompt` and `RequeueConversation`.
   - Update `task_history_db.rs` with `SchedulerFacts` payload builder and recording helpers.
2. **Step 2 - Queue Scheduler Engine:**
   - Implement `check_and_dispatch_enqueued_prompts()` in `repo_db.rs` / `scheduler.rs`.
   - Register `agm queue-scheduler` and `agm scheduler` CLI subcommands in `agm.rs`.
   - Wire the 10-minute loop into app startup daemon (`lib.rs`).
3. **Step 3 - Reconnect Stabilization & Re-enqueuing:**
   - Enhance `wait_for_instance_prompt_channel` in `instance.rs` with 5-10s settle delay.
   - Implement `requeue_running_conversations_for_instance()` in `repo_db.rs`.
   - Integrate into `launch_instance_inner` and switch flows.
4. **Step 4 - Audit UI Enhancements:**
   - Update `src/pages/Audit.tsx` with Scheduler badges, filter chips, and trail inspector.
5. **Step 5 - Testing & Verification:**
   - Unit tests for idle detection and prompt re-queuing.
   - Live CLI execution with `--once`.
