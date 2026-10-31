# Subtask 04: Batch File Execution Interceptor (`Imp-698`)
Sequence: 214
Step: 284
Status: COMPLETED
Module: `src-tauri/src/modules/instance/batch_file_execution_interceptor.rs`
Test Suite: `src-tauri/src/modules/instance/tests_batch_file_execution_interceptor.rs`

## Description
Enforces AGENTS.md rule: "Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`. Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`)."
