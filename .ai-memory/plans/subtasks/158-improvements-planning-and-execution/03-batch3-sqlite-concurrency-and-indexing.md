# Subtask 03: Batch 3 Execution (Improvements 43–48: Split-SQLite Concurrency, Indexing & Auto-Vacuum)

## Improvements In Scope
- **Imp-43**: Dedicated Read-Only Connection Helper (`src-tauri/src/modules/db_orm/connection.rs`)
- **Imp-44**: Incremental Auto-Vacuum Configuration (`src-tauri/src/modules/db_orm/connection.rs`)
- **Imp-45**: Active Prompts FIFO Index Creation (`src-tauri/src/modules/repo_db/schema.rs`)
- **Imp-46**: Task History 500-Row Auto-Rotation (`src-tauri/src/modules/task_history_db.rs`)
- **Imp-47**: Prepared Statement Caching for CRUD (`src-tauri/src/modules/repo_db/prompts_crud.rs`)
- **Imp-48**: SQLite Startup Integrity Check Guard (`src-tauri/src/modules/db_orm/connection.rs`)

## Status
`[COMPLETED]`

## Verified Changes
- **Imp-43**: `open_split_db_readonly` helper created in `src-tauri/src/modules/db_orm/connection.rs` using `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX` and `PRAGMA query_only = ON;`.
- **Imp-44**: `PRAGMA auto_vacuum = INCREMENTAL;` configured in `src-tauri/src/modules/db_orm/connection.rs`.
- **Imp-45**: Index `idx_active_prompts_fifo` created on `(status, created_at, id)` in `src-tauri/src/modules/repo_db/schema.rs`.
- **Imp-46**: Task history 500-row partition cap (`SPLIT_ROW_CAP = 500`) verified in `src-tauri/src/modules/task_history_db/types.rs` and `tasks.rs`.
- **Imp-47**: Cached prepared statement execution implemented via `update_prompt_status` in `src-tauri/src/modules/repo_db/prompts_crud.rs` and re-exported in `repo_db.rs`.
- **Imp-48**: Fast non-blocking SQLite integrity check guard `verify_split_db_integrity` with `E2001` telemetry recording implemented in `src-tauri/src/modules/db_orm/connection.rs`.
