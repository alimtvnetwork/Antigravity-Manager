# Subtask 04: Windows Batch File Launch Interceptor (`Imp-698`)
Sequence: 214
Step: 284
Status: IN_PROGRESS
Module: `src-tauri/src/modules/instance/windows_batch_file_launch_interceptor.rs`
Test Suite: `src-tauri/src/modules/instance/tests_windows_batch_file_launch_interceptor.rs`

## Description
Enforces AGENTS.md rule: "Never pass .cmd or .bat batch files directly as the executable path in std::process::Command::new(...). Multi-instance launchers must implement multi-stage fallback: target binary -> detected base binary -> Windows Shell dispatch (cmd.exe /c start)." Intercepts direct `.bat`/`.cmd` executions and routes through secure Windows shell dispatch.
