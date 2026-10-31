# Subtask 06: Zero-Data-Loss Importer & Schema Version Tracking (`SchemaVersion`)

**Task ID:** `Task-06`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 03  
**Status:** PENDING  

## Objective
Implement `src-tauri/src/modules/db_migration/restore.rs` and `src-tauri/src/modules/db_migration/version.rs` to re-import historical snapshot data into the fresh PascalCase schemas via field adapters, track `SchemaVersion`, and provide fast-path boot bypass.

## Target Files
- `src-tauri/src/modules/db_migration/restore.rs`
- `src-tauri/src/modules/db_migration/version.rs`
- `src-tauri/src/modules/db_migration/adapters.rs`

## Invariants
- Field adapters map snake_case columns to PascalCase entity fields without data truncation.
- Seeding `SchemaVersion` table with `VersionNumber = 1`.
- Fast boot bypass checks `SchemaVersion >= 1` in <10ms to avoid re-running migration on every launch.
- Subsequent database alterations increment `VersionNumber` via forward migration scripts.
