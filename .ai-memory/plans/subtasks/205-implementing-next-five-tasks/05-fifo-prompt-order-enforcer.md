# Subtask 05: SQL FIFO Prompt Query Order Enforcer (`Imp-654`)
Status: COMPLETED
Parent Plan: 205-implementing-next-five-tasks.md
Sequence: 205
Owner: Worker-3
Target Module: `src-tauri/src/modules/db_orm/fifo_prompt_order_enforcer.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_fifo_prompt_order_enforcer.rs`

## Objective
Enforce AGENTS.md strict First-In-First-Out invariant:
"All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."

Implemented `src-tauri/src/modules/db_orm/fifo_prompt_order_enforcer.rs` and unit tests in `src-tauri/src/modules/db_orm/tests_fifo_prompt_order_enforcer.rs`.

## Requirements
- Automatically detect and transform queries targeting prompt queue tables.
- Replace inverted LIFO orderings with canonical `ORDER BY created_at ASC, id ASC`.
- Append canonical FIFO ordering when missing.
- Emit structured `FifoEnforcementReport`.
