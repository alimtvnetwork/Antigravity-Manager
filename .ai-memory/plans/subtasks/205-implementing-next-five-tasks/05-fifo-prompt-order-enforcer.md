# Subtask 05: SQL FIFO Prompt Query Order Enforcer
Status: IN PROGRESS
Sequence: 205
Owner: Worker-3
Target Module: `src-tauri/src/modules/db_orm/fifo_prompt_order_enforcer.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_fifo_prompt_order_enforcer.rs`
Objective: Enforce AGENTS.md strict First-In-First-Out invariant: Automatically rewrites, sanitizes, and enforces canonical `ORDER BY created_at ASC, id ASC` sorting across SQLite queries, stripping forbidden LIFO descending clauses.
