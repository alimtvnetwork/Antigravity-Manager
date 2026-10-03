# CLI Prompt Queue Scheduler, Audit Trail & IDE Reconnect

## 1. Overview & Architecture
Autonomous prompt execution relies on background scheduling, idle detection, reconnect stabilization, and Split-DB audit logging.

- **CLI Scheduler**: `agm queue-scheduler` (aliases `agm scheduler`, `agm qs`, supports `--once`).
- **Background Daemon**: 10-minute (600s) interval Tokio loop initialized in `src-tauri/src/lib.rs` and `src-tauri/src/modules/scheduler.rs`.
- **Audit Logging**: Recorded in `data/task-history/` Split-SQLite databases via `task_history_db.rs`.
- **UI Surface**: `src/pages/Audit.tsx` with dedicated filters, color-coded badges, and payload inspection.

---

## 2. Root Cause Analysis (RCA) & Pitfalls to Avoid

### Pitfall 1: False-Busy Classification from `active_prompts`
- **Root Cause**: `get_live_project_execution_info()` in `repo_db.rs` queries `active_prompts WHERE status = 'running' OR status = 'queued'` and flags `is_running = true`.
- **Consequence**: When an enqueued prompt was waiting in the queue, `is_prompt_running_for_project()` evaluated the project as BUSY, permanently blocking the scheduler from auto-pushing the prompt.
- **Rule for Future AI**: Never use UI aggregate helper functions like `get_live_project_execution_info()` inside queue dispatchers. Direct inspection of `conversation_summaries.db` (checking `not_fully_idle != 0 || status.contains("RUNNING") || age < 600`) is mandatory to distinguish real in-flight executions from dormant/queued entries.

### Pitfall 2: Premature Prompt Injection on IDE Startup
- **Root Cause**: Launching an Antigravity IDE instance takes 5–8 seconds for Chromium/Electron processes and language server extension hosts to bind IPC ports.
- **Consequence**: Injecting prompts immediately upon PID detection resulted in dropped messages, timeout errors, or the IDE creating blank conversation sessions.
- **Rule for Future AI**: In `instance::wait_for_instance_prompt_channel`, enforce a minimum 5.0–6.0 second settle delay after `DevToolsActivePort` or conscious PID detection before restoring or dispatching prompts.

### Pitfall 3: Dropping Conversation Context on Instance Switch
- **Root Cause**: Account switching terminated instances without backing up in-flight prompts, or restored prompts without their original `session_id`/`conversation_id`.
- **Rule for Future AI**: Always invoke `requeue_running_conversations_for_instance(&instance.id)` before instance restart/rotation. Persist `session_id` into `.antigravity_resume_task.json` with `auto_boot: true` so the Antigravity IDE re-attaches to the ongoing conversation thread instead of initiating a new thread.

---

## 3. Split-DB Audit Trail Specifications

Audit Action Codes:
- `1` = `AddAccount`
- `2` = `UpdateAccount`
- `3` = `SwitchAccount`
- `4` = `SchedulePrompt` (Purple badge `#8b5cf6`)
- `5` = `RequeueConversation` (Cyan badge `#06b6d4`)

Every scheduler dispatch records `SchedulerFacts`:
```json
{
  "scheduler_run_id": "sched-20261003-120000",
  "project_name": "Antigravity-Manager",
  "repo_path": "d:\\work\\Antigravity-Manager",
  "instance_id": "default",
  "prompt_id": "prompt-xyz",
  "prompt_preview": "...",
  "conversation_id": "...",
  "action_taken": "dispatched",
  "reason": "Project verified idle; enqueued prompt pushed automatically",
  "idle_check_passed": true,
  "timestamp": 1727932000
}
```

---

## 4. Key Files & Modules
- `src-tauri/src/modules/audit_action.rs`: Enum definition and action code mapping.
- `src-tauri/src/modules/task_history_db.rs`: Split-DB schema and `record_scheduler_event`.
- `src-tauri/src/modules/repo_db.rs`: `is_prompt_running_for_project`, `check_and_dispatch_enqueued_prompts`, `requeue_running_conversations_for_instance`.
- `src-tauri/src/modules/instance.rs`: `wait_for_instance_prompt_channel` stabilization delay.
- `src-tauri/src/modules/scheduler.rs`: 10-minute Tokio background ticker.
- `src-tauri/src/bin/agm.rs`: CLI command `agm queue-scheduler [--once]`.
- `src/pages/Audit.tsx` & `src/types/audit.ts`: Frontend Audit Trail UI and filtering.
