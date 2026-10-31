---
plan: 50-instant-rerun-enqueue-and-minor-release
subtask: "01"
title: Backend Queue Latency Elimination, Reactive Scheduler Notification & Immediate Dispatch
domain: backend-rust
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/scheduler.rs
  - src-tauri/src/commands/instance.rs
  - src-tauri/src/modules/repo_db/agy.rs
status: pending
---

# Subtask 01: Backend Queue Latency Elimination, Reactive Scheduler Notification & Immediate Dispatch

## 1. Context & Motivation
In Antigravity Manager, enqueued prompts and prompt reruns currently suffer from noticeable latency:
1. **30-Second Scheduler Backoff Sleep**: In `src-tauri/src/modules/scheduler.rs`, `start_prompt_queue_scheduler` sleeps for `PROMPT_QUEUE_IDLE_BACKOFF_INTERVAL = 30s` whenever the queue is empty. When a new prompt is enqueued, no event wakes up this background loop, parking prompts in SQLite for up to 30 seconds before dispatch.
2. **Missing Immediate Dispatch in IPC**: In `src-tauri/src/commands/instance.rs`, the `enqueue_prompt` IPC handler stores the prompt in SQLite and writes the resume file, but never checks if the target instance is currently idle to dispatch immediately.
3. **1.6-Second CDP Retry Blocking**: In `src-tauri/src/modules/repo_db/agy.rs`, `enqueue_prompt_for_instance` attempts synchronous CDP submission with a 4-iteration $\times$ 400ms retry loop, blocking thread execution for up to 1.6s on pure queue operations and risking keystroke collisions with active turns.

This subtask implements the backend changes to eliminate all artificial delays, achieving sub-10ms enqueue response times and sub-50ms immediate dispatch for idle instances while strictly maintaining FIFO queue integrity.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Functions | Planned Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/scheduler.rs` | `PROMPT_QUEUE_NOTIFIER`, `notify_prompt_queue`, `start_prompt_queue_scheduler` | 1. Define global `static PROMPT_QUEUE_NOTIFIER: Lazy<tokio::sync::Notify>`.<br/>2. Expose `pub fn notify_prompt_queue()` to signal pending work.<br/>3. Refactor `start_prompt_queue_scheduler` using `tokio::select!` to break both idle (30s) and active (5s) sleeps immediately upon signal. |
| `src-tauri/src/commands/instance.rs` | `enqueue_prompt` | 1. Invoke `crate::modules::scheduler::notify_prompt_queue()` upon successful enqueue.<br/>2. Spawn background blocking dispatch check via `check_and_dispatch_enqueued_prompts(Some(&resolved_id))` to execute immediately if the instance/project is idle. |
| `src-tauri/src/modules/repo_db/agy.rs` | `enqueue_prompt_for_instance` | Remove the synchronous, blocking CDP retry and submission logic (`submit_prompt_via_cdp_for_instance_with_mode`). Ensure pure enqueue operations perform only SQLite write and resume snapshot persistence in <10ms. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Implement `Notify` Mechanism in `src-tauri/src/modules/scheduler.rs`

1. Import `tokio::sync::Notify`:
   ```rust
   use tokio::sync::Notify;
   ```
2. Define the global static notifier:
   ```rust
   pub static PROMPT_QUEUE_NOTIFIER: Lazy<Notify> = Lazy::new(Notify::new);

   /// Notify the prompt queue scheduler that a prompt has been enqueued or state has changed,
   /// immediately breaking any idle backoff sleep.
   pub fn notify_prompt_queue() {
       PROMPT_QUEUE_NOTIFIER.notify_one();
   }
   ```
3. Update `start_prompt_queue_scheduler` to race sleep against `PROMPT_QUEUE_NOTIFIER.notified()`:
   - When `pending_count > 0`:
     ```rust
     tokio::select! {
         _ = tokio::time::sleep(PROMPT_QUEUE_ACTIVE_POLL_INTERVAL) => {},
         _ = PROMPT_QUEUE_NOTIFIER.notified() => {
             logger::log_info("[PromptQueueScheduler] Prompt queue notifier signaled during active poll; advancing ticker");
         }
     }
     ```
   - When `pending_count == 0`:
     ```rust
     tokio::select! {
         _ = tokio::time::sleep(PROMPT_QUEUE_IDLE_BACKOFF_INTERVAL) => {},
         _ = PROMPT_QUEUE_NOTIFIER.notified() => {
             logger::log_info("[PromptQueueScheduler] Prompt queue notifier signaled; waking ticker immediately from idle backoff");
         }
     }
     ```

---

### 3.2 Step 2: Augment `enqueue_prompt` in `src-tauri/src/commands/instance.rs`

1. In `src-tauri/src/commands/instance.rs`, after the prompt is saved and `res` is `Ok`:
   ```rust
   if res.is_ok() {
       crate::modules::scheduler::notify_prompt_queue();
       let dispatch_inst = resolved_id.clone();
       tokio::task::spawn_blocking(move || {
           match crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(&dispatch_inst)) {
               Ok(count) if count > 0 => {
                   crate::modules::logger::log_info(&format!(
                       "[enqueue_prompt] Immediate dispatch triggered for instance '{}': {} prompt(s) dispatched",
                       dispatch_inst, count
                   ));
               }
               Ok(_) => {
                   crate::modules::logger::log_info(&format!(
                       "[enqueue_prompt] Immediate dispatch check for instance '{}': instance busy or prompt queued for scheduler",
                       dispatch_inst
                   ));
               }
               Err(err) => {
                   crate::modules::logger::log_warn(&format!(
                       "[enqueue_prompt] Immediate dispatch check error for instance '{}': {}",
                       dispatch_inst, err
                   ));
               }
           }
       });
   }
   ```
2. Verify positive booleans:
   - `let is_prompt_saved = res.is_ok();`
   - Ensure no double-negative flags or unwrapped errors.

---

### 3.3 Step 3: Decouple CDP from `enqueue_prompt_for_instance` in `src-tauri/src/modules/repo_db/agy.rs`

1. In `src-tauri/src/modules/repo_db/agy.rs` inside `enqueue_prompt_for_instance`:
   - Locate lines 1052-1082 containing:
     ```rust
     // If instance is running, forward directly into Antigravity IDE UI queue via CDP (without cancelling running turn)
     let (is_running, _, pids) =
         crate::modules::instance::is_instance_process_running_smart(&canonical_inst);
     ...
     match crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance_with_mode(...) { ... }
     ```
   - Remove this block entirely.
   - **Rationale**: An enqueue operation records a pending item into the queue. CDP injection is part of the *dispatch* phase, performed safely only when `check_and_dispatch_enqueued_prompts` confirms the instance workspace is idle.
   - Removing this block eliminates the 1.6-second blocking retry loop, ensures SQLite writes complete in $< 10\text{ms}$, and prevents keystroke collisions with running IDE turns.

---

## 4. Verification & Testing Strategy

### 4.1 Local Pre-Flight Checks
Execute the required repository pre-flight gates:
```bash
# 1. Format check
cd src-tauri && cargo fmt -- --check

