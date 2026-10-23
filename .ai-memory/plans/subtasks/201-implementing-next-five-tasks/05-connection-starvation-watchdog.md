# Subtask 05: Split-DB Connection Pool Starvation & Checkout Watchdog
Status: IN PROGRESS
Sequence: 201
Owner: Worker-3
Target Module: `src-tauri/src/modules/db_orm/connection_starvation_watchdog.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_connection_starvation_watchdog.rs`
Objective: Enforce AGENTS.md bounded timeout and error visibility rules: track SQLite split-DB connection checkouts across threads, detect pool starvation exceeding 3000ms threshold, and alert without swallowing errors.
