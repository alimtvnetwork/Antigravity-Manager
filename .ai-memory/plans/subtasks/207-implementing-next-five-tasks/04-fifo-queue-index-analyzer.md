# Subtask 04: FIFO Queue SQLite Composite Index Analyzer (`Imp-663`)
Status: IN_PROGRESS
Parent Plan: 207-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md FIFO Restore & Queue Invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."

Implement `src-tauri/src/modules/db_orm/fifo_queue_index_analyzer.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_queue_index_analyzer.rs`.

## Requirements
- Analyze SQLite index definitions across `active_prompts`, `prompt_backups`, and `repo_prompts`.
- Verify existence of composite index on `(created_at, id)` or `(created_at ASC, id ASC)`.
- Flag missing indexes as performance and ordering risks.
- Generate automated SQL remediation statements (`CREATE INDEX IF NOT EXISTS idx_<table_name>_created_at_id ON <table_name>(created_at ASC, id ASC);`).
- Return structured `FifoIndexAnalysisReport` with index status and remediation queries.
