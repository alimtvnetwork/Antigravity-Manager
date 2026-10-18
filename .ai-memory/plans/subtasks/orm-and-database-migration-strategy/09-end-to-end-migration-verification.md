# Subtask 09: End-to-End Migration & Verification Testing

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** PENDING  
**Target Files:** `src-tauri/tests/test_migration_e2e.rs`

## Objective
Author and run comprehensive integration test simulating legacy database state, exporting snapshot, initializing fresh PascalCase databases, re-importing data, and verifying zero loss.
1. Populate legacy `errors.db`, `repo_prompts.db`, `backup-prompts.db` with legacy snake_case schemas and sample rows.
2. Create dummy `accounts.json` with multiple accounts.
3. Trigger migration pipeline.
4. Verify all records exist in new PascalCase tables and views.
5. Verify `accounts.json` is completely unchanged and verified.

## Acceptance Criteria
- Integration test passes green via `cargo test --test test_migration_e2e`.
- Verifies view queries (`VwRecentErrors`, `VwActivePromptsWithProjects`, `VwUnrestoredBackups`).
