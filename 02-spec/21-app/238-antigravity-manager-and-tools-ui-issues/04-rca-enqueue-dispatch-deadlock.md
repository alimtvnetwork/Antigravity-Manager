# 4-Part Root Cause Analysis: Enqueue & Rerun IDE Dispatch Deadlock & Turn Evaluation

**Specification ID**: `02-spec/21-app/238-antigravity-manager-and-tools-ui-issues/04-rca-enqueue-dispatch-deadlock.md`  
**Related Spec**: `02-spec/21-app/238-antigravity-manager-and-tools-ui-issues/01-architecture-spec.md`  
**Component Scope**: `src-tauri/src/modules/repo_db/queue.rs`, `src-tauri/src/modules/repo_db/agy.rs`, `src-tauri/src/modules/repo_db/transcript.rs`, `src-tauri/src/modules/repo_db/liveness.rs`, `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`

---

## 1. Symptom

1. **User Observation**: When the user clicks the "Enqueue" button in the Prompt Tree View, or clicks "Rerun" (which dispatches messages via the FIFO enqueue flow), the prompt never fires or appears inside the running Google Antigravity IDE composer. The IDE input area remains empty (`"Ask anything, @ to mention, / for actions"`), and no enqueued item appears in the IDE's queued turn stack.
2. **UI Count Inversion & Hallucinated Queue State**:
   - In `media_1793435783163_3985db8b.png`, the conversation header displays an amber badge `QUEUED 10` with the banner:
     `10 prompt(s) enqueued in FIFO queue for this conversation.`
   - The action button displays `Enqueue 10 Q`.
   - In the left sidebar tree, Turn T1 is displayed as emerald `RUNNING`, while 10 past completed turns (T2–T11) are all displayed with amber badges as `QUEUED`.
   - Despite displaying 10 enqueued items, clicking "Enqueue" does nothing to dispatch them to the IDE.

---

## 2. Root Cause Analysis (4 Defect Pillars)

### Pillar A: Premature Idle Guard Aborts CDP Submission in Background Scheduler
In `src-tauri/src/modules/repo_db/queue.rs` lines 126–141:
```rust
let is_running = is_prompt_running_for_project(&project_id, &inst_id)
    || is_prompt_running_for_project(&repo_path, &inst_id);

if is_running {
    crate::modules::logger::log_info(&format!(
        "[PromptQueueScheduler] Project '{}' ({}) is currently active/busy; enqueued prompts remain queued",
        clean_project_name, project_id
    ));
    continue;
}
```
- **Flaw**: This guard executed *before* line 240, where live DevTools CDP dispatch is handled.
- **Consequence**: Whenever an agent is executing a task in the IDE, `is_prompt_running_for_project` evaluates to `true`. The scheduler executed `continue`, aborting before reaching the CDP dispatch block.
- **Architectural Error**: Antigravity IDE is *not* a single-turn blocking CLI; it features native multi-turn conversation queuing. When text is inserted and submitted via CDP with `cancel_running_first: false`, Antigravity IDE natively accepts and stages the turn into its live turn queue (`[aria-label="Collapse queued messages"]`). By blocking CDP submission behind `is_running`, the scheduler created a perpetual deadlock, holding prompts in SQLite while the IDE sat ready to receive them.

### Pillar B: Gate 4 Zombie Liveness Lock in `liveness.rs`
In `src-tauri/src/modules/repo_db/liveness.rs` line 152:
```rust
let max_allowed_elapsed = if not_fully_idle > 0 { 600 } else { 60 };
```
- **Flaw**: Antigravity IDE frequently writes `not_fully_idle = 1` or `CASCADE_RUN_STATUS_RUNNING` into `conversation_summaries.db` during model execution, but does not immediately clear it upon completion if no subsequent turn is started.
- **Consequence**: Gate 4 treats any project with `not_fully_idle > 0` as actively running for up to 600 seconds (10 minutes) after the last modified timestamp. Even when the assistant was completely finished and waiting for user input, Gate 4 returned `true`, keeping the queue scheduler locked for 10 minutes.

### Pillar C: Missing Status Transition in `enqueue_prompt_for_instance`
In `src-tauri/src/modules/repo_db/agy.rs` lines 1121–1144:
```rust
if is_running {
    let cdp_res = submit_prompt_via_cdp_for_instance_with_mode(&canonical_inst, fallback_prompt, false, target_is_new);
    match cdp_res {
        Ok(()) => { /* Only logged info, did not update status */ }
        Err(ref cdp_err) => { /* Logged warning */ }
    }
}
```
- **Flaw**: When `enqueue_prompt_for_instance` successfully delivered the prompt into the live IDE via DevTools CDP, it failed to transition `active_prompt.status` from `'queued'` to `'dispatched'`.
- **Consequence**: The prompt remained in SQLite `active_prompts` with `status = 'queued'`. When the instance later became idle, the background scheduler re-fetched the same row and attempted a duplicate dispatch, or left the row lingering, causing the UI to show stale queued counts.

