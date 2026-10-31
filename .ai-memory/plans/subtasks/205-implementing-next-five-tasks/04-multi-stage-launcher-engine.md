# Subtask 04: Multi-Stage Launcher Engine & Execution Ladder (`Imp-653`)
Status: COMPLETED
Parent Plan: 205-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Process Spawning & Job Breakaway Invariant:
"Multi-instance launchers must implement multi-stage fallback: target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`). Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`. If running inside a restricted Windows Job Object, `CreateProcessW` fails with `os error 5: Access is denied`."

Implemented `src-tauri/src/modules/instance/multi_stage_launcher_engine.rs` and unit tests in `src-tauri/src/modules/instance/tests_multi_stage_launcher_engine.rs`.

## Requirements
- Provide `build_launch_plan` resolving target binary, base binary fallback, or Windows Shell dispatch.
- Wrap `.cmd`/`.bat` scripts with `cmd.exe /c start`.
- Guard against Win32 Job Object breakaway denial (`os error 5`).
