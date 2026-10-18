---
name: orm-and-database-migration-strategy
description: Specialized skill for managing compile-time safe Diesel ORM schemas, Python ORM code generation (51-db-orm-generator.py), PascalCase Split-DB schemas, single-file backup and re-import migration engine, and Zero-Account-Loss safeguards in Antigravity-Manager.
---

# ORM and Database Migration Strategy (`orm-and-database-migration-strategy`)

This skill defines the compile-time type-safe SQLite ORM architecture, automated schema code generation, singular PascalCase naming, single-snapshot lossless migration, and account integrity safeguards for Antigravity-Manager (`agm`).

## 1. Architectural Core Principles

1. **Compile-Time Safety via Diesel ORM:**
   - Every SQLite database table is mapped declaratively via `diesel::table!` macros.
   - Rust entities implement Diesel's `Queryable`, `Selectable`, `Insertable`, and `AsChangeset` traits alongside standard serialization traits (`Debug`, `Clone`, `Serialize`, `Deserialize`).
   - Query construction leverages Diesel's type-checked DSL (`diesel::insert_into`, `.filter()`, `.load()`) preventing runtime SQL syntax errors and schema mismatch bugs.
   - Dual-mode queries supported via compile-time Diesel queries and fluent `QueryBuilder` helper.

2. **Automated Schema Code Generation (`51-db-orm-generator.py`):**
   - The central Python script `03-ai-scripts/51-db-orm-generator.py` maintains table and view metadata.
   - Automatically generates:
     - `src-tauri/src/modules/db_orm/generated/entities.rs`: Rust structs with PascalCase fields and affirmative booleans.
     - `src-tauri/src/modules/db_orm/generated/tables.rs`: Type-safe table definitions and column constants.
     - `src-tauri/src/modules/db_orm/generated/views.rs`: Rust representation of SQLite pre-compiled views.
     - `src-tauri/src/modules/db_orm/generated/diesel_schema.rs`: Declarative `diesel::table!` definitions.
     - `src-tauri/src/modules/db_orm/generated/diesel_models.rs`: Queryable and Insertable Diesel model structs.

3. **Strict PascalCase & Positive Boolean Discipline:**
   - **Table Names:** Strictly singular PascalCase (e.g. `ErrorLog`, `RunningProject`, `ActivePrompt`, `BackupBatch`, `PromptBackup`, `GreenProject`, `TaskHistory`, `SchemaVersion`).
   - **Primary Keys:** Strictly `{TableName}Id` (e.g. `ErrorLogId`, `RunningProjectId`, `ActivePromptId`).
   - **Column Names:** Strictly PascalCase (e.g. `ProjectId`, `ProjectName`, `CreatedAt`, `IsRunning`, `HasImages`).
   - **Affirmative Booleans:** All boolean fields MUST start with positive prefixes (`Is...`, `Has...`). Negative column names (`is_not_...`, `has_no_...`) are strictly forbidden.

4. **Split-DB Concept & Schema Normalization:**
   - Database responsibilities are decoupled across distinct SQLite files to maintain small database footprints and avoid lock contention:
     - `errors.db`: Runtime and proxy error telemetry (`ErrorLog`, `SchemaVersion`).
     - `repo_prompts.db`: Workspace projects, active prompt goals, sequences (`RunningProject`, `ActivePrompt`, `FailedCommand`, `MediaVault`, `SchemaVersion`).
     - `prompt-backups.db`: Long-term prompt backups and batches (`BackupBatch`, `PromptBackup`, `GreenProject`, `SchemaVersion`).
     - `task-history.db`: High-volume task execution telemetry and audit logs (`TaskHistory`, `SchemaVersion`).
   - Complex join queries are offloaded into pre-defined SQLite views (`VwRecentErrors`, `VwActivePromptsWithProjects`, `VwQueuedPromptsFifo`, `VwUnrestoredBackups`, `VwLatestBackupBatches`, `VwRecentTaskHistory`).
   - FIFO dispatch queue views strictly enforce `ORDER BY CreatedAt ASC, ActivePromptId ASC`.

5. **Single-File Backup & Lossless Re-Import Strategy:**
   - For major schema transitions, historical databases are dumped into a single JSON snapshot: `~/.antigravity_tools/migration/pre_migration_v4_snapshot.json`.
   - Fresh PascalCase SQLite databases are initialized at `SchemaVersion = 1`.
   - Historical records are adapted through bidirectional field-mapping adapters and restored with 100% row-count parity check.

6. **Zero-Account-Loss Cryptographic Guard:**
   - Pre-migration SHA-256 hash and account count of `accounts.json` are captured.
   - Post-migration verification confirms 100% hash equivalence.
   - If an account count divergence or checksum mismatch is detected, the migration triggers an immediate atomic rollback.

7. **Fast Boot Check & Application Integration:**
   - Application startup (`src-tauri/src/lib/setup_app.rs`) executes `modules::db_migration::check_and_run_migrations()` in both GUI and headless modes.
   - Fast boot check executes in `<10ms` by reading `SchemaVersion` before any write operations.

---

## 2. Code Generation Workflow

To regenerate Rust ORM entities and Diesel schemas after modifying table definitions:

```bash
python 03-ai-scripts/51-db-orm-generator.py
```

Generated outputs are placed in `src-tauri/src/modules/db_orm/generated/`.

---

## 3. CLI Management Commands

Antigravity-Manager provides native CLI commands for inspecting and operating the database migration lifecycle:

```bash
# Check current schema version across all Split-DBs
agm db status

# Export all Split-DBs and account verification metadata to snapshot
agm db backup [--output <path>]

# Restore Split-DBs from single-file snapshot
agm db restore [--input <path>]

# Verify account integrity and database row counts
agm db verify
```

---

## 4. Verification & Testing Protocol

When making modifications to the persistence or migration subsystems:

1. **Targeted Rust Unit Tests:**
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml --lib modules::db_
   ```
2. **End-to-End Migration Integration Test:**
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml --test database_orm_migration_e2e
   ```
3. **Format & Quality Gate:**
   ```bash
   cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
   cargo check --manifest-path src-tauri/Cargo.toml --lib
   ```
