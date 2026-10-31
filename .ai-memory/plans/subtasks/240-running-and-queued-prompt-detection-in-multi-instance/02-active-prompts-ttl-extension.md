# Subtask 02: Active Prompts In-Flight TTL Extension & Status Persistence in tree_phases.rs & agy.rs

## Slug
`running-and-queued-prompt-detection-in-multi-instance - Task 02`

## Goal
Prevent premature transition of dispatched or queued prompts to `completed` in `active_prompts`.

## Changes
- In `src-tauri/src/modules/repo_db/tree_phases.rs` (`transition_stale_inflight_prompts`):
  - Extend the timeout for `dispatched` prompts from 120s to 3600s (1 hour).
  - Exempt prompts whose associated conversation ID has active running status.
- In `src-tauri/src/modules/repo_db/agy.rs` (`enqueue_prompt_for_instance`):
  - Preserve `status = "queued"` or mark `"enqueued_ide"` so that the prompt continues to be recognized as queued in UI and SQLite.
