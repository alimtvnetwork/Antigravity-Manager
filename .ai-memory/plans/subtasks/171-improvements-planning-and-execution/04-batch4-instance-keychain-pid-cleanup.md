# Subtask 04: Batch 4 (Imp 439–444: Multi-Instance Sandboxed Keychain, PID Reclaim & Session Cleanup)

- **Parent Plan**: `.ai-memory/plans/pending/171-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/171-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-439: Platform-Agnostic Sandboxed Keychain Path Resolver** (`src-tauri/src/modules/instance/keychain_path_resolver.rs`)
   - Resolves sandboxed mock keyring file paths and mock credential store flags across Windows, Linux, and macOS runtimes.
2. **Imp-440: Instance PID Reclaim Barrier & Liveness Verifier** (`src-tauri/src/modules/instance/pid_reclaim_barrier.rs`)
   - Verifies that a target instance process has truly terminated before releasing saved PID locks, preventing zombie PID re-use.
3. **Imp-441: Volatile Session & Cache Directory Cleanup Worker** (`src-tauri/src/modules/instance/session_cleanup_worker.rs`)
   - Cleans temporary session caches, crash dump directories, and GPU cache folders within sandboxed instance profile roots.
4. **Imp-442: Instance Profile Clone Integrity Verifier** (`src-tauri/src/modules/instance/profile_clone_verifier.rs`)
   - Verifies the byte-for-byte and structural integrity of cloned instance profile directories before activating them in the registry.
5. **Imp-443: Dynamic Port Lease Allocation Tracker** (`src-tauri/src/modules/instance/port_lease_tracker.rs`)
   - Tracks active and reserved TCP/CDP debugging ports across multi-instance sandboxes to prevent binding collisions.
6. **Imp-444: Workspace Storage State & Window Layout Sanitizer** (`src-tauri/src/modules/instance/workspace_storage_sanitizer.rs`)
   - Sanitizes stale workspace storage state files and window coordinate locks to prevent multi-instance window position clumping.
