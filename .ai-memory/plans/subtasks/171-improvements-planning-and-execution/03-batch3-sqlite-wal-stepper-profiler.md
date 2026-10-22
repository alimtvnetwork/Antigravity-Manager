# Subtask 03: Batch 3 (Imp 433–438: Split-SQLite WAL Stepper, Profiling & Schema Integrity)

- **Parent Plan**: `.ai-memory/plans/pending/171-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/171-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-433: WAL Stepped Incremental Checkpointer** (`src-tauri/src/modules/db_orm/wal_stepped_checkpoint.rs`)
   - Executes incremental stepped WAL checkpoints (`PASSIVE` -> `RESTART` -> `TRUNCATE`) based on WAL file size and active reader count.
2. **Imp-434: SQLite Lock Contention & Busy Wait Profiler** (`src-tauri/src/modules/db_orm/lock_contention_profiler.rs`)
   - Measures lock contention events and busy-handler execution durations across split SQLite databases, emitting high-latency warnings.
3. **Imp-435: Column Type Affinity & Constraint Validator** (`src-tauri/src/modules/db_orm/column_type_affinity_validator.rs`)
   - Verifies SQLite column affinity definitions and primary key autoincrement constraints across split databases during startup.
4. **Imp-436: Query Explain Query Plan Cost Estimator** (`src-tauri/src/modules/db_orm/query_cost_estimator.rs`)
   - Inspects `EXPLAIN QUERY PLAN` output for table scans (`SCAN TABLE`) and flags missing index references on frequently queried columns.
5. **Imp-437: Temporary Database & Journal File Cleanup Sentinel** (`src-tauri/src/modules/db_orm/temp_db_cleanup_sentinel.rs`)
   - Sweeps orphaned temporary SQLite journal files (`-journal`, `-wal.tmp`) left behind by interrupted process crashes.
6. **Imp-438: Split Database Backup Snapshot Manifest Generator** (`src-tauri/src/modules/db_orm/database_snapshot_manifest.rs`)
   - Generates atomic metadata manifests (file sizes, page counts, SHA-256 hashes) for multi-database point-in-time backup sets.
