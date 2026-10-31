# Subtask 03: Batch 3 (Imp 313–318: Split-SQLite Vacuum Schedulers, Index Health & Transaction Guards)

- **Parent Plan**: `.ai-memory/plans/pending/167-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/167-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- Created `vacuum_scheduler.rs` executing incremental vacuums during idle states.
- Created `index_health_checker.rs` inspecting table indexes and query plan execution.
- Created `deadlock_detector.rs` monitoring busy handler retry cascades.
- Created `migration_version_stamp.rs` reading and stamping user_version pragmas.
- Created `page_cache_manager.rs` setting negative PRAGMA cache_size based on RAM.
- Created `audit_table_archiver.rs` discovering and purging archivable prompt records.
- Re-exported all items in `src-tauri/src/modules/db_orm/mod.rs`.

## Targeted Improvements
- **Imp-313: Incremental Auto-Vacuum Scheduler** (`src-tauri/src/modules/db_orm/vacuum_scheduler.rs`)
- **Imp-314: Database Index Health Checker** (`src-tauri/src/modules/db_orm/index_health_checker.rs`)
- **Imp-315: Concurrency Deadlock Detector** (`src-tauri/src/modules/db_orm/deadlock_detector.rs`)
- **Imp-316: Split Database Migration Version Stamp** (`src-tauri/src/modules/db_orm/migration_version_stamp.rs`)
- **Imp-317: SQLite Page Cache Memory Manager** (`src-tauri/src/modules/db_orm/page_cache_manager.rs`)
- **Imp-318: Prompt Audit Table Auto-Archive Sentinel** (`src-tauri/src/modules/db_orm/audit_table_archiver.rs`)
