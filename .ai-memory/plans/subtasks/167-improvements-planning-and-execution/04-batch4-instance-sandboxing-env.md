# Subtask 04: Batch 4 (Imp 319–324: Multi-Instance Sandboxing, Telemetry Isolation & Environment Pruning)

- **Parent Plan**: `.ai-memory/plans/pending/167-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/167-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- Created `env_whitelist_filter.rs` stripping parent VS Code/Node environment variables.
- Created `profile_dir_validator.rs` validating directory containment and writeability.
- Created `ipc_socket_cleaner.rs` cleaning stale VS Code/Electron IPC sockets.
- Created `terminate_fallback_ladder.rs` providing 3-tier progressive PID shutdown.
- Created `cloned_guid_reseeder.rs` regenerating synthetic MachineGuids in storage.json.
- Created `state_snapshot_exporter.rs` exporting sanitized instance health and port JSON.
- Re-exported all items in `src-tauri/src/modules/instance.rs`.

## Targeted Improvements
- **Imp-319: Environment Variable Whitelist Filter** (`src-tauri/src/modules/instance/env_whitelist_filter.rs`)
- **Imp-320: Profile Sandboxed Directory Validator** (`src-tauri/src/modules/instance/profile_dir_validator.rs`)
- **Imp-321: VS Code IPC Socket Cleaner** (`src-tauri/src/modules/instance/ipc_socket_cleaner.rs`)
- **Imp-322: Conscious Process Terminate Fallback Ladder** (`src-tauri/src/modules/instance/terminate_fallback_ladder.rs`)
- **Imp-323: Cloned Profile MachineGuid Reseeder** (`src-tauri/src/modules/instance/cloned_guid_reseeder.rs`)
- **Imp-324: Instance State Health Snapshot Exporter** (`src-tauri/src/modules/instance/state_snapshot_exporter.rs`)
