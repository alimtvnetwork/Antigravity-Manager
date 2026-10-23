# Subtask 03: FIFO Dead Letter Queue Spillover Guard (`Imp-687`)
Status: IN_PROGRESS
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."

Implement `src-tauri/src/modules/db_orm/fifo_dead_letter_queue_spillover_guard.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_dead_letter_queue_spillover_guard.rs`.

## Requirements
- Move persistently failing prompts (exceeding retry attempts limit) into dead-letter quarantine table.
- Preserve original `created_at` timestamp and chronological lineage for post-mortem analysis.
- Reject un-ordered LIFO spillover queries.
- Comprehensive unit tests covering spillover triggering, timestamp preservation, and quarantine state.
