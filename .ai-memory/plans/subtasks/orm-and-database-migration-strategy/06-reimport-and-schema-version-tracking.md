# Subtask 06: Zero-Data-Loss Importer & Schema Version Tracking

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** PENDING  
**Target Files:** `src-tauri/src/modules/db_migration/` (`restore.rs`, `version.rs`, `adapter.rs`)

## Objective
Implement fresh database initialization and re-import of historical snapshot data into the new PascalCase databases:
1. Initialize fresh split databases with PascalCase schemas and `SchemaVersion = 1`.
2. Map legacy snake_case columns to target PascalCase entities.
3. Validate row counts between snapshot and re-imported databases.
4. Set forward migration gate so subsequent startups skip legacy conversion.

## Acceptance Criteria
- 100% of rows from snapshot successfully imported into PascalCase tables.
- `SchemaVersion` recorded and verified on boot.
