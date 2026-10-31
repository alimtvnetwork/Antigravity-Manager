# Subtask 04: FIFO Queue Batch Drain Guard (`Imp-673`)
Status: COMPLETED
Parent Plan: 209-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."

Implement `src-tauri/src/modules/db_orm/fifo_queue_batch_drain_guard.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_queue_batch_drain_guard.rs`.

## Requirements
- Support chunked batch draining of queued prompts across SQLite queue tables.
- Guarantee that every batch fetch query enforces `ORDER BY created_at ASC, id ASC`.
- Prevent sequence inversion and race conditions when multiple batch chunks are drained sequentially.
- Provide structured report tracking batch chunk counts, total drained records, and FIFO sequence monotonicity.
