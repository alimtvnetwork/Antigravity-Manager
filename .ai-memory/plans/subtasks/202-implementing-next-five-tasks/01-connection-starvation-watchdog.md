# Subtask 01: Split-DB Connection Pool Starvation & Checkout Watchdog
Status: COMPLETED
Sequence: 202
Owner: Worker-1
Target Module: `src-tauri/src/modules/db_orm/connection_starvation_watchdog.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_connection_starvation_watchdog.rs`
Objective: Enforce AGENTS.md bounded wait and timeout rules: Under high concurrency across split SQLite databases (`active_prompts`, `repo_prompts.db`), tracks active connection checkout durations across threads and detects thread pool starvation before query timeout exhaustion. Complies with the AGENTS.md mandate: "Bound every wait with a timeout" and "No Silent Errors".
Lines: 241 lines implementation + 230 lines unit tests.
