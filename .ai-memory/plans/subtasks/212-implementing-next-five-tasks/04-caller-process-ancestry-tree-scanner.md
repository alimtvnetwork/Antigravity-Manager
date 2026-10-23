# Subtask 04: Caller Process Ancestry Tree Scanner (`Imp-688`)
Status: COMPLETED
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Caller IDE Self-Preservation:
"Caller IDE Self-Preservation & Transient Test Sandboxing: Automated tests, CLI commands, and auto-switcher daemons MUST detect the caller's active IDE process (`detect_caller_instance_id()`) to prevent terminating the developer's working IDE session. Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided."

Implemented `src-tauri/src/modules/instance/caller_process_ancestry_tree_scanner.rs` and unit tests in `src-tauri/src/modules/instance/tests_caller_process_ancestry_tree_scanner.rs`.
Registered in `src-tauri/src/modules/instance.rs`.

## Verification Evidence
- Defined `ProcessNode` and `KNOWN_IDE_BINARIES` (`antigravity.exe`, `cursor.exe`, `windsurf.exe`, `code.exe`).
- Implemented `scan_caller_process_ancestry` ascending parent PPID hierarchy up to a bound of 12 levels.
- Detected caller IDE terminal session, extracted caller instance ID, and recorded PID traversal chain.
- Added comprehensive unit tests covering IDE ancestry detection, standalone shell detection, environment override, and loop termination.
- Deferred E2E command documented in `02-spec/21-app/212-implementing-next-five-tasks/e2e_master_plan.md`.
