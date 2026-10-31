# Subtask 04: FIFO Multi-Table Restore Transaction Coordinator (`Imp-668`)
Status: COMPLETED
Parent Plan: 208-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."

Implement `src-tauri/src/modules/db_orm/fifo_restore_transaction_coordinator.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_restore_transaction_coordinator.rs`.

## Requirements
- Coordinate prompt restoration across `active_prompts`, `prompt_backups`, and `repo_prompts.db`.
- Enforce strict FIFO ordering `ORDER BY created_at ASC, id ASC` across all coordinated queries.
- Roll back entire restore transaction if any stage attempts out-of-order restoration or inverted ordering.
- Provide structured report tracking restored items count, table sequence, and transaction commit status.
