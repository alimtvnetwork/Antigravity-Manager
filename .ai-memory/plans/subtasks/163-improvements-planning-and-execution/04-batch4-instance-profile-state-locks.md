# Subtask 04: Batch 4 (Imp 199–204: Multi-Instance Profile State, Locks & Keyrings)

- **Parent Plan**: `.ai-memory/plans/pending/163-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/163-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- **Imp-199 (`lock_reclaimer.rs`)**: Implemented profile stale lockfile reclaimer safely removing orphaned `.antigravity-session` and `code.lock` files when instance is dead.
- **Imp-200 (`arg_normalizer.rs`)**: Implemented instance launch CLI argument normalizer standardizing `--user-data-dir`, `--remote-debugging-port=0`, and keyring bypass flags.
- **Imp-201 (`project_uri.rs`)**: Implemented active project folder URI formatter and `workspace.json` payload generator.
- **Imp-202 (`vitality_scanner.rs`)**: Implemented conscious Win32 / POSIX process vitality scanner distinguishing live processes from dead PIDs.
- **Imp-203 (`session_uuid.rs`)**: Implemented telemetry session UUID rotator generating unique random UUIDv4 values for `telemetry.sessionId` and `telemetry.sqmId`.
- **Imp-204 (`path_sanitizer.rs`)**: Implemented cloned profile recent paths sanitizer purging non-existent directories.
- Re-exported all submodules and types in `src-tauri/src/modules/instance.rs`.

## Targeted Improvements
- **Imp-199: Profile Stale Lockfile Reclaimer** (`src-tauri/src/modules/instance/lock_reclaimer.rs`)
- **Imp-200: Instance Launch Arg Normalizer** (`src-tauri/src/modules/instance/arg_normalizer.rs`)
- **Imp-201: Active Project Folder URI Formatter** (`src-tauri/src/modules/instance/project_uri.rs`)
- **Imp-202: Instance Process Vitality Scanner** (`src-tauri/src/modules/instance/vitality_scanner.rs`)
- **Imp-203: Telemetry Session UUID Rotator** (`src-tauri/src/modules/instance/session_uuid.rs`)
- **Imp-204: Cloned Profile Recent Paths Sanitizer** (`src-tauri/src/modules/instance/path_sanitizer.rs`)
