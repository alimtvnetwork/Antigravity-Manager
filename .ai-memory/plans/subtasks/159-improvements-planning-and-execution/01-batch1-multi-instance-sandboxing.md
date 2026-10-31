# Subtask 01: Batch 1 Execution (Improvements 61–66: Multi-Instance Profile Sandboxing & Process Hygiene)

## Improvements In Scope
- **Imp-61**: Stale Session Lock Recovery (`src-tauri/src/modules/instance/lifecycle.rs`)
- **Imp-62**: Sandboxed Keyring Bypass Marker (`src-tauri/src/modules/instance/launch.rs`)
- **Imp-63**: Instance Launch Backoff Ladder (`src-tauri/src/modules/instance/lifecycle.rs`)
- **Imp-64**: Deep-Merge Telemetry Scrubbing (`src-tauri/src/modules/instance/clone.rs`)
- **Imp-65**: Cross-Platform Shell Escaper (`src-tauri/src/utils/command.rs`)
- **Imp-66**: Conscious PID Verification (`src-tauri/src/modules/process_cleaner.rs`)

## Status
`[COMPLETED]`

## Execution Summary
- **Imp-61**: Implemented `recover_stale_session_locks` in `src-tauri/src/modules/instance/lifecycle.rs`, scanning and safely unlinking orphaned lockfiles (`.vscode-server.lock`, `.antigravity-session`, `SingletonLock`, etc.) when the holding PID is dead.
- **Imp-62**: Added `--use-mock-keychain` alongside `--password-store=basic` across all 4 spawn pathways in `src-tauri/src/modules/instance/launch.rs` for isolated sandboxing.
- **Imp-63**: Added `probe_instance_cdp_with_backoff` in `src-tauri/src/modules/instance/lifecycle.rs` with configurable retry backoff ladder.
- **Imp-64**: Added `scrub_storage_json_telemetry` in `src-tauri/src/modules/instance/clone.rs` generating fresh synthetic SHA-256 IDs for `telemetry.machineId` and `telemetry.macMachineId`, plus fresh UUIDs for device and session IDs.
- **Imp-65**: Added `escape_cmd_arg`, `escape_posix_arg`, `escape_shell_arg`, and `join_shell_args` in `src-tauri/src/utils/command.rs`.
- **Imp-66**: Created `src-tauri/src/modules/process_cleaner.rs` implementing `is_verified_antigravity_pid`, `verify_parent_child_hierarchy`, and `safe_terminate_conscious_pid`, registered in `modules/mod.rs`.
