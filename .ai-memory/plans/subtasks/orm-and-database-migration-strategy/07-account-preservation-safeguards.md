# Subtask 07: Account Data Preservation & Integrity Safeguard (Zero Account Loss)

**Task ID:** `Task-07`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 03  
**Status:** PENDING  

## Objective
Implement safeguards in `src-tauri/src/modules/db_migration/account_guard.rs` ensuring zero accounts from `accounts.json` and `accounts/*.json` are lost or corrupted during the migration lifecycle.

## Target Files
- `src-tauri/src/modules/db_migration/account_guard.rs`
- `src-tauri/src/modules/db_migration/tests/account_tests.rs`

## Invariants
- Records pre-migration SHA-256 hash and account count of `accounts.json`.
- Account files are strictly read-only during migration and never deleted or modified.
- Verifies post-migration hash and record count match pre-migration 100%.
- If any discrepancy is detected, aborts migration and triggers rollback.
