# Completed Plan: 152-ide-instance-prompt-cache-and-dispatch-fix

## User Request (Verbatim)
```text
# High Priority Instruction

Hi there. Look, the sending prompt from the instance section to the IDE does not work. Find the root cause of it, why we cannot send the prompt and why we cannot enqueue the prompt. In both cases, it has no effect so far. Every time it tries to open or reopen the IDE instance, which is very wrong. If the instance is already open, you have to check in the process if this instance is already running. If the instance is already running, you need to cache it so that you can understand if the instance. That means you have to have the smart logic. When we send it or queue the prompt, first, you have to check how many instances are running for that Antigravity Tools Manager. You need to cache those as well. Let's say you find that it is running in the cache, but then again, you try to send and see this PID is closed. Then you have to rerun it and see if it is running again. If it is not running, then and only then you have to reopen and send the prompt. It is not happening right now like this. It is not working at all. You need to work on it, find the root cause, do the end-to-end testing, do things from CLI as well. Currently, the view is nice. I appreciate that, but there are too many tags, which is not necessary in the prompt section. Too many target bracket item, which can be reduced. And too many running items, which is not running, so you need to fix that as well. I hope you respect this and try to fix all this stuff and make a minor bump and release again.
```

## Specifications Reference
- Architecture Spec: `02-spec/21-app/152-ide-instance-prompt-cache-and-dispatch-fix/01-architecture-spec.md`
- Component & UI Spec: `02-spec/21-app/152-ide-instance-prompt-cache-and-dispatch-fix/02-component-and-ui-spec.md`

## Root Cause Analysis Summary
1. **Destructive Cold Relaunch & Reopen Loops**:
   - `ensure_instance_running_smart` lacked upfront startup process cache warming, causing cold misses on application startup.
   - When Electron bootstrap wrapper PIDs exited, naive single-PID checks declared instances dead, ignoring surviving child worker PIDs in `entry.pids`.
   - On offline assessment, cold launch invoked `close_instance` (`taskkill /F /PID`), killing active IDE instances and triggering reopen loops.
   - Client-side `PromptTreeViewModal.tsx` fired a second immediate `focusInstanceWorkspace` call, racing with backend `sendPromptNow`.
2. **Latent FIFO Queue Dispatching & Inconsistent Resumes**:
   - Background prompt queue scheduler ran only every 600 seconds (10 minutes) without adaptive acceleration when prompts were enqueued.
   - Duplicate ad-hoc inline JSON builders omitted `conversation_id`, `session_id`, and normalized timestamps in `.antigravity_resume_task.json`.
3. **Ghost Running Items & Visual UI Noise**:
   - In `PromptTreeViewModal.tsx`, project `runningCount` counted conversations without filtering out ghost conversations (`isGhostConversation`) or empty 0-word placeholder records.
   - In `Instances.tsx`, `hasActiveTask` checked `node.is_running` without verifying genuine, non-ghost, non-empty active conversations.
   - In `repo_db.rs`, startup purge failed with SQLite `FOREIGN KEY constraint failed` when deleting corrupted `running_projects` referenced by `active_prompts`, preventing stale `is_running = 1` resets.
   - Project listing tables lacked deduplication, repeating workspaces across historical database sessions.
   - Unnecessary UI tags (`{totalProjectPrompts} prompts`, `{conv.step_count} stp`, and brackets) cluttered sidebar layouts.

## Completed Tasks & Verified Deliverables
1. **Task-01: Smart Process Caching, Closed PID Recovery & Zero-Relaunch Guarantee**
   - Implemented `warm_up_smart_process_cache()` in `src-tauri/src/modules/instance.rs` and wired it into desktop/headless setup in `src-tauri/src/lib.rs`.
   - Upgraded `check_cached_pid_alive` with multi-PID vitality scanning and child PID promotion when primary bootstrap PIDs exit.
   - Added Tier 2 fresh OS scan fallback in `is_instance_process_running_smart` before concluding offline.
   - Enforced Reopen Guard in `ensure_instance_running_smart` and `launch_instance_inner_with_extra_workspaces`: existing instances are strictly focused, never killed.
2. **Task-02: Adaptive FIFO Enqueued Prompt Background Ticker & Canonical Resume Generation**
   - Upgraded `start_prompt_queue_scheduler` in `src-tauri/src/modules/scheduler.rs` to an adaptive ticker: 5s when prompts are queued, 30s idle backoff.
   - Standardized all `.antigravity_resume_task.json` file writes in `src-tauri/src/modules/repo_db.rs` through canonical `resume_task_document(...)`.
   - Enforced strict FIFO ordering (`ORDER BY created_at ASC, id ASC`) across all queue queries.
3. **Task-03: UI Tag Compaction & Target Bracket Stripping**
   - Removed visible `{totalProjectPrompts} prompts` pill from sidebar project rows; moved count to hover tooltip.
   - Converted project action buttons (Refresh, Pin, Archive) to hover-revealed (`opacity-0 group-hover:opacity-100`).
   - Removed `{conv.step_count} stp` pills from primary conversation rows and subagent child rows; moved step counts to tooltips.
   - Normalized sequence codes to clean `#P001` and `C001` format without square brackets.
   - Removed redundant client-side `focusInstanceWorkspace` race call from `handleResendPrompt`.
4. **Task-04: Running Prompt State Reconciliation & Startup Purge FK Fix**
   - Fixed foreign key constraint in `purge_corrupted_running_projects` by deleting referencing `active_prompts` before `running_projects`.
   - Tightened `memory_prompts_map` TTL and tree conversation recency windows from 600s to 45s.
   - Integrated `is_instance_process_running_smart` into Gate 0 process liveness checks in `is_prompt_running_for_project` and `get_live_project_execution_info`.
   - Deduplicated workspace project rows by `(norm_inst, clean_path)` in `get_live_project_execution_info`.
   - Filtered ghost and empty conversations in `PromptTreeViewModal.tsx` and `Instances.tsx`.
5. **Task-05: CLI Parity, Pre-flight Verification & Minor Bump**
   - Verified `cargo fmt -- --check` (exit code 0).
   - Verified `npm run build` (exit code 0).
   - Tested CLI commands (`agm status`, `agm help`).
