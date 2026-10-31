# Architecture Specification: ORM and Split Database Engine

**Specification File:** `02-spec/21-app/orm-and-database-migration-strategy/01-architecture-spec.md`  
**Parent Document:** [`readme.md`](readme.md)  
**Status:** DRAFT / IN PROGRESS  

---

## 1. Architectural Topology

The Antigravity-Manager database architecture transitions from ad-hoc `rusqlite::Connection::execute` calls to a 3-tier decoupled ORM and Split-DB system:

```
+----------------------------------------------------------------------------------------------------+
|                                    Application & Business Layer                                    |
|   (modules/repo_db, modules/backup_prompts_db, modules/error_db, modules/task_history_db, CLI)     |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                 Rust ORM Abstraction Layer (db_orm)                                 |
|  +---------------------------+  +-------------------------------+  +----------------------------+  |
|  |     Generated Entities    |  |       Type-Safe Builder       |  |       Migration Engine     |  |
|  | - ActivePrompt            |  | - select::<T>()               |  | - pre_migration_export()   |  |
|  | - RunningProject          |  | - insert(entity)              |  | - fresh_schema_init()      |  |
|  | - ErrorLog                |  | - update(entity)              |  | - re_import_snapshot()     |  |
|  | - PromptBackup            |  | - delete_by_id(id)            |  | - verify_zero_loss()       |  |
|  | - VwActivePrompts         |  | - where_eq("Status", "done")  |  | - SchemaVersion tracker    |  |
|  +---------------------------+  +-------------------------------+  +----------------------------+  |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                               Split SQLite Databases (~/.antigravity_tools/)                       |
|   errors.db             repo-prompts.db             prompt-backups.db             task-history.db  |
|   (ErrorLog, VwErrors)  (ActivePrompt, RunningProj) (PromptBackup, VwBackups)     (TaskHistory)    |
|   WAL mode + 5s timeout WAL mode + 5s timeout       WAL mode + 5s timeout         WAL mode + 5s    |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Split-DB Domain Boundaries

| Database File | Bounded Context | Core Tables (Singular PascalCase) | Pre-defined Views (`Vw...`) |
|---|---|---|---|
| `errors.db` | System & Client Diagnostics | `ErrorLog`, `SchemaVersion` | `VwRecentErrors`, `VwErrorSummaryByCode` |
| `repo-prompts.db` | Live Prompts, Queue & Projects | `ActivePrompt`, `RunningProject`, `PromptSequence`, `SchemaVersion` | `VwActivePromptsWithProjects`, `VwQueuedPromptsFifo` |
| `prompt-backups.db` | Instance Switch Resumption | `PromptBackup`, `SchemaVersion` | `VwUnrestoredBackups`, `VwLatestBackupBatches` |
| `task-history.db` | Task Execution Audit Trail | `TaskHistory`, `SchemaVersion` | `VwRecentTaskHistory` |
| `email-vault.db` | Secure Credential Vault | `EmailAccount`, `EmailAuditLog`, `SchemaVersion` | `VwActiveEmailAccounts` |

---

## 3. Code Generator Pipeline (`51-db-orm-generator.py`)

Similar to the `gitmap` code generator standards and `03-ai-scripts/35-db-struct-enum-generator.py`:
- Input: Declarative schema definitions (tables, columns, types, nullability, views).
- Output: Rust source files in `src-tauri/src/modules/db_orm/generated/`:
  - `entities.rs`: Model structs with `#[serde(rename_all = "PascalCase")]`.
  - `tables.rs`: Type-safe table metadata, column name constants, and SQL DDL scripts.
  - `views.rs`: Readonly view models with mapping functions.
  - `queries.rs`: Strongly-typed CRUD methods (`find_by_id`, `find_all`, `insert`, `update`, `delete`).
