# Subtask 07: Zero Account Loss Safeguard & Accounts Verification

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** PENDING  
**Target Files:** `src-tauri/src/modules/db_migration/account_guard.rs`

## Objective
Guarantee zero account loss during database migration and upgrades:
1. Verify `accounts.json` path and compute SHA-256 hash.
2. Read and parse accounts list, recording email addresses and tokens count.
3. Compare post-migration state with pre-migration snapshot to ensure 0 lost accounts.
4. Block migration if any accounts discrepancy is detected.

## Acceptance Criteria
- Unit tests verify integrity checking with mock `accounts.json`.
- Aborts safely with rollback if `accounts.json` is modified or missing during migration.
