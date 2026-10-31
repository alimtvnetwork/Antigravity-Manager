# Database Management Rules & Architecture Checklist

**Version:** 1.0.0  
**Mandate:** Non-negotiable database rules across Antigravity-Manager.  
**Location:** `.ai-memory/checklists/database-management-rules.md`  

---

## 1. Split Database Architecture (Split-DB)
- [ ] **Domain-Specific Isolation**: Never consolidate unrelated domain models into a single monolithic SQLite database. Maintain isolated SQLite databases per domain (`errors.db`, `repo-prompts.db`, `prompt-backups.db`, `task-history.db`, `email-vault.db`).
- [ ] **WAL Mode Standard**: All split SQLite databases must enable Write-Ahead Logging (`PRAGMA journal_mode = WAL;`) and synchronous normal mode (`PRAGMA synchronous = NORMAL;`) with a 5000ms busy timeout (`PRAGMA busy_timeout = 5000;`).
- [ ] **Kebab-Case File Naming**: All `.db` files must strictly use kebab-case file names with `.db` extension (e.g. `repo-prompts.db`, `prompt-backups.db`, `task-history.db`). PascalCase or snake_case file names are strictly forbidden.

---

## 2. PascalCase Schema & Column Naming Discipline
- [ ] **Singular PascalCase Table Names**: Tables must strictly use singular PascalCase names:
  - `ErrorLog` (NOT `errors` or `error_logs`)
  - `ActivePrompt` (NOT `active_prompts`)
  - `RunningProject` (NOT `running_projects`)
  - `PromptBackup` (NOT `prompt_backups`)
  - `TaskHistory` (NOT `task_history`)
  - `EmailAccount` (NOT `email_accounts`)
- [ ] **PascalCase Column Names**: Every column in every SQLite table must strictly use PascalCase (e.g. `ErrorLogId`, `PromptContent`, `CreatedAt`, `UpdatedAt`, `InstanceId`).
- [ ] **Primary Key Convention**: Primary keys must strictly follow the `{TableName}Id` convention (e.g. `ErrorLogId`, `ActivePromptId`, `PromptBackupId`).
- [ ] **Foreign Key Equality**: Foreign key column names must strictly mirror the referenced primary key (`RunningProjectId` references `RunningProject.RunningProjectId`).

---

## 3. Affirmative Boolean Column Standards
- [ ] **Positive Prefixes Only**: Boolean columns must always begin with positive prefixes `Is` or `Has` (`IsActive`, `HasImages`, `IsRestored`, `IsRunning`).
- [ ] **Total Ban on Negatives**: Column names containing negative particles (`IsNotActive`, `HasNoImages`, `IsDisabled`) are strictly forbidden. Invert state in the application layer if needed.
- [ ] **Implicit Checking**: Never evaluate booleans explicitly against `== 1` or `== true` in application logic; use implicit checks (`if prompt.has_images`).

---

## 4. ORM & Generator-First Communication
- [ ] **Total Ban on Raw SQL in Business Layer**: Business logic, commands, and service modules must never execute raw `SELECT`, `INSERT`, `UPDATE`, `DELETE` SQL strings directly.
- [ ] **Type-Safe ORM Abstractions**: Database queries must flow through generated model structs and type-safe query builders (`db_orm` module).
- [ ] **Automated Generator Script**: All model structs, table definitions, and CRUD accessors must be maintained or verified via the Python generator script (`03-ai-scripts/51-db-orm-generator.py`).
- [ ] **Pre-Defined Database Views for Joins**: Complex multi-table queries must be pre-defined as SQLite views (`CREATE VIEW Vw... AS SELECT ...`) with `Vw` prefix. Application logic queries views as flat entities without embedding raw JOINs.

---

## 5. Migration & Zero-Data-Loss Invariant
- [ ] **Pre-Migration Single-File Backup**: Before executing any schema migration or database upgrade, all existing databases must be cleanly exported into a unified backup snapshot (`pre_migration_v4_snapshot.json` or `.sqlite`).
- [ ] **Zero Account Loss Guarantee**: Accounts (`accounts.json`) and associated encryption credentials must be verified with cryptographic integrity checks before and after any database operation.
- [ ] **Version Number Tracking**: Every database must maintain a `SchemaVersion` table tracking version number, migration timestamp, and applied migration checksum.
- [ ] **Fresh Start with Re-Import**: The initial PascalCase migration creates fresh databases and re-imports historical data through field-mapping adapters. Subsequent schema changes must use forward migrations.
