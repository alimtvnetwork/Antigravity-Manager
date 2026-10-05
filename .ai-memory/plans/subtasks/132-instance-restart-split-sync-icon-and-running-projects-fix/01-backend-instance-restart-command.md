# Subtask 01: Backend Instance Restart Command & Lifecycle

- **Objective**: Implement `restart_instance` in `src-tauri/src/modules/instance.rs`, expose Tauri command in `src-tauri/src/commands/instance.rs` and register in `src-tauri/src/lib.rs`.
- **Details**:
  - Stop instance via `stop_instance`.
  - Wait for process termination.
  - Launch instance via `launch_instance`.
  - Expose IPC command and frontend service method.
