# Subtask 03: Batch 3 (Imp 373–378: Split-SQLite Concurrency & Diagnostics)

- **Parent Plan**: `.ai-memory/plans/pending/169-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/169-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-373: SQLite Lock Escalation Guard** (`src-tauri/src/modules/db_orm/lock_escalation_guard.rs`)
   - Intercepts read-to-write transaction escalation, applying bounded exponential backoff to prevent SQLITE_BUSY deadlocks.
2. **Imp-374: Table Row Size Profiler** (`src-tauri/src/modules/db_orm/table_row_size_profiler.rs`)
   - Scans and samples row byte sizes in `repo_prompts.db` and `prompt_backups.db` to identify oversized payloads.
3. **Imp-375: Split Database Orphan Checker** (`src-tauri/src/modules/db_orm/split_db_orphan_checker.rs`)
   - Validates logical cross-database foreign keys between task history entries and conversation prompt records.
4. **Imp-376: SQLite Page Cache Sizing Adjuster** (`src-tauri/src/modules/db_orm/page_cache_adjuster.rs`)
   - Evaluates system memory and dynamically tunes `PRAGMA cache_size` to optimize memory footprint vs query performance.
5. **Imp-377: Split Database Pragma Diagnostic Reporter** (`src-tauri/src/modules/db_orm/pragma_diagnostic_reporter.rs`)
   - Inspects and compiles comprehensive PRAGMA configuration reports (`wal_autocheckpoint`, `synchronous`, `cache_size`, `busy_timeout`).
6. **Imp-378: Stale WAL File Recovery Sentinel** (`src-tauri/src/modules/db_orm/stale_wal_cleanup_sentinel.rs`)
   - Scans for dangling `-wal` and `-shm` files on startup after abnormal app termination and safely runs WAL checkpoint recovery.
