# Subtask 04: Batch 4 (Imp 409–414: Multi-Instance Sandboxing, Sockets & Environment)

- **Parent Plan**: `.ai-memory/plans/pending/170-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/170-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-409: Sandboxed Environment Variable Allowlist** (`src-tauri/src/modules/instance/env_allowlist_filter.rs`)
   - Filters environment variables passed to spawned IDE instances, stripping parent proxy tokens and developer secrets.
2. **Imp-410: Stale Unix/Named Pipe Socket Cleaner** (`src-tauri/src/modules/instance/pipe_socket_cleaner.rs`)
   - Inspects instance runtimes for orphaned named pipes and domain socket files, removing them safely before launch.
3. **Imp-411: Instance State Snapshot Rollback** (`src-tauri/src/modules/instance/state_snapshot_rollback.rs`)
   - Creates pre-switch state snapshots of instance configuration files and enables 1-step rollback if launch fails.
4. **Imp-412: Cloned Profile Telemetry Salt Generator** (`src-tauri/src/modules/instance/cloned_telemetry_salt.rs`)
   - Computes and salts distinct anonymous telemetry hash seeds for duplicated instance workspaces.
5. **Imp-413: Child Process Termination Guard** (`src-tauri/src/modules/instance/child_termination_guard.rs`)
   - Provides a 3-stage escalating kill ladder for stubborn IDE child processes.
6. **Imp-414: Instance Workspace Directory Validator** (`src-tauri/src/modules/instance/workspace_dir_validator.rs`)
   - Validates that target workspace directories exist, have write permissions, and contain valid project metadata.
