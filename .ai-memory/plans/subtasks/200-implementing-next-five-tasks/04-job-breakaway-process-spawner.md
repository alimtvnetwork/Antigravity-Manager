# Subtask 04: Multi-Stage Launcher Job Breakaway Spawner
Status: COMPLETED
Sequence: 200
Owner: Worker-2
Target Module: `src-tauri/src/modules/instance/job_breakaway_process_spawner.rs`
Test Suite: `src-tauri/src/modules/instance/tests_job_breakaway_process_spawner.rs`

## Evidence
- Implemented `SpawnerTargetKind`, `SpawnFallbackLadderStage`, `ProcessSpawnDecision`, `is_windows_batch_file`, and `resolve_spawn_plan`.
- Enforces AGENTS.md process spawning & job breakaway invariant: never unconditionally specify `CREATE_BREAKAWAY_FROM_JOB` (0x01000000) without ladder fallback on OS error 5 (`Access is denied`), and never pass `.bat` or `.cmd` directly into `Command::new`, resolving to base binary or Windows Shell dispatch (`cmd.exe /c start`).
- 8 unit tests covering standard binary spawn, error 5 step-down, batch file fallback to base binary, batch file fallback to shell dispatch, and Serde roundtrip.
