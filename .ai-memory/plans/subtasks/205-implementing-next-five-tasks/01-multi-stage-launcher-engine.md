# Subtask 01: Multi-Stage Launcher Engine & Batch Execution Guard
Status: IN PROGRESS
Sequence: 205
Owner: Worker-1
Target Module: `src-tauri/src/modules/instance/multi_stage_launcher_engine.rs`
Test Suite: `src-tauri/src/modules/instance/tests_multi_stage_launcher_engine.rs`
Objective: Enforce AGENTS.md multi-stage process spawning invariant: Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`). Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`.
