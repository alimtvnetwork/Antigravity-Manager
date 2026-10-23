# Subtask 03: FIFO Prompt Dispatch Batch Coordinator (`Imp-697`)
Sequence: 214
Step: 283
Status: IN_PROGRESS
Module: `src-tauri/src/modules/db_orm/fifo_prompt_dispatch_batch_coordinator.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_fifo_prompt_dispatch_batch_coordinator.rs`

## Description
Enforces AGENTS.md rule: "All prompt restore, dispatch, and queue processing queries across SQLite databases (active_prompts, prompt_backups, repo_prompts.db) MUST enforce strict First-In-First-Out ordering using ORDER BY created_at ASC, id ASC. Inverted LIFO queries (ORDER BY updated_at DESC) are strictly forbidden in dispatch logic." Coordinates atomic multi-row prompt dispatch batches across SQLite queues.