### Pillar D: Transcript Turn Inversion (`active_turn_idx` Hard Fallback to Turn 0)
In `src-tauri/src/modules/repo_db/transcript.rs` lines 1017–1035:
```rust
let active_turn_idx: Option<usize> = if is_conv_running {
    if let Some(last_model_idx) = last_model_step_idx {
        // ...
    } else {
        Some(0)
    }
};
```
- **Flaw**: When inspecting a running conversation, if `last_model_step_idx` was un-indexed or `0`, `active_turn_idx` defaulted to `Some(0)` (the very first turn, T1).
- **Consequence**: The status matching loop assigned:
  - `idx == 0` (T1) $\to$ `RUNNING`
  - `idx > 0` (T2 through T11) $\to$ `QUEUED`
  Historical completed turns that happened hours or days ago were all marked `QUEUED`. In `src/components/instances/PromptTreeView/DetailHeader.tsx`, `queuedPromptCount` counted 10 queued turns, displaying the erroneous banner `10 prompt(s) enqueued in FIFO queue for this conversation` and `Enqueue 10 Q`.

---

## 2. Solution Architecture & Changes

### Fix 1: Prioritize Live CDP Dispatch over Idle Check in Queue Scheduler
- In `src-tauri/src/modules/repo_db/queue.rs`:
  - Query `is_instance_process_running_smart(&inst_id)` *first*.
  - If the IDE process is alive, fetch the oldest FIFO queued prompt (`ORDER BY created_at ASC, id ASC`) and immediately dispatch it via CDP with `cancel_running_first: false`.
  - When CDP succeeds, transition prompt status to `'dispatched'` and update SQLite and memory maps.
  - Only when the IDE is *not* running or CDP is unavailable does `is_prompt_running_for_project` gate execution (preventing concurrent headless `agy` CLI spawns on the same workspace).

### Fix 2: Update Status to Dispatched upon Immediate CDP Enqueue
- In `src-tauri/src/modules/repo_db/agy.rs` (`enqueue_prompt_for_instance`):
  - When `submit_prompt_via_cdp_for_instance_with_mode` returns `Ok(())`:
    - Mark `active_prompt.status = "dispatched"`.
    - Persist the updated state to SQLite via `save_or_requeue_prompt(&active_prompt)`.
    - Invalidate prompt tree cache (`invalidate_prompt_tree_cache(Some(&canonical_inst))`).
    - The prompt enters the IDE turn queue immediately, and SQLite reflects genuine dispatch.

### Fix 3: Bound Gate 4 Summary Activity Window
- In `src-tauri/src/modules/repo_db/liveness.rs`:
  - Reduce `max_allowed_elapsed` for `not_fully_idle > 0` from 600s down to 90s.
  - If no transcript or summary writes have occurred within 90 seconds, do not treat the session as actively running, eliminating zombie busy locks.

### Fix 4: Correct Transcript Active Turn Resolution
- In `src-tauri/src/modules/repo_db/transcript.rs`:
  - When `is_conv_running` is true:
    - If `last_model_is_done` and all `raw_turns` have completed model responses (`matched_idx + 1 >= n`), set `active_turn_idx = None`. No past turns are marked `RUNNING` or `QUEUED`.
    - If no model step is recorded yet, set `active_turn_idx = Some(n - 1)` (the latest turn), *never* `Some(0)`.
    - Historical turns before `active_turn_idx` are strictly marked `COMPLETED` (`is_running = false, is_queued = false`).
    - Only genuine in-flight turns are marked `RUNNING`, and only genuinely pending un-responded turns are marked `QUEUED`.

### Fix 5: Frontend Cache Invalidation & Direct Reload
- In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
  - After `enqueuePrompt` returns, trigger `invalidate_prompt_tree_cache` and `loadTree(false, true)`.
  - Re-fetch tree data so the UI accurately displays the dispatched state.

---

## 4. Verification Matrix & E2E Validation

| Test Case | Condition | Expected Result | Verified Status |
| :--- | :--- | :--- | :--- |
| **TC-CDP-01** | IDE running active prompt, user clicks Enqueue | Prompt submits via CDP with `cancel_running_first: false`, appears in IDE turn queue immediately | ✅ Implemented |
| **TC-CDP-02** | User triggers Rerun x2 | Both nudge prompts submit sequentially via CDP and stage as enqueued turns in IDE | ✅ Implemented |
| **TC-STATUS-01** | Successful CDP enqueue injection | SQLite row transitions from `queued` to `dispatched`; no double-dispatch occurs | ✅ Implemented |
| **TC-TRANS-01** | Multi-turn conversation with completed turns | Historical turns T1..T10 are marked `COMPLETED`; `queuedPromptCount` evaluates to 0 (no false +10 badge) | ✅ Implemented |
| **TC-GATE4-01** | Inactive turn with stale `not_fully_idle = 1` (>90s) | Gate 4 evaluates to false; scheduler does not deadlock | ✅ Implemented |
| **TC-PREFLIGHT-01** | Cargo clippy, Cargo fmt, Vite build, Vitest suite | All gates pass with 0 errors | ✅ Validated |
