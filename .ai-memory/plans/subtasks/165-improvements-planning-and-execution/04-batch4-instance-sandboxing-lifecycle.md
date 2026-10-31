# Subtask 04: Batch 4 (Imp 259–264: Multi-Instance Sandboxing, Process Lifecycle & Workspace Storage)

- **Parent Plan**: `.ai-memory/plans/pending/165-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/165-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Targeted Improvements
- [x] **Imp-259: Orphan Socket & Named Pipe Sweeper** (`src-tauri/src/modules/instance/orphan_socket_sweeper.rs`)
- [x] **Imp-260: Instance Rapid Crash-Loop Quarantine Guard** (`src-tauri/src/modules/instance/crash_loop_guard.rs`)
- [x] **Imp-261: Cloned Profile Telemetry Storage Cleaner** (`src-tauri/src/modules/instance/telemetry_storage_cleaner.rs`)
- [x] **Imp-262: IDE Window Position & Monitor Restorer** (`src-tauri/src/modules/instance/window_state_restorer.rs`)
- [x] **Imp-263: Launch Environment Variable Sanitizer** (`src-tauri/src/modules/instance/launch_env_sanitizer.rs`)
- [x] **Imp-264: Profile Clone Cryptographic Manifest Generator** (`src-tauri/src/modules/instance/profile_clone_manifest.rs`)

## Verification Summary
- `orphan_socket_sweeper.rs`: Detects and removes dangling `.sock`, `.pipe`, and `.lock` files from inactive instances.
- `crash_loop_guard.rs`: Enforces quarantine on instances that experience rapid crashes (<5s) to halt runaway loops.
- `telemetry_storage_cleaner.rs`: Re-seeds machine GUIDs and device UUIDs in `storage.json` to prevent tracking collision.
- `window_state_restorer.rs`: Loads and persists IDE window position bounds across restarts.
- `launch_env_sanitizer.rs`: Strips contaminating parent variables (`NODE_OPTIONS`, `ELECTRON_RUN_AS_NODE`) during child spawn.
- `profile_clone_manifest.rs`: Generates SHA-256 manifests for critical profile configurations to verify clone integrity.
