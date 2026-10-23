# Subtask 03: FIFO Dead Letter Queue Spillover Guard (`Imp-687`)
Status: COMPLETED
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."

Implemented `src-tauri/src/modules/db_orm/fifo_dead_letter_queue_spillover_guard.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_dead_letter_queue_spillover_guard.rs`.
Registered in `src-tauri/src/modules/db_orm/mod.rs`.

## Verification Evidence
- Defined `DeadLetterReason` (`MaxRetriesExceeded`, `MalformedPayload`, `PermanentUpstreamRejection`).
- Implemented `evaluate_dead_letter_spillover` moving failed prompts to quarantine while strictly preserving their original `created_at` timestamp.
- Implemented `validate_dead_letter_sql` ensuring dead-letter queries enforce `ORDER BY original_created_at ASC, id ASC` and rejecting LIFO clauses.
- Added comprehensive unit tests covering retry thresholding, chronological preservation, and SQL query validation.
- Deferred E2E command documented in `02-spec/21-app/212-implementing-next-five-tasks/e2e_master_plan.md`.
