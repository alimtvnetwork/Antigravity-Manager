# Subtask 05: Multi-Stage Launcher Fallback & Batch File Guard (`Imp-649`)
Status: COMPLETED
Parent Plan: 204-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Process Spawning & Job Breakaway Invariant:
"Never unconditionally specify `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`) without a fallback ladder (`spawn_with_job_breakaway_fallback`). If running inside a restricted Windows Job Object, `CreateProcessW` fails with `os error 5: Access is denied`. Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`. Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`)."

Implement `src-tauri/src/modules/instance/multi_stage_launcher_dispatcher.rs` and unit tests in `src-tauri/src/modules/instance/tests_multi_stage_launcher_dispatcher.rs`.

## Requirements
- Multi-stage launcher fallback coordination:
  - Stage 1: Verified target binary executable path.
  - Stage 2: Detected base binary from registry.
  - Stage 3: Windows Shell dispatch via `cmd.exe /c start "" <path>`.
- Batch file direct execution prohibition: intercept and reject any `.cmd` or `.bat` file passed directly to `std::process::Command::new`.
- Windows Job Object breakaway fallback: simulate/handle `os error 5: Access is denied` by falling back to standard process spawn without breakaway flags.
- Return structured `LaunchResult` with stage used, fallback attempts, and launcher violations.
