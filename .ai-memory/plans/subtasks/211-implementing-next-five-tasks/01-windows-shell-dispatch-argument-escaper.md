# Subtask 01: Windows Shell Dispatch Argument Escaper (`Imp-680`)
Status: COMPLETED
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Process Spawning & Job Breakaway Invariant:
"Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`). Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`."

Implemented `src-tauri/src/modules/instance/windows_shell_dispatch_argument_escaper.rs` and unit tests in `src-tauri/src/modules/instance/tests_windows_shell_dispatch_argument_escaper.rs`.
Registered in `src-tauri/src/modules/instance.rs`.

## Verification Evidence
- Defined `CMD_METACHARACTERS` (`&`, `|`, `(`, `)`, `<`, `>`, `^`, `"`, `%`, `!`, ` `).
- Implemented `escape_shell_argument`, `contains_cmd_metacharacters`, `build_windows_shell_start_command`, and `audit_and_build_shell_dispatch`.
- Handled empty string, quoting whitespace, escaping embedded double quotes, and mandatory empty window title `""`.
- Added comprehensive unit tests covering plain, spaced, and metacharacter arguments.
- Deferred E2E command documented in `02-spec/21-app/211-implementing-next-five-tasks/e2e_master_plan.md`.
