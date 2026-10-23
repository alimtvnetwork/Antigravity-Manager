# Subtask 04: FIFO Prompt Restore Concurrency Limiter (`Imp-678`)
Status: IN_PROGRESS
Parent Plan: 210-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."

Implement `src-tauri/src/modules/db_orm/fifo_prompt_restore_concurrency_limiter.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_concurrency_limiter.rs`.

## Requirements
- Enforce strict concurrency limits on prompt restore tasks across SQLite queues.
- Acquire mutual exclusion leases per queue table (`active_prompts`, `prompt_backups`, `repo_prompts`) to prevent interleaving of sequence IDs.
- Reject concurrent conflicting restore operations with typed lease conflict errors.
- Provide structured report tracking lease holders, acquired permits, release timestamps, and concurrency state.
