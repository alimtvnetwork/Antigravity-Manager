# Subtask 01: Batch 1 (Imp 151–156: Multi-Instance Profile Sandbox Isolation & Keyring Security)

- **Parent Plan**: `.ai-memory/plans/pending/162-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/162-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- **Imp-151: Multi-Instance Local Storage Scrubber** (`src-tauri/src/modules/instance/storage_scrubber.rs`): Deep scrubbing of telemetry keys (`telemetry.machineId`, `macMachineId`, `sqmId`, `devDeviceId`) and auth tokens with synthetic UUID generation.
- **Imp-152: Ephemeral Sandbox Profile Purger** (`src-tauri/src/modules/instance/sandbox_purger.rs`): Identifies and recursively removes ephemeral test profiles (`.antigravity-sandbox-*`, `.test-profile-*`) with locked file resilience.
- **Imp-153: Keyring Mock Mode Arguments Injector** (`src-tauri/src/modules/instance/keyring_flags.rs`): Injects `--password-store=basic` and `--use-mock-keychain` into sandboxed IDE commands.
- **Imp-154: Instance Workspace Continuity Detector** (`src-tauri/src/modules/instance/workspace_continuity.rs`): Inspects `workspaceStorage` folders and extracts `workspace.json` target folders.
- **Imp-155: Cross-Instance Port Collision Detector** (`src-tauri/src/modules/instance/port_detector.rs`): Probes loopback TCP ports starting at 9222 to allocate next free non-conflicting CDP port.
- **Imp-156: Profile Clone Integrity Checksum** (`src-tauri/src/modules/instance/clone_checksum.rs`): Verifies file presence and byte parity between source and destination profile configs (`settings.json`, `keybindings.json`).

## Targeted Improvements
- **Imp-151: Multi-Instance Local Storage Scrubber** (`src-tauri/src/modules/instance/storage_scrubber.rs`)
- **Imp-152: Ephemeral Sandbox Profile Purger** (`src-tauri/src/modules/instance/sandbox_purger.rs`)
- **Imp-153: Keyring Mock Mode Arguments Injector** (`src-tauri/src/modules/instance/keyring_flags.rs`)
- **Imp-154: Instance Workspace Continuity Detector** (`src-tauri/src/modules/instance/workspace_continuity.rs`)
- **Imp-155: Cross-Instance Port Collision Detector** (`src-tauri/src/modules/instance/port_detector.rs`)
- **Imp-156: Profile Clone Integrity Checksum** (`src-tauri/src/modules/instance/clone_checksum.rs`)
