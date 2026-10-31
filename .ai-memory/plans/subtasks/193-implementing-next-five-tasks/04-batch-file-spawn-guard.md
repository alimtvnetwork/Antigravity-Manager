# Subtask 04: Windows Batch File Direct Execution Guard

- **Parent**: `193-implementing-next-five-tasks`
- **Status**: `COMPLETED`
- **Target File**: `src-tauri/src/modules/release_gate/batch_file_spawn_guard.rs`
- **Test File**: `src-tauri/src/modules/release_gate/tests_batch_file_spawn_guard.rs`
- **Outcome**: Implemented `validate_executable_path`, `is_batch_file_extension`, and `resolve_windows_execution_command`. Intercepts raw `.bat`/`.cmd` files from direct `Command::new` execution and wraps them in `cmd.exe /c start "" <path> <args>`. Tests cover extension checking, validation rejection, and shell command wrapping.
