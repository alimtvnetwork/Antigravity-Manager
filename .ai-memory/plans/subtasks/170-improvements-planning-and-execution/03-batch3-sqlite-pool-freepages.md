# Subtask 03: Batch 3 (Imp 403–408: Split-SQLite Maintenance, WAL Recovery & Forensics)

- **Parent Plan**: `.ai-memory/plans/pending/170-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/170-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-403: SQLite Connection Pool Starvation Guard** (`src-tauri/src/modules/db_orm/pool_starvation_guard.rs`)
   - Tracks checkout durations for SQLite connections, warning and recovering connections held longer than 5 seconds.
2. **Imp-404: Split Database Free Page Stepper** (`src-tauri/src/modules/db_orm/free_page_stepper.rs`)
   - Inspects `PRAGMA freelist_count` and executes stepped incremental vacuums to reclaim disk space gradually.
3. **Imp-405: Index Fragmentation Analyzer** (`src-tauri/src/modules/db_orm/index_fragmentation_analyzer.rs`)
   - Runs targeted index density checks on `active_prompts` and `task_history` tables to detect when `REINDEX` is required.
4. **Imp-406: Table Column Default Value Validator** (`src-tauri/src/modules/db_orm/column_default_validator.rs`)
   - Inspects `PRAGMA table_info` across all split databases, verifying default values exist on nullable timestamp/status columns.
5. **Imp-407: Transaction Commit Latency Observer** (`src-tauri/src/modules/db_orm/commit_latency_observer.rs`)
   - Measures commit durations across split SQLite transactions, alerting when WAL synchronization exceeds 50ms.
6. **Imp-408: Database File Header Checksum Guard** (`src-tauri/src/modules/db_orm/header_checksum_guard.rs`)
   - Validates SQLite 3 database header bytes, page size offsets, and write version counters before establishing new connection pools.
