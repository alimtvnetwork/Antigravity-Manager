# Subtask 04: Batch 4 (Imp 349–354: Multi-Instance Ports & Crash Loop)

- **Parent Plan**: `.ai-memory/plans/pending/168-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/168-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- Created `keychain_file_guard.rs` verifying mock keyring marker existence and size.
- Created `port_collision_evictor.rs` discovering and evicting conflicting debugging ports.
- Created `crash_loop_backoff.rs` applying exponential backoff to short-lifespan crash loops.
- Created `storage_checksum_guard.rs` verifying telemetry UUID format in storage.json.
- Created `child_health_monitor.rs` tracking heartbeat pulses to detect frozen child IDEs.
- Created `workspace_hash_sentinel.rs` checking deterministic workspace folder hash collisions.
- Re-exported all items in `src-tauri/src/modules/instance.rs`.

## Targeted Improvements
- **Imp-349: Sandboxed Keychain File Guard** (`src-tauri/src/modules/instance/keychain_file_guard.rs`)
- **Imp-350: Port Collision Immediate Evictor** (`src-tauri/src/modules/instance/port_collision_evictor.rs`)
- **Imp-351: Crash Loop Exponential Backoff Guard** (`src-tauri/src/modules/instance/crash_loop_backoff.rs`)
- **Imp-352: Cloned Instance Storage Checksum Guard** (`src-tauri/src/modules/instance/storage_checksum_guard.rs`)
- **Imp-353: Child Process Health Heartbeat Monitor** (`src-tauri/src/modules/instance/child_health_monitor.rs`)
- **Imp-354: Instance Workspace Hash Collision Sentinel** (`src-tauri/src/modules/instance/workspace_hash_sentinel.rs`)
