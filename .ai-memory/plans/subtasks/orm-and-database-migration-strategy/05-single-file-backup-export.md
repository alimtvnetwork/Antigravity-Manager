# Subtask 05: Unified Single-File Backup & Pre-Migration Exporter

**Task ID:** `Task-05`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 02  
**Status:** PENDING  

## Objective
Implement `src-tauri/src/modules/db_migration/backup.rs` to snapshot all existing legacy SQLite databases and account states into a single unified JSON backup file (`pre_migration_v4_snapshot.json`) prior to database re-creation.

## Target Files
- `src-tauri/src/modules/db_migration/mod.rs`
- `src-tauri/src/modules/db_migration/backup.rs`
- `src-tauri/src/modules/db_migration/types.rs`

## Invariants
- Exports all existing rows across legacy tables (`errors`, `running_projects`, `active_prompts`, `backup_batches`, `prompt_backups`, etc.).
- Records account verification payload (account count, emails list, `accounts.json` SHA-256 hash).
- Atomic snapshot creation before any modification or table drop.
