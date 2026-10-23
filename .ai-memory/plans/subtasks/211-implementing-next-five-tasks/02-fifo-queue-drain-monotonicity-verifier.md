# Subtask 02: FIFO Queue Drain Monotonicity Verifier (`Imp-681`)
Status: IN_PROGRESS
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."

Implement `src-tauri/src/modules/db_orm/fifo_queue_drain_monotonicity_verifier.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_queue_drain_monotonicity_verifier.rs`.

## Requirements
- Verify that successive drained batches from SQLite queue tables maintain strictly non-decreasing `(created_at, id)` sequence.
- Detect sequence inversion, timestamp regression, or duplicate IDs within and across batches.
- Provide structured report tracking processed batches, total items, and monotonicity compliance.
- Comprehensive unit test assertions for monotonic batches and inversion rejections.
