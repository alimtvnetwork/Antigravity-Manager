# Subtask 01: Windows Shell Dispatch Argument Escaper (`Imp-680`)
Status: IN_PROGRESS
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Process Spawning & Job Breakaway Invariant:
"Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`). Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`."

Implement `src-tauri/src/modules/instance/windows_shell_dispatch_argument_escaper.rs` and unit tests in `src-tauri/src/modules/instance/tests_windows_shell_dispatch_argument_escaper.rs`.

## Requirements
- Escape command arguments according to Windows `cmd.exe` shell rules.
- Sanitize shell metacharacters (`&`, `|`, `(`, `)`, `<`, `>`, `^`, `"`, `%`).
- Provide safe argument formatting when constructing fallback `cmd.exe /c start ""` command lines.
- Comprehensive unit tests covering quoting, spaces, metacharacters, and empty strings.
