# Subtask 04: Batch 4 (Imp 499–504: Multi-Instance Sandboxed Keychain, Environment & Lifecycle)

- **Parent Plan**: `.ai-memory/plans/pending/173-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/173-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-499: Environment Variable Host Token Scrubber** (`src-tauri/src/modules/instance/env_token_scrubber.rs`)
   - Scrubs sensitive host environment variables (`GITHUB_TOKEN`, `OPENAI_API_KEY`) from environment blocks passed to instances.
2. **Imp-500: Stale Socket Lockfile Recovery Sentinel** (`src-tauri/src/modules/instance/socket_lock_recovery.rs`)
   - Detects and cleans abandoned socket lockfiles (`antigravity-*.lock`) left behind by interrupted process shutdowns.
3. **Imp-501: Cloned Profile Telemetry Isolation Verifier** (`src-tauri/src/modules/instance/profile_telemetry_isolation.rs`)
   - Verifies that `storage.json` within cloned instance directories contains unique, non-colliding telemetry salts and machine GUIDs.
4. **Imp-502: Multi-Stage Process Spawn Ladder** (`src-tauri/src/modules/instance/process_spawn_ladder.rs`)
   - Implements a resilient 3-stage spawn ladder: direct target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`).
5. **Imp-503: Non-Blocking Port Availability Guard** (`src-tauri/src/modules/instance/port_allocation_guard.rs`)
   - Performs non-blocking TCP socket bind checks on candidate remote debugging ports before assigning them to instance profiles.
6. **Imp-504: Instance Workspace Subdirectory Guard** (`src-tauri/src/modules/instance/workspace_directory_guard.rs`)
   - Verifies and auto-creates required instance directory hierarchies (`User`, `logs`, `workspaceStorage`) before launch.
