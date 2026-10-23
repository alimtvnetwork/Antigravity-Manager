# Subtask 04: Windows Batch File Direct Execution Guard

- **Parent**: `193-implementing-next-five-tasks`
- **Status**: `IN_PROGRESS`
- **Target File**: `src-tauri/src/modules/release_gate/batch_file_spawn_guard.rs`
- **Test File**: `src-tauri/src/modules/release_gate/tests_batch_file_spawn_guard.rs`
- **Requirement**: Guard against passing `.cmd`/`.bat` directly to `Command::new`, enforcing 3-stage launcher fallback.
