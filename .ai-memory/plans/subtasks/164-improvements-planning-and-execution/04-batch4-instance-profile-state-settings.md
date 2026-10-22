# Subtask 04: Batch 4 (Imp 229–234: Multi-Instance Profile State, Settings & Session Guards)

- **Parent Plan**: `.ai-memory/plans/pending/164-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/164-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- **Imp-229 (`extension_sanitizer.rs`)**: Implemented extension runtime state sanitizer purging stale socket and lock files (`.sock`, `.ipc`, `.lock`).
- **Imp-230 (`port_matrix.rs`)**: Implemented multi-profile port collision matrix preventing port overlaps across concurrent CDP sessions.
- **Imp-231 (`identity_reseeder.rs`)**: Implemented cloned profile telemetry identity reseeder regenerating unique machine, MAC, and device UUIDs.
- **Imp-232 (`workspace_hash.rs`)**: Implemented FNV-1a deterministic workspace folder hash resolver matching IDE conventions.
- **Imp-233 (`health_probe.rs`)**: Implemented instance CDP loopback health readiness probe runner.
- **Imp-234 (`state_snapshot.rs`)**: Implemented instance runtime state persistence snapshotter for crash recovery.
- Re-exported all submodules and types in `src-tauri/src/modules/instance.rs`.

## Targeted Improvements
- **Imp-229: Profile Extension State Sanitizer** (`src-tauri/src/modules/instance/extension_sanitizer.rs`)
- **Imp-230: Multi-Profile Port Collision Matrix** (`src-tauri/src/modules/instance/port_matrix.rs`)
- **Imp-231: Cloned Profile Machine Identity Reseeder** (`src-tauri/src/modules/instance/identity_reseeder.rs`)
- **Imp-232: Workspace Storage Hash Path Resolver** (`src-tauri/src/modules/instance/workspace_hash.rs`)
- **Imp-233: Instance Health Check Probe Runner** (`src-tauri/src/modules/instance/health_probe.rs`)
- **Imp-234: Instance State Persistence Snapshotter** (`src-tauri/src/modules/instance/state_snapshot.rs`)
