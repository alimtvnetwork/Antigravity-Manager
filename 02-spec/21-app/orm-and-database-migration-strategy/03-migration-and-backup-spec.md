# Migration and Backup Specification: Zero-Data-Loss & Single-File Snapshot Pipeline

**Specification File:** `02-spec/21-app/orm-and-database-migration-strategy/03-migration-and-backup-spec.md`  
**Parent Document:** [`readme.md`](readme.md)  
**Status:** DRAFT / IN PROGRESS  

---

## 1. Migration Overview

To safely transition from historical heterogeneous snake_case SQLite schemas to singular PascalCase schemas with zero data loss and zero account loss, Antigravity-Manager employs a **Single-File Snapshot & Fresh Initialization** pipeline:

```
[Legacy SQLite Databases]        [accounts.json & Credentials]
  errors.db, repo_prompts.db,             (Auth Vault)
  backup-prompts.db, etc.                      |
            |                                  |
            v                                  v
+--------------------------------------------------------------------+
|                Pre-Migration Single-File Backup Exporter           |
|                -> ~/.antigravity_tools/migration/                  |
|                -> pre_migration_v4_snapshot.json                   |
|                   (Contains full dump + accounts verification)      |
+--------------------------------------------------------------------+
            |
            v
+--------------------------------------------------------------------+
|               Fresh PascalCase Database Initialization             |
|          Create new .db files with strict PascalCase DDLs          |
|                  and SchemaVersion = 1 records                     |
+--------------------------------------------------------------------+
            |
            v
+--------------------------------------------------------------------+
|               Re-Import Engine with Field Adapters                 |
|       - Map old snake_case columns to PascalCase properties        |
|       - Transform legacy IDs to {TableName}Id                      |
|       - Bulk-insert via Rust ORM QueryBuilder                      |
+--------------------------------------------------------------------+
            |
            v
+--------------------------------------------------------------------+
|               Verification & Zero-Account-Loss Check               |
|       - Compare pre-export counts vs post-import counts            |
|       - Verify accounts.json hash & account count match 100%       |
+--------------------------------------------------------------------+
```

---

## 2. Pre-Migration Single-File Backup Snapshot (`pre_migration_v4_snapshot.json`)

The single-file snapshot captures all existing database tables, records, metadata, and the accounts inventory into a portable JSON structure:

```json
{
  "version": "1.0",
  "exportedAt": 1740000000,
  "agmVersion": "4.226.3",
  "accountsVerification": {
    "accountCount": 3,
    "emails": ["user1@gmail.com", "user2@gmail.com", "user3@gmail.com"],
    "sha256": "abcdef1234567890..."
  },
  "databases": {
    "errors": {
      "errors": [
        {
          "id": "err-001",
          "error_code": "E1001",
          "severity": "warn",
          "message": "Sample error",
          "stack_trace": "...",
          "source": "cli",
          "trigger_action": "start",
          "metadata_json": "{}",
          "created_at": 1739999900
        }
      ]
    },
    "repo_prompts": {
      "running_projects": [],
      "active_prompts": [],
      "agm_project_sequences": [],
      "agm_conversation_sequences": [],
      "failed_commands": []
    },
    "prompt_backups": {
      "backup_batches": [],
      "prompt_backups": [],
      "green_projects": []
    }
  }
}
```

---

## 3. Field-Mapping Adapters (Legacy Snake_Case -> PascalCase)

| Legacy Database | Legacy Table | Legacy Column | Target Table | Target Column | Transformation Rule |
|---|---|---|---|---|---|
| `errors.db` | `errors` | `id` | `ErrorLog` | `ErrorLogId` | Direct copy |
| `errors.db` | `errors` | `error_code` | `ErrorLog` | `ErrorCode` | Direct copy |
| `errors.db` | `errors` | `severity` | `ErrorLog` | `Severity` | Direct copy |
| `errors.db` | `errors` | `message` | `ErrorLog` | `Message` | Direct copy |
| `errors.db` | `errors` | `stack_trace` | `ErrorLog` | `StackTrace` | Direct copy |
| `errors.db` | `errors` | `source` | `ErrorLog` | `Source` | Direct copy |
| `errors.db` | `errors` | `trigger_action` | `ErrorLog` | `TriggerAction` | Direct copy |
| `errors.db` | `errors` | `metadata_json` | `ErrorLog` | `MetadataJson` | Direct copy |
| `errors.db` | `errors` | `created_at` | `ErrorLog` | `CreatedAt` | Direct copy |
| `repo_prompts.db` | `running_projects` | `id` | `RunningProject` | `RunningProjectId` | Direct copy |
| `repo_prompts.db` | `running_projects` | `is_running` | `RunningProject` | `IsRunning` | Ensure affirmative boolean (0/1) |
| `repo_prompts.db` | `active_prompts` | `id` | `ActivePrompt` | `ActivePromptId` | Direct copy |
| `repo_prompts.db` | `active_prompts` | `project_id` | `ActivePrompt` | `RunningProjectId` | Direct copy |
| `prompt-backups.db`| `backup_batches` | `id` | `BackupBatch` | `BackupBatchId` | Direct copy |
| `prompt-backups.db`| `backup_batches` | `is_fully_restored`| `BackupBatch` | `IsFullyRestored`| Ensure affirmative boolean (0/1) |
| `prompt-backups.db`| `prompt_backups` | `id` | `PromptBackup` | `PromptBackupId` | Direct copy |
| `prompt-backups.db`| `prompt_backups` | `backup_batch_id` | `PromptBackup` | `BackupBatchId` | Direct copy |
| `prompt-backups.db`| `prompt_backups` | `prompt_id` | `PromptBackup` | `OriginalPromptId` | Direct copy |
| `prompt-backups.db`| `prompt_backups` | `is_restored` | `PromptBackup` | `IsRestored` | Ensure affirmative boolean (0/1) |
| `prompt-backups.db`| `prompt_backups` | `has_images` | `PromptBackup` | `HasImages` | Ensure affirmative boolean (0/1) |

---

## 4. Schema Versioning & Forward Migration Invariant

1. Every split database contains a `SchemaVersion` table.
2. During the fresh initialization, `SchemaVersion` is seeded with:
   - `VersionNumber = 1`
   - `AppliedAt = Utc::now().timestamp()`
   - `Description = 'PascalCase Split-DB Schema with Views'`
   - `Checksum = '<sha256 of DDL>'`
3. When Antigravity-Manager boots:
   - It checks `SchemaVersion`. If `VersionNumber >= 1`, it recognizes the database as modernized and bypasses legacy conversion.
   - Any future modifications will apply incremental migrations `VersionNumber = 2, 3, ...` within explicit database transactions.

---

## 5. Zero Account Loss Verification Protocol

1. Before touching database files, the migration engine reads `accounts.json` from the Antigravity config directory.
2. It calculates the SHA-256 hash of `accounts.json` and records the total account count and email list in memory.
3. Database backup, fresh creation, and re-import run independently of `accounts.json`.
4. After all database operations complete:
   - The engine re-reads `accounts.json`.
   - It verifies that the file exists, the SHA-256 checksum matches, and the account count is completely unchanged.
   - If any discrepancy occurs, the migration is aborted and rolled back.

---

## 6. CLI Integration (`agm db ...`)

The native `agm` terminal binary exposes dedicated database operations:
- `agm db backup [--file <path>]`: Creates the unified pre-migration single-file backup.
- `agm db restore [--file <path>]`: Re-imports data from a snapshot into the PascalCase databases.
- `agm db status`: Displays schema versions, table names, row counts, and health status for all split databases.
- `agm db verify`: Runs integrity checks and account preservation verification.
