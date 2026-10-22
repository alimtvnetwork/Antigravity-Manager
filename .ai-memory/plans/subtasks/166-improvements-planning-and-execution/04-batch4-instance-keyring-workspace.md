# Subtask 04: Batch 4 (Imp 289–294: Multi-Instance Keyring Bypass, Process Health & Workspace Storage)

- **Parent Plan**: `.ai-memory/plans/pending/166-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/166-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- Created `mock_keyring_writer.rs` writing `1\n` to bypass markers (`antigravity-keyring-unavailable`, etc.).
- Created `process_vitality_probe.rs` providing fast PID probe and process naming diagnostics.
- Created `workspace_storage_migrator.rs` handling deep copy, json rewrite, and anchor verification.
- Created `port_reclaim_sentinel.rs` sweeping stale CDP debugging ports.
- Created `launch_argv_builder.rs` constructing sandboxed CLI flags with mock keychain.
- Created `session_file_reclaimer.rs` safely clearing orphaned singleton locks.
- Re-exported all items in `src-tauri/src/modules/instance.rs`.

## Targeted Improvements
- **Imp-289: Mock Keyring Marker Writer** (`src-tauri/src/modules/instance/mock_keyring_writer.rs`)
- **Imp-290: Lightweight Process Vitality Probe** (`src-tauri/src/modules/instance/process_vitality_probe.rs`)
- **Imp-291: Workspace Storage Migrator & Re-anchor** (`src-tauri/src/modules/instance/workspace_storage_migrator.rs`)
- **Imp-292: Abandoned Debugging Port Reclaimer** (`src-tauri/src/modules/instance/port_reclaim_sentinel.rs`)
- **Imp-293: Modular Instance Launch Argv Builder** (`src-tauri/src/modules/instance/launch_argv_builder.rs`)
- **Imp-294: Singleton Session Lockfile Reclaimer** (`src-tauri/src/modules/instance/session_file_reclaimer.rs`)
