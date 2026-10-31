# Subtask 01: Affirmative Running Status Supremacy in tree_collect.rs & transcript.rs

## Slug
`running-and-queued-prompt-detection-in-multi-instance - Task 01`

## Goal
Ensure conversations marked `CASCADE_RUN_STATUS_RUNNING` or `not_fully_idle > 0` with live Antigravity processes are deterministically treated as `RUNNING` without false `IDLE` overrides from transcript heuristics.

## Changes
- Modify `src-tauri/src/modules/repo_db/tree_collect.rs`:
  - Evaluate affirmative database status: `(cascade_status.is_running() || not_fully_idle > 0) && is_recent`.
  - If affirmative and any Antigravity IDE process is running (`crate::modules::process::is_antigravity_running(None)`), set `is_conv_running = true`.
- Modify `src-tauri/src/modules/repo_db/transcript.rs`:
  - Ensure tool execution steps (`GENERIC`, `TOOL_OUTPUT`) and intermediate planner responses do not prematurely trigger terminal done.
