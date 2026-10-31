# Subtask 04: PascalCase Split-DB Schemas & Views Engine

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** PENDING  
**Target Files:** `src-tauri/src/modules/error_db/`, `repo_db/`, `backup_prompts_db/`

## Objective
Convert SQLite schema initializations in `error_db`, `repo_db`, and `backup_prompts_db` to PascalCase tables and views:
1. `errors.db`: `ErrorLog`, `SchemaVersion`, `VwRecentErrors`, `VwErrorSummaryByCode`.
2. `repo-prompts.db`: `RunningProject`, `ActivePrompt`, `ProjectSequence`, `ConversationSequence`, `FailedCommand`, `PromptTreeCache`, `MediaVault`, `SchemaVersion`, `VwActivePromptsWithProjects`, `VwQueuedPromptsFifo`.
3. `prompt-backups.db`: `BackupBatch`, `PromptBackup`, `GreenProject`, `SchemaVersion`, `VwUnrestoredBackups`, `VwLatestBackupBatches`.

## Acceptance Criteria
- All table and column names strictly PascalCase.
- All views successfully created and queryable.
- Backward compatibility wrapper checks if database is already migrated.
