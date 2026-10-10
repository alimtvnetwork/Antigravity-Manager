# Plan: 152-ide-instance-prompt-cache-and-dispatch-fix

## User Request (Verbatim)
```text
# High Priority Instruction

Hi there. Look, the sending prompt from the instance section to the IDE does not work. Find the root cause of it, why we cannot send the prompt and why we cannot enqueue the prompt. In both cases, it has no effect so far. Every time it tries to open or reopen the IDE instance, which is very wrong. If the instance is already open, you have to check in the process if this instance is already running. If the instance is already running, you need to cache it so that you can understand if the instance. That means you have to have the smart logic. When we send it or queue the prompt, first, you have to check how many instances are running for that Antigravity Tools Manager. You need to cache those as well. Let's say you find that it is running in the cache, but then again, you try to send and see this PID is closed. Then you have to rerun it and see if it is running again. If it is not running, then and only then you have to reopen and send the prompt. It is not happening right now like this. It is not working at all. You need to work on it, find the root cause, do the end-to-end testing, do things from CLI as well. Currently, the view is nice. I appreciate that, but there are too many tags, which is not necessary in the prompt section. Too many target bracket item, which can be reduced. And too many running items, which is not running, so you need to fix that as well. I hope you respect this and try to fix all this stuff and make a minor bump and release again.
```

## Root Cause Analysis Summary
1. **Prompt Dispatch / Enqueue Failure & Reopen Loop**:
   - `ensure_instance_running_smart` experienced false negatives because `INSTANCE_PROCESS_CACHE` started empty and Windows PID checks allocated new `System::new()`.
   - When a false negative occurred, AGM invoked `launch_instance_with_workspaces`, reopening the IDE every time instead of reusing the running process.
   - `PromptTreeViewModal.tsx` immediately called `focusInstanceWorkspace`, racing with the process scan lock and triggering duplicate launches.
   - `launch_instance_inner_with_extra_workspaces` invoked `close_instance` (`taskkill /F /PID`) when `!is_already_running`, terminating existing processes.
   - `enqueue_prompt_for_instance` saved prompts as `'queued'` in SQLite, but `check_and_dispatch_enqueued_prompts` was never ticked in any background loop.
   - Hand-crafted JSON was written instead of using `resume_task_document(...)`, dropping session and conversation context.
2. **Ghost Running Items & UI Clutter**:
   - `runningCount` in `PromptTreeViewModal.tsx` counted ghost and empty (0-word) conversations without filtering, showing glowing running badges on idle projects.
   - `repo_db.rs` `memory_prompts_map` had a 600-second stale TTL leak and loose prefix matching (`clean_target.starts_with(&format!("{}-", folder_name))`), causing idle projects to inherit running status.
   - Instance cards displayed pulsing green `Prompt` button (`hasActiveTask = true`) because of ghost running nodes.
   - Redundant tags: `{totalProjectPrompts} prompts` on every row, `{conv.step_count} stp` on every turn, and bracketed sequence codes (`[AGM:P001 | GM:#1]`).

## Completed Subtasks
- **Subtask 01 (`Task-01`)**: Backend Smart Process Cache, Closed PID Recovery & Prompt Dispatch Ticker
  - Implemented `warm_up_smart_process_cache()` in `src-tauri/src/modules/instance.rs`.
  - Implemented closed PID recovery and multi-PID vitality promotion in `src-tauri/src/modules/instance.rs`.
  - Enforced zero-relaunch guarantee in `ensure_instance_running_smart()`.
  - Protected running processes against aggressive kill in `launch_instance_inner_with_extra_workspaces()`.
  - Standardized resume payload via `resume_task_document(...)` in `src-tauri/src/modules/repo_db.rs`.
  - Reduced memory prompt TTL from 600s to 45s and eliminated loose prefix matching.
  - Enforced strict FIFO queue ordering (`ORDER BY created_at ASC, id ASC`).
  - Added adaptive background prompt queue scheduler in `src-tauri/src/modules/scheduler.rs`.
  - Registered startup warm-up hooks in `src-tauri/src/lib.rs`.
- **Subtask 02 (`Task-02`)**: UI Tag Compaction, Bracket Stripping & Ghost Running Elimination
  - Removed `{totalProjectPrompts} prompts` pill from sidebar project rows in `src/components/instances/PromptTreeViewModal.tsx`; moved to tooltip.
  - Made project action buttons (Refresh, Pin, Archive) hover-only.
  - Removed `{conv.step_count} stp` pill from sidebar conversation and subagent rows; moved to tooltip.
  - Normalized sequence codes to clean `#P001` and `C001` badges without target brackets.
  - Synchronized `runningCount` in `PromptTreeViewModal.tsx` to filter out ghost and empty (0-word) conversations.
  - Hardened `hasActiveTask` in `src/pages/Instances.tsx` to prevent ghost active task badges on idle instances.
  - Removed redundant client-side `focusInstanceWorkspace` call from `handleResendPrompt`.

## Specifications & Documentation
- Canonical Architecture Spec: `02-spec/21-app/152-ide-instance-prompt-cache-and-dispatch-fix/01-architecture-spec.md`
- Component and UI Spec: `02-spec/21-app/152-ide-instance-prompt-cache-and-dispatch-fix/02-component-and-ui-spec.md`
