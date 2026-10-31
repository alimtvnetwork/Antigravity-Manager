# Subtask 08: CLI Integration (`agm db backup`, `agm db restore`, `agm db status`)

**Task ID:** `Task-08`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 01  
**Status:** PENDING  

## Objective
Implement CLI commands under `agm db` (`src-tauri/src/bin/agm/db_cmds.rs`) allowing developers and automated deployment scripts to inspect database status, trigger backup snapshots, restore from backup, and verify schema versions.

## Target Files
- `src-tauri/src/bin/agm/db_cmds.rs`
- `src-tauri/src/bin/agm/main.rs` (register `db` subcommand)

## Invariants
- `agm db status`: prints table list, row counts, and SchemaVersion.
- `agm db backup`: creates single-file snapshot.
- `agm db restore <path>`: restores snapshot into fresh PascalCase schema.
- `agm db verify`: asserts zero data loss and account hash match.
