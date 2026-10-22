# Subtask 04: Batch 4 (Imp 469–474: Multi-Instance Sandboxed Config, Port Reclamation & Launch Telemetry)

- **Parent Plan**: `.ai-memory/plans/pending/172-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/172-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-469: IDE Launch Argv Security Sanitizer** (`src-tauri/src/modules/instance/launch_argv_sanitizer.rs`)
   - Strips dangerous command-line arguments (`--inspect`, `--no-sandbox`, host-specific user data paths) before spawning instances.
2. **Imp-470: CDP Remote Port Reclaim Watchdog** (`src-tauri/src/modules/instance/port_reclaim_watchdog.rs`)
   - Detects when terminated IDE processes close their remote debugging ports and immediately marks port leases as available.
3. **Imp-471: Profile Structural Fingerprint Hasher** (`src-tauri/src/modules/instance/profile_fingerprint_hasher.rs`)
   - Computes SHA-256 hashes of profile configuration files to identify configuration drift across instance clones.
4. **Imp-472: Instance Runtime Crash Frequency Counter** (`src-tauri/src/modules/instance/runtime_crash_counter.rs`)
   - Tracks abnormal crash events per instance profile within a 10-minute sliding window to trigger backoff delay ladders.
5. **Imp-473: Window Layout Coordinates Anchor** (`src-tauri/src/modules/instance/window_layout_anchor.rs`)
   - Persists and staggers default window coordinates for multi-instance launches to prevent window stacking.
6. **Imp-474: Sandboxed Storage Obsolete File Cleaner** (`src-tauri/src/modules/instance/sandboxed_storage_cleaner.rs`)
   - Cleans obsolete backup stores and temporary database snapshots from instance directories.
