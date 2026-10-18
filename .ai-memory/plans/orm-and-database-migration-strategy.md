# Master Plan: ORM and Database Migration Strategy

Request slug: `orm-and-database-migration-strategy`
Status: COMPLETED
Phase: Complete
Capacity: A = 3 agents × H = 2 hands = 6 concurrent operations

## User Request (Verbatim)
# ORM and Database Migration Strategy: high priority instruction, non-negotiable task

Can it please ensure that we are using ORM for the SQLite communication? That's first thing. Second is that try to follow the `gitmap` way or the coding guideline. The database communication needs to be in a healthy way so that changing one place does change in other places. First of all, if you're using SQL, which we hardly avoid, it should be somewhere like a ORM that would be used like Entity Framework. I'm not sure about the Rust. If they have it, in that case, you use a generator to generate this object automatically. You should have a generator script, just like we have it in the `gitmap`. And also the database, every column and everything needs to be a Pascal case, which is not I have seen in your code. So make sure that we convert that. And so many tables that we have created in the past, and migration would be a very much difficult situation. The way that we are going to solve it is that we are going to use, let's say, backup to a single Database (Database (Db)) or file, and then re-import once the new database is formed. That means for this migration, we want to have a fresh Database (Database (Db)), which starts with the Pascal case, no migration. But later on, whichever change we make, we have to go with the migration. Do you understand? And also you have to confirm that you are using the split Database (Database (Db)) concept to reduce the Database (Database (Db)) size. Also, you're using the normalization of the database so that the data looks small, and also you use views to see a complex query. Please create a checklist under the AI memory with strict rules for the future. Create your spec so that this does not happen again. And previously, I have given the command using voice. The split Database (Database (Db)) became speed error database, which is wrong. It should be split Database (Database (Db)). Okay? Try to understand what I just said and make sure that you do a end-to-end test of the current data and make sure no account is lost. Make sure of that. Okay? And the migration needs to be very smooth. So you test it so that we do not lose anything, okay, whenever we install to a new machine with the existing settings or things like that. Does this make sense? So existing settings would be, let's say, exported, and then after creating the new database, that's the first time, if we have the old database and system, so it would know by the version number when we have the oldest database, that would be marked with the version number or the database, let's say, scheme. Checking any one of these thing, database schema means does not mean that we have-- I think the version number is enough because we are migrating from this version onwards to the new one. Okay? So follow through this. Does this make sense?

## Task Inventory & Status Matrix

| Task-ID | Title | Owner | Status | Target Files |
|---|---|---|---|---|
| Task-01 | Architecture Specification & AI Memory Checklist | Lead | DONE | `02-spec/21-app/orm-and-database-migration-strategy/`, `.ai-memory/checklists/` |
| Task-02 | Code Generator Script (`51-db-orm-generator.py`) for Rust ORM Models | Worker 01 | DONE | `03-ai-scripts/51-db-orm-generator.py`, `src-tauri/src/modules/db_orm/` |
| Task-03 | Rust Type-Safe ORM Abstraction Layer (`db_orm` module) | Worker 01 | DONE | `src-tauri/src/modules/db_orm/`, `mod.rs`, `traits.rs`, `builder.rs` |
| Task-04 | PascalCase Split-DB Schemas & Views Engine (`errors.db`, `repo-prompts.db`, `prompt-backups.db`) | Worker 02 | DONE | `src-tauri/src/modules/db_orm/generated/` |
| Task-05 | Unified Single-File Backup & Pre-Migration Exporter | Worker 02 | DONE | `src-tauri/src/modules/db_migration/backup.rs` |
| Task-06 | Zero-Data-Loss Importer & Schema Version Tracking (`SchemaVersion`) | Worker 03 | DONE | `src-tauri/src/modules/db_migration/restore.rs`, `version.rs` |
| Task-07 | Account Data Preservation & Integrity Safeguard (Zero Account Loss) | Worker 03 | DONE | `src-tauri/src/modules/db_migration/account_guard.rs` |
| Task-08 | CLI Integration (`agm db backup`, `agm db restore`, `agm db status`) | Worker 01 | DONE | `src-tauri/src/bin/agm/db_cmds.rs`, `main.rs`, `help_text.rs` |
| Task-09 | End-to-End Migration & Verification Testing | Worker 02 | DONE | `src-tauri/src/modules/db_migration/tests.rs`, `src-tauri/tests/database_orm_migration_e2e.rs` |
| Task-10 | Pre-flight Quality Gates, Fmt, Clippy & Atomic GitMap Commit | Lead | DONE | Full preflight pass & atomic GitMap push |
