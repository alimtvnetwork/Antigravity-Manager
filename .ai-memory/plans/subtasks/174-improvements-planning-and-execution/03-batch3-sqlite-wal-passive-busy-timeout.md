# Subtask 03: Batch 3 (Imp 523–528: Split-SQLite Maintenance, WAL Checkpointing & Lock Recovery)

- **Parent Plan**: `.ai-memory/plans/pending/174-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/174-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-523: Passive WAL Checkpoint Scheduler** (`src-tauri/src/modules/db_orm/wal_passive_checkpoint_scheduler.rs`)
   - Executes passive non-blocking `PRAGMA wal_checkpoint(PASSIVE)` during idle periods to shrink log page counts without reader contention.
2. **Imp-524: Dynamic Busy Timeout Scaler** (`src-tauri/src/modules/db_orm/connection_busy_timeout_scaler.rs`)
   - Scales SQLite `busy_timeout` dynamically according to active connection count and read/write load.
3. **Imp-525: Schema Migration Append-Only Journal** (`src-tauri/src/modules/db_orm/schema_migration_journal.rs`)
   - Maintains an append-only migration journal in SQLite tracking schema transition history and applied timestamps.
4. **Imp-526: Split Table Column Statistics Collector** (`src-tauri/src/modules/db_orm/table_column_stats_collector.rs`)
   - Gathers row counts and approximate null-ratio statistics across split database tables for query tuning.
5. **Imp-527: Serializable Read-Uncommitted Guard** (`src-tauri/src/modules/db_orm/read_uncommitted_guard.rs`)
   - Configures `PRAGMA read_uncommitted = 0;` on transaction boundaries to ensure strict serializable isolation.
6. **Imp-528: Database Page Freelist Reclaim Analyzer** (`src-tauri/src/modules/db_orm/page_freelist_analyzer.rs`)
   - Measures `PRAGMA freelist_count;` and determines whether `incremental_vacuum` should reclaim unused pages.
