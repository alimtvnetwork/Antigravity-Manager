# Subtask 04: Batch 4 (Imp 529–534: Multi-Instance Sandboxed Profile Isolation & Process Watchdogs)

- **Parent Plan**: `.ai-memory/plans/pending/174-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/174-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-529: Sandbox Toolchain PATH Normalizer** (`src-tauri/src/modules/instance/env_path_normalizer.rs`)
   - Normalizes system `PATH` entries for child IDE instances, ensuring sandbox tools and GitMap take precedence.
2. **Imp-530: Stale Instance PID Process Reaper** (`src-tauri/src/modules/instance/stale_pid_reaper.rs`)
   - Inspects tracked instance PIDs against OS process lists, marking orphaned instance records as stopped cleanly.
3. **Imp-531: Instance Profile Configuration SHA-256 Hasher** (`src-tauri/src/modules/instance/profile_state_hasher.rs`)
   - Computes SHA-256 fingerprint hashes of instance profile configuration states to detect external edits.
4. **Imp-532: IDE Binary Architecture and Execution Probe** (`src-tauri/src/modules/instance/binary_execution_probe.rs`)
   - Probes candidate IDE executables for launchability, verifying binary architecture and execution permissions.
5. **Imp-533: Remote Debugging Port Lease Guard** (`src-tauri/src/modules/instance/remote_port_guard.rs`)
   - Reserves and validates high-range remote debugging ports (9222–9299), tracking port leases across instances.
6. **Imp-534: Instance Directory Permission Pre-Flight Guard** (`src-tauri/src/modules/instance/directory_permission_guard.rs`)
   - Checks read/write permissions across instance data directories before initiating process launch.
