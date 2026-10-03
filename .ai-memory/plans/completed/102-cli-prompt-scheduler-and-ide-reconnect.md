# Parent Plan: 102 - CLI Prompt Scheduler, Audit Trail & IDE Reconnect

**Version:** 1.0.0  
**Status:** Completed & Verified  
**Spec Reference:** `02-spec/21-app/102-cli-prompt-scheduler-and-ide-reconnect.md`  
**Total Steps Budget (N):** 300  
**Concurrency Capacity:** A = 2, H = 2

---

## 1. User Request (Verbatim)
> First of all, when the tool is running, which the tool at least minimum CLI should run, let's say in every 10 minutes, that would be just bookkeeping and checking if there is a schedule. Schedule means some of those would be scheduled, like if the Antigravity IDE was running and some of the prompts are queued to this, let's say, conversations. So the CLI scheduler, let's create another CLI scheduler that would know using the AGM that there was queued prompts, enqueued prompts. Okay? Now, its job would be to understand and see if these enqueued prompts are, let's say, going to be working or not, is any prompt is running or not. So it should get that idea again. If enqueued prompts are not running or no prompts are running in those enqueued projects, it will again push first enqueued prompts automatically. So in every 10 minutes, and it should also be in the audit section to see what the scheduler did. The UI should be powerful enough to showcase that, and history should be very much protected so that we understand the trail log, what is going on. And I want you to do this end-to-end testing if it is possible for you. And also, the prompts running. So when the IDE gets back in, try to wait for, let's say, 10 seconds or five seconds to put the running conversation back into the enqueue and try to send the running conversation send back again to the IDE, Antigravity, okay, to that specific instance. Can you please confirm that? Okay. That is very, very crucial.

---

## 2. Granular Subtask Decomposition

- [x] **Subtask-01: Audit Action & Split-DB Schema Expansion** (`01-audit-action-and-schema.md`)
  - Add `AuditAction::SchedulePrompt = 4` and `AuditAction::RequeueConversation = 5` in `src-tauri/src/modules/audit_action.rs`.
  - Add `SchedulerFacts` payload structure and recording helper `record_scheduler_event` in `src-tauri/src/modules/task_history_db.rs`.

- [x] **Subtask-02: Enqueued Prompt Auto-Push Engine** (`02-prompt-queue-engine.md`)
  - Implement `check_and_dispatch_enqueued_prompts(instance_id: Option<&str>)` in `src-tauri/src/modules/repo_db.rs`.
  - Query projects with enqueued prompts; verify project idleness; dispatch first FIFO prompt; record audit event.

- [x] **Subtask-03: 10-Minute CLI Scheduler Command & Daemon** (`03-cli-scheduler-daemon.md`)
  - Implement `agm queue-scheduler` in `src-tauri/src/bin/agm.rs` (supporting loop mode and `--once` flag).
  - Register periodic 10-minute Tokio task in `src-tauri/src/modules/scheduler.rs` and `src-tauri/src/lib.rs`.

- [x] **Subtask-04: IDE Reconnect Stabilization Delay & Re-Enqueuing** (`04-ide-reconnect-and-requeue.md`)
  - Enhance `wait_for_instance_prompt_channel` in `src-tauri/src/modules/instance.rs` with 5-10s stabilization delay.
  - Implement `requeue_running_conversations_for_instance` in `src-tauri/src/modules/repo_db.rs`.
  - Wire into `launch_instance_inner` and instance switch routines.

- [x] **Subtask-05: Audit UI Scheduler Trail & History Inspector** (`05-audit-ui-scheduler-trail.md`)
  - Update `src/pages/Audit.tsx` with Scheduler badges, filter toggles, and detail inspectors.
  - Render scheduler run facts, project names, and prompt dispatch records.

- [x] **Subtask-06: End-to-End Testing & Pre-Flight Validation** (`06-e2e-testing-and-validation.md`)
  - Run pre-flight checks (`cargo fmt -- --check`, `npm run build`).
  - Update plans and subtask manifests.
