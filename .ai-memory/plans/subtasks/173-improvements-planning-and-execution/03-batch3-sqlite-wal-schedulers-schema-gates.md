# Subtask 03: Batch 3 (Imp 493–498: Split-SQLite Maintenance, WAL Checkpoint Schedulers & Schema Gates)

- **Parent Plan**: `.ai-memory/plans/pending/173-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/173-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-493: Periodic WAL Truncate Checkpoint Scheduler** (`src-tauri/src/modules/db_orm/wal_truncate_scheduler.rs`)
   - Triggers periodic `PRAGMA wal_checkpoint(TRUNCATE)` when the WAL file page count exceeds safety thresholds (500 pages).
2. **Imp-494: Connection Pool Checkout Rate Limiter** (`src-tauri/src/modules/db_orm/connection_checkout_limiter.rs`)
   - Limits concurrent connection checkouts across background tasks to protect user-facing interactive queries.
3. **Imp-495: Split Database Schema User Version Gate** (`src-tauri/src/modules/db_orm/schema_version_checker.rs`)
   - Compares `PRAGMA user_version` against compiled application migration numbers before initializing SQLite connection pools.
4. **Imp-496: Table Index Cardinality Analyzer** (`src-tauri/src/modules/db_orm/index_cardinality_analyzer.rs`)
   - Analyzes column distinct value counts to identify low-selectivity indexes that cause query optimizer regressions.
5. **Imp-497: Temporary Storage Memory Guard** (`src-tauri/src/modules/db_orm/temp_store_memory_guard.rs`)
   - Enforces `PRAGMA temp_store = MEMORY;` across split database connections to keep transient tables in RAM.
6. **Imp-498: Table Page Space Allocation Counter** (`src-tauri/src/modules/db_orm/table_page_counter.rs`)
   - Inspects page allocations per table across split databases, detecting unexpected table bloat before it impacts performance.
