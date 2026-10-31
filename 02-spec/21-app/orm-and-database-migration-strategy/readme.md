# ORM and Database Migration Strategy Specification

**Slug:** `orm-and-database-migration-strategy`  
**Status:** DRAFT / IN PROGRESS  
**Author:** Antigravity  
**Date:** 2026-10-18  

---

## 1. Specification Overview

This specification establishes the architectural blueprints for the **ORM-First and Split Database Migration Strategy** in Antigravity-Manager. It transitions all existing SQLite communications from fragmented raw SQL strings into a unified, type-safe Rust ORM framework with automated code generation, strict PascalCase naming across all tables and columns, singular table names, pre-defined SQLite Views for joins, affirmative booleans, and a zero-data-loss single-file backup/re-import migration pipeline.

---

## 2. Specification Documents Index

1. [`01-architecture-spec.md`](01-architecture-spec.md): High-level system architecture, Split-DB boundaries, and Rust ORM layer topology.
2. [`02-schema-and-orm-spec.md`](02-schema-and-orm-spec.md): Complete PascalCase table definitions, columns, primary/foreign key mappings, and database views.
3. [`03-migration-and-backup-spec.md`](03-migration-and-backup-spec.md): Single-file pre-migration export, version number tracking (`SchemaVersion`), zero-account-loss safeguards, and automated re-import engine.

---

## 3. Core Architectural Invariants

- **Invariant 1 (ORM-First Communication):** Raw SQL statements (`SELECT`, `INSERT`, `UPDATE`, `DELETE`) are strictly forbidden in business logic modules (`modules/repo_db`, `modules/error_db`, `modules/backup_prompts_db`, etc.). All data operations must route through the `db_orm` query builder and generated entity structs.
- **Invariant 2 (PascalCase Schema Discipline):** All tables must be singular PascalCase (`ActivePrompt`, `RunningProject`, `ErrorLog`, `PromptBackup`). All column names must be PascalCase (`ActivePromptId`, `PromptContent`, `CreatedAt`).
- **Invariant 3 (Affirmative Booleans):** All boolean fields must use positive prefixes `Is` or `Has` (`IsActive`, `HasImages`, `IsRestored`, `IsRunning`). Negative naming is strictly banned.
- **Invariant 4 (Views Over Joins):** Complex queries joining across tables must be defined as SQLite Views (`Vw...`). The ORM queries views as flat readonly entities.
- **Invariant 5 (Single-File Snapshot & Zero Account Loss):** Upgrades must export existing state to a single unified backup snapshot before spinning up new databases. Historical data is re-imported through adapters. Account configurations (`accounts.json`) are validated to ensure 0 lost profiles.