# 2. Clippy & compiler verification
cd src-tauri && cargo clippy --all-targets --all-features
```

### 4.2 Latency Verification
1. Measure `enqueue_prompt` IPC latency: Verify elapsed wall-clock time between invoking `enqueue_prompt` and receiving the `ActivePrompt` response is $< 20\text{ms}$.
2. Measure Idle Dispatch Latency: On an idle instance, verify that `check_and_dispatch_enqueued_prompts` dispatches the prompt turn within $< 50\text{ms}$.
3. Measure Wakeup Latency: On an empty queue with scheduler sleeping in 30s backoff, enqueue a prompt and verify in logs that `[PromptQueueScheduler] Prompt queue notifier signaled; waking ticker immediately from idle backoff` appears within $< 5\text{ms}$.

### 4.3 FIFO Queue Monotonicity Verification
1. Enqueue 3 prompts in rapid succession for a single workspace: `Prompt A`, `Prompt B`, `Prompt C`.
2. Inspect `active_prompts` table in SQLite (`repo_prompts.db`):
   ```sql
   SELECT id, prompt_content, status, created_at FROM active_prompts ORDER BY created_at ASC, id ASC;
   ```
3. Verify that prompts are dispatched strictly in order of creation (`Prompt A` first, then `Prompt B`, then `Prompt C`).

---

## 5. Rollback & Safeguards

- If immediate dispatch encounters an unexpected error during rapid multi-turn enqueue, the prompt record remains safely in SQLite with status `'queued'`.
- The background `start_prompt_queue_scheduler` will safely drain the prompt on its next 5-second tick once the workspace returns to idle.
- No database migrations or schema alterations are introduced, allowing immediate git rollback if needed with zero data loss or database incompatibility.
