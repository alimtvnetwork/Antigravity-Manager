# Subtask 01: Send Preemption Task Cancellation & Enqueue Staging Inversion

- **Slug**: `send-cancel-preemption-and-enqueue-staging`
- **Component**: Prompt Dispatch Engine & Prompt Tree Action Capsule
- **Primary Files**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
  - `src-tauri/src/commands/instance.rs`
  - `src-tauri/src/modules/repo_db/agy.rs`
- **Status**: QUEUED

---

## 1. Problem Statement
The user explicitly stated:
> "The sending prompts to the screen from the prompt tree. That button works, but that button does not work as expected. The Enqueue button does not work. The Send button is doing Enqueue button's work. Send means whatever task is running at the moment, it would be cancel and the current prompt which I'm sending, that should be executed right now. That should be the Send button. The Enqueue should be currently what the Send button is doing. That should be the Enqueue button. Please correct it."

## 2. Technical Requirements
1. **Preemptive Task Cancellation on Send**:
   - When the user clicks "Send", AGM must check if a task/turn is currently running for the target instance.
   - If running, issue a cancellation signal / terminate the running execution process or task runner for that instance so that the active turn is cancelled immediately.
   - Clear any intermediate active lock or state, and dispatch the prompt directly so it executes right away without waiting behind previous tasks.
2. **Enqueue Staging**:
   - When the user clicks "Enqueue", AGM must stage the prompt into the FIFO queue (`active_prompts` table / queue worker) without interrupting the currently running task.
   - Fix `onEnqueue` / `handleEnqueue` in `DetailHeader.tsx` and ensure it visibly provides feedback ("Prompt enqueued to FIFO backlog!").
3. **Button Semantics & Visuals**:
   - `Send` button: Emerald theme icon, executes cancellation of current task + immediate execution.
   - `Enqueue` button: Amber theme icon (`ListPlus`), queues behind current task.

## 3. Verification Criteria
- Clicking Send while an instance has a running prompt cancels the running prompt and immediately dispatches the new prompt.
- Clicking Enqueue does not interrupt the running prompt and appends the new prompt to the queue list.
