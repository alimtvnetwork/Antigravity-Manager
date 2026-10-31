# Subtask 03: FIFO Prompt Restore Cursor Stepper (`Imp-697`)
Sequence: 214
Step: 283
Status: COMPLETED
Module: `src-tauri/src/modules/db_orm/fifo_prompt_restore_cursor_stepper.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_cursor_stepper.rs`

## Description
Implements a stateful cursor-driven FIFO prompt restore query stepper (`WHERE (created_at > ?) OR (created_at = ? AND id > ?) ORDER BY created_at ASC, id ASC LIMIT ?`) that enables deterministic paginated drains of prompt queues without unbounded memory usage or sequence inversions.

## Implementation Details
- Enforces AGENTS.md rule: "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic."
- Defined `CANONICAL_FIFO_ORDER_BY: &str = "ORDER BY created_at ASC, id ASC"`.
- Defined `RestoreCursor` struct with `last_created_at: i64`, `last_id: i64`, and `satisfies_cursor` evaluation method.
- Defined `CursorStepConfig` struct with `batch_size: usize`, `table_name: String`.
- Defined `CursorQuery` struct with `sql: String`, `cursor: Option<RestoreCursor>`, `batch_size: usize`, and bind `params()`.
- Defined `CursorStepResult` struct with `retrieved_count: usize`, `next_cursor: Option<RestoreCursor>`, `is_exhausted: bool`, `is_order_preserved: bool`.
- Defined `CursorStepperViolation` enum with complete `Display` and `std::error::Error` implementations.
- Implemented `build_fifo_cursor_query(table: &str, cursor: Option<&RestoreCursor>, batch_size: usize) -> CursorQuery`.
- Implemented `validate_cursor_query_sql(sql: &str) -> Result<(), CursorStepperViolation>` checking for missing ORDER BY, inverted LIFO DESC, missing secondary tie-breaker `id ASC`, and disallowed primary sort columns.
- Implemented `verify_row_monotonicity(rows: &[(i64, i64)]) -> Result<(), CursorStepperViolation>` detecting duplicate primary keys, timestamp regressions, and ID tie-breaker inversions.
- Implemented `step_fifo_cursor(retrieved_rows: &[(i64, i64)], batch_size: usize) -> CursorStepResult`.
- Implemented `FifoPromptRestoreCursorStepper` stateful session manager for multi-step paginated queue drains.
- Registered module and test suite in `src-tauri/src/modules/db_orm/mod.rs`.
- Comprehensive companion unit test suite `tests_fifo_prompt_restore_cursor_stepper.rs` verifying 16 test cases.
