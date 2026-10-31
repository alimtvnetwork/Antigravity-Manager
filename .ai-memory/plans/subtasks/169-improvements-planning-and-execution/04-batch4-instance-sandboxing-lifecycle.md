# Subtask 04: Batch 4 (Imp 379–384: Multi-Instance Sandboxing & Lifecycle)

- **Parent Plan**: `.ai-memory/plans/pending/169-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/169-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-379: Project Path Canonical Normalizer** (`src-tauri/src/modules/instance/project_path_canonical_normalizer.rs`)
   - Resolves Windows 8.3 short paths, mixed delimiters, and trailing slashes to guarantee consistent repository path indexing.
2. **Imp-380: Keyring Mock Marker Writer** (`src-tauri/src/modules/instance/keyring_mock_writer.rs`)
   - Atomically writes and verifies OS keyring bypass markers and mock flags in sandboxed instance user data directories.
3. **Imp-381: Instance Session Trace UUID** (`src-tauri/src/modules/instance/session_trace_uuid.rs`)
   - Generates cryptographically secure, instance-tagged session trace tokens (`agm-inst-<id>-<uuid>`) for unified telemetry.
4. **Imp-382: Instance Child Process Tree Enumerator** (`src-tauri/src/modules/instance/child_process_tree_enumerator.rs`)
   - Recursively discovers all child processes spawned by an Antigravity IDE instance before safe termination.
5. **Imp-383: Storage JSON Migration Guard** (`src-tauri/src/modules/instance/storage_migration_guard.rs`)
   - Validates `storage.json` integrity across IDE updates, backfilling missing telemetry keys and stripping corrupted state nodes.
6. **Imp-384: Remote Debugging Port Allocator** (`src-tauri/src/modules/instance/remote_debugging_port_allocator.rs`)
   - Scans and allocates free TCP ports for CDP remote debugging with socket bind verification and race-condition resistance.
