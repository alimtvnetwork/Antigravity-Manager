# Subtask 05: Single-File Pre-Migration Backup & Exporter

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** PENDING  
**Target Files:** `src-tauri/src/modules/db_migration/` (`mod.rs`, `backup.rs`, `export.rs`)

## Objective
Implement single-file backup export of all legacy SQLite databases into `~/.antigravity_tools/migration/pre_migration_v4_snapshot.json`.
1. Dump all rows from legacy tables (`errors`, `running_projects`, `active_prompts`, `backup_batches`, `prompt_backups`, `failed_commands`).
2. Include SHA256 checksum and account count of `accounts.json` for validation.
3. Save snapshot atomically with timestamped fallback.

## Acceptance Criteria
- Snapshot file passes JSON schema validation.
- Unit tests verify complete table dump from in-memory and disk SQLite databases.
