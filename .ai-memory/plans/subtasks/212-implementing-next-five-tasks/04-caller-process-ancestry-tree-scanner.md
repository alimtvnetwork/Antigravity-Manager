# Subtask 04: Caller Process Ancestry Tree Scanner (`Imp-688`)
Status: IN_PROGRESS
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Caller IDE Self-Preservation:
"Caller IDE Self-Preservation & Transient Test Sandboxing: Automated tests, CLI commands, and auto-switcher daemons MUST detect the caller's active IDE process (`detect_caller_instance_id()`) to prevent terminating the developer's working IDE session. Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided."

Implement `src-tauri/src/modules/instance/caller_process_ancestry_tree_scanner.rs` and unit tests in `src-tauri/src/modules/instance/tests_caller_process_ancestry_tree_scanner.rs`.

## Requirements
- Inspect process hierarchy upward through PPID ancestry to identify caller origin.
- Detect if current process is running inside an active Antigravity IDE terminal session.
- Return structured caller detection report identifying caller instance ID and process tree depth.
- Comprehensive unit tests covering ancestry traversal, root termination, and caller ID extraction.
