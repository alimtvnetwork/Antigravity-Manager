# Subtask 03: FIFO Queue Compaction Guard (`Imp-692`)
Sequence: 213
Step: 278
Status: COMPLETED
Module: `src-tauri/src/modules/db_orm/fifo_queue_compaction_guard.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_fifo_queue_compaction_guard.rs`

## Description
Coordinates SQLite database queue compaction and tombstone cleanup (`active_prompts`, `prompt_backups`), ensuring that compaction passes NEVER re-order rows, strictly maintain monotonically increasing `created_at ASC, id ASC`, and safely purge acknowledged rows without creating sequence gaps or transaction deadlocks.

## Implemented Components
1. `CANONICAL_FIFO_ORDER_BY = "ORDER BY created_at ASC, id ASC"`: Mandatory canonical ORDER BY clause invariant.
2. `CompactionTable` enum: Identifies known prompt queue tables (`ActivePrompts`, `PromptBackups`, `RepoPrompts`) with table name resolution and `all()` slice helper.
3. `QueueRow` struct: Represents compaction candidate queue rows with `id`, `created_at`, `status`, `is_tombstone`, `active()`, `tombstone()`, and `is_purgeable()`.
4. `CompactionViolation` enum: Classifies ordering and query violations:
   - `EmptyQuery`
   - `MissingOrderByClause`
   - `InvertedLifoOrder`
   - `MissingIdTieBreaker`
   - `InvalidOrderByClause`
   - `TimestampRegression`
   - `IdTieBreakerInversion`
   - `DuplicateId`
   - `TargetTableMismatch`
5. `CompactionAuditReport` struct: Summarizes compaction audit outcome:
   - `is_order_preserved`: Bool flag indicating whether monotonic sequence was maintained.
   - `total_rows_before`: Number of rows prior to compaction.
   - `tombstones_purged`: Count of eliminated tombstoned/purged records.
   - `retained_rows_count`: Count of active retained rows.
   - `violations`: Detected violation messages.
   - `retained_rows`: Retained `QueueRow` records.
   - `is_clean()`: Convenience helper.
6. `validate_compaction_query_sql`: SQL parser and validator ensuring all compaction queries strictly enforce `ORDER BY created_at ASC, id ASC`, intercepting LIFO `DESC` clauses, missing tie-breakers, or invalid sort columns.
7. `execute_fifo_compaction`: Validates chronological sequence monotonicity across input rows, detects timestamp regressions, tie-breaker inversions, and duplicate IDs, and safely purges tombstones.
8. Query Builders & Enforcers:
   - `build_canonical_compaction_query(table: CompactionTable) -> String`
   - `build_canonical_tombstone_purge_query(table: CompactionTable) -> String`
   - `enforce_compaction_order_by(sql: &str) -> String`
9. Comprehensive Unit Test Suite in `src-tauri/src/modules/db_orm/tests_fifo_queue_compaction_guard.rs`.
10. Module registered and re-exported in `src-tauri/src/modules/db_orm/mod.rs`.
