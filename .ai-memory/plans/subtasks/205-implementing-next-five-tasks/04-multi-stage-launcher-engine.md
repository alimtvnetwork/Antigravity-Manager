# Subtask 04: Multi-Stage Launcher Engine & Execution Ladder (`Imp-653`)
Status: COMPLETED
Parent Plan: 205-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Process Spawning & Job Breakaway Invariant:
"Never unconditionally specify `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`) without a fallback ladder (`spawn_with_job_breakaway_fallback`). If running inside a restricted Windows Job Object, `CreateProcessW` fails with `os error 5: Access is denied`. Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`. Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`)."

Implement `src-tauri/src/modules/instance/multi_stage_launcher_engine.rs` and unit tests in `src-tauri/src/modules/instance/tests_multi_stage_launcher_engine.rs`.

## Requirements
- Execute process launches across the 3-stage fallback ladder:
  - Stage 1: Verified target binary executable.
  - Stage 2: Detected base binary from registry.
  - Stage 3: Windows Shell dispatch via `cmd.exe /c start "" <path>`.
- Handle Windows Job Object access denial (`os error 5: Access is denied`) by retrying without `CREATE_BREAKAWAY_FROM_JOB`.
- Enforce strict rejection of direct `.bat` or `.cmd` execution via process commands.
- Return structured `ProcessLaunchExecutionReport` with chosen stage, exit code / child PID, and error history.
