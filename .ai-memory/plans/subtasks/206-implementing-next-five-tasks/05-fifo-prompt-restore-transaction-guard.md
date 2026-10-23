# Subtask 05: FIFO Prompt Restore Transaction Guard (`Imp-659`)
Status: IN_PROGRESS
Parent Plan: 206-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."

Implement `src-tauri/src/modules/db_orm/fifo_prompt_restore_transaction_guard.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_transaction_guard.rs`.

## Requirements
- Intercept batch queue dispatch and restore operations in SQLite transactions.
- Verify sorting clause strictly enforces `ORDER BY created_at ASC, id ASC`.
- Abort transaction if LIFO ordering or missing tie-breaker is detected.
- Provide structured report with transaction outcome and compliance metrics.
