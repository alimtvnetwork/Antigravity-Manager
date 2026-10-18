# Subtask 04: PascalCase Split-DB Schemas & Views Engine

**Task ID:** `Task-04`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 02  
**Status:** PENDING  

## Objective
Implement schema initialization DDLs with singular PascalCase table names, PascalCase columns, `{TableName}Id` primary keys, and pre-defined SQLite views (`Vw...`) for complex queries across `errors.db`, `repo-prompts.db`, `prompt-backups.db`, and `task-history.db`.

## Target Files
- `src-tauri/src/modules/db_orm/schemas.rs`
- `src-tauri/src/modules/db_orm/views.rs`

## Invariants
- All tables singular PascalCase (`ErrorLog`, `ActivePrompt`, `RunningProject`, `PromptBackup`, `BackupBatch`, `TaskHistory`).
- All columns PascalCase (`ErrorLogId`, `RunningProjectId`, `CreatedAt`).
- Views pre-defined: `VwRecentErrors`, `VwErrorSummaryByCode`, `VwActivePromptsWithProjects`, `VwQueuedPromptsFifo`, `VwUnrestoredBackups`, `VwLatestBackupBatches`, `VwRecentTaskHistory`.
- All boolean columns strictly affirmative (`IsRunning`, `HasImages`, `IsRestored`, `IsFullyRestored`).
