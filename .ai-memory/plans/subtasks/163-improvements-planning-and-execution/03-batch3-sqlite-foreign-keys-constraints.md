# Subtask 03: Batch 3 (Imp 193–198: Split-SQLite Foreign Key & Constraint Integrity)

- **Parent Plan**: `.ai-memory/plans/pending/163-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/163-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- **Imp-193 (`foreign_keys.rs`)**: Implemented Split-DB foreign key constraint validator executing `PRAGMA foreign_key_check` with orphan diagnostics.
- **Imp-194 (`busy_retry.rs`)**: Implemented database busy timeout retry ladder with exponential backoff (`[10, 50, 200, 500]` ms).
- **Imp-195 (`vacuum_stepper.rs`)**: Implemented bounded auto-vacuum incremental stepper reclaiming freelist pages in 100-page chunks.
- **Imp-196 (`table_profiler.rs`)**: Implemented table row count and storage profiler discovering user tables and row volumes.
- **Imp-197 (`version_drift.rs`)**: Implemented schema version drift detector comparing local `SchemaVersion` against compile-time target.
- **Imp-198 (`read_sandbox.rs`)**: Implemented read-only SQL AST safety verification rejecting mutations in query explorers.
- Re-exported all submodules and types in `src-tauri/src/modules/db_orm/mod.rs`.

## Targeted Improvements
- **Imp-193: Split-DB Foreign Key Constraint Validator** (`src-tauri/src/modules/db_orm/foreign_keys.rs`)
- **Imp-194: Database Busy Timeout Retry Ladder** (`src-tauri/src/modules/db_orm/busy_retry.rs`)
- **Imp-195: SQLite Auto-Vacuum Incremental Stepper** (`src-tauri/src/modules/db_orm/vacuum_stepper.rs`)
- **Imp-196: Table Row Count & Size Profiler** (`src-tauri/src/modules/db_orm/table_profiler.rs`)
- **Imp-197: Split-DB Schema Version Drift Detector** (`src-tauri/src/modules/db_orm/version_drift.rs`)
- **Imp-198: Safe Query Read-Only Sandbox Guard** (`src-tauri/src/modules/db_orm/read_sandbox.rs`)
