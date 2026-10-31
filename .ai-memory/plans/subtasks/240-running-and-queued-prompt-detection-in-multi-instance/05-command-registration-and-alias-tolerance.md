# Subtask 05: Tauri Command invalidate_prompt_tree_cache Registration & Multi-Instance Alias Tolerance

## Slug
`running-and-queued-prompt-detection-in-multi-instance - Task 05`

## Goal
Register `invalidate_prompt_tree_cache` Tauri command and relax instance ID matching for multi-instance compatibility.

## Changes
- In `src-tauri/src/commands/instance.rs` and `src-tauri/src/lib/run.rs`:
  - Register `invalidate_prompt_tree_cache` pointing to `reset_prompt_tree_cache`.
- In `src/components/instances/PromptTreeView/usePromptTreeData.ts`:
  - Improve `isInstanceCompatible` to handle prefix variations and trimmed aliases.
