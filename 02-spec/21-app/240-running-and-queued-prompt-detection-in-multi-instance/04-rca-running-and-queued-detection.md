# Root Cause Analysis: Running Status and Queued Prompt Detection in Multi-Instance

## 1. Problem Description
In Antigravity Manager, when inspecting an active conversation in "Project & Prompt Tree View" or instance cards (e.g. `C178 Gitmap`):
1. The conversation displayed as `IDLE now` with a green idle dot, and its current active turn was marked as completed (`TURN T5 Inspecting single prompt turn 5 of 5`), despite the Antigravity IDE actively generating and running tools (`Working`).
2. Multiple user prompts queued behind the active task (displayed in the Antigravity IDE under `Queued Messages (6) Sends after agent finishes working`) were completely absent from the Prompt Tree View and multi-instance UI, which reported 0 queued prompts.
3. The frontend error store repeatedly caught `E9001: Command invalidate_prompt_tree_cache not found` during cache invalidation calls.

## 2. Root Cause
1. **Liveness Priority Inversion in `tree_collect.rs`**:
   - `tree_collect.rs:155` set `let is_terminal = is_explicit_idle || inspection.is_terminal_done;`.
   - `inspect_conversation_transcript` was flagging `is_terminal_done = true` based on historical planner responses or step-level tool completions in the tail of `transcript.jsonl`.
   - In `tree_collect.rs:160`, `else if is_terminal { false }` unconditionally forced `is_conv_running = false`, overriding affirmative running markers in `conversation_summaries.db` (`status == "CASCADE_RUN_STATUS_RUNNING"` and `not_fully_idle = 1`).
   - Consequently, `extract_conversation_prompt_turns` was called with `is_conv_running = false`, marking every turn as `COMPLETED`.
2. **Aggressive Stale Purge in `tree_phases.rs`**:
   - `enqueue_prompt_for_instance` changed the prompt's status in `active_prompts` to `'dispatched'` upon successful CDP submission.
   - `transition_stale_inflight_prompts` had a 120-second threshold (`updated_at < now - 120`) that forcibly flipped `'dispatched'` prompts to `'completed'`.
   - Since tasks typically take more than 2 minutes, enqueued prompts were deleted from the active queue and hidden from the UI before execution.
3. **Missing IDE In-Memory Composer Harvesting**:
   - Prompts typed directly into the IDE composer while the agent is generating are held in internal memory and not written to `transcript.jsonl` until their turn starts. AGM lacked a live CDP harvester to read these queued items.
4. **Missing Tauri Command Registration**:
   - `invalidate_prompt_tree_cache` was called by `usePromptTreePromptActions.ts` but never registered in `src-tauri`.

## 3. Corrective Actions
1. **Affirmative Running Status Supremacy**:
   - In `tree_collect.rs`, if `cascade_status.is_running()` or `(not_fully_idle > 0 && is_recent)`, and any Antigravity IDE process is running on the system, deterministically set `is_conv_running = true`.
2. **Extend In-Flight TTL**:
   - In `tree_phases.rs`, extend the dispatched prompt TTL to 3600 seconds (1 hour) so queued prompts remain visible throughout long task runs.
3. **Live CDP Queued Prompt Harvester**:
   - In `cdp_onboarding.rs`, implement `extract_queued_composer_messages_for_instance` to query the live IDE DOM for `Queued Messages` and synthesize corresponding `QUEUED` turns in `tree_node_conv.rs`.
4. **Command Registration**:
   - Register `invalidate_prompt_tree_cache` in `commands/instance.rs` and `lib/run.rs`.

## 4. Prevention
- Ensure all affirmative database running signals take precedence over secondary transcript heuristic scans.
- Enforce that background housekeeping jobs never purge enqueued tasks without verifying completion against transcript logs.
