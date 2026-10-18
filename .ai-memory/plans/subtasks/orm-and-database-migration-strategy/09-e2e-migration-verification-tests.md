# Subtask 09: End-to-End Migration & Verification Testing

**Task ID:** `Task-09`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 02  
**Status:** PENDING  

## Objective
Author and execute comprehensive automated integration tests in `src-tauri/tests/database_orm_migration_e2e.rs` to verify the complete migration pipeline, zero account loss, and zero data loss.

## Target Files
- `src-tauri/tests/database_orm_migration_e2e.rs`

## Invariants
- Tests seed legacy databases with test data and accounts.
- Executes full snapshot export, fresh PascalCase database initialization, and re-import.
- Asserts row counts match 100%.
- Asserts `accounts.json` SHA-256 hash matches 100%.
- Verifies views return expected aggregated/joined records.
- Verifies rollback triggers properly if account data is compromised.
