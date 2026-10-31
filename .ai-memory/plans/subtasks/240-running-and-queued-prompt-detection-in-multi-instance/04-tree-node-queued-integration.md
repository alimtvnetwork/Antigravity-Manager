# Subtask 04: Integration of Queued Messages into Tree Nodes & Multi-Instance UI

## Slug
`running-and-queued-prompt-detection-in-multi-instance - Task 04`

## Goal
Integrate live harvested queued prompts and SQLite queued prompts into conversation nodes in `tree_node_conv.rs` and `tree_collect.rs`.

## Changes
- In `src-tauri/src/modules/repo_db/tree_node_conv.rs`:
  - Merge live CDP queued messages into `target_conv.prompts` as `AgmPromptTurnNode` entries with `status: "QUEUED"`, `is_queued: true`, `is_running: false`.
  - Update `target_conv.is_queued = true` and `target_conv.status = if target_conv.is_running { "RUNNING" } else { "QUEUED" }`.
- Ensure multi-instance UI badges and Prompt Tree View FIFO queue banners display the correct queued prompt counts.
