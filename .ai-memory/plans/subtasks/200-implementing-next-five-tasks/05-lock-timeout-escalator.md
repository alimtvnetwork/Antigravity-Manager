# Subtask 05: Split-DB Lock Contention Timeout Escalator
Status: IN PROGRESS
Sequence: 200
Owner: Worker-3
Target Module: `src-tauri/src/modules/db_orm/lock_timeout_escalator.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_lock_timeout_escalator.rs`
Objective: Enforce AGENTS.md bounded timeout and error visibility rules: adaptive SQLite split-DB lock timeout escalation under high concurrency, escalating from 250ms to 5000ms with jitter, routing exhausted locks into structured AppError::DatabaseLocked.
