# Subtask 02: Batch 2 Execution (Improvements 07–12: Multi-Instance Sandboxing & Lifecycle)

## Improvements In Scope
- **Imp-07**: Conscious IDE PID Process Verification (`src-tauri/src/modules/instance/lifecycle.rs`)
- **Imp-08**: Graceful 3000ms Instance Stop Ladder (`src-tauri/src/modules/instance/lifecycle.rs`)
- **Imp-09**: Cloned Profile Synthetic UUID Generation (`src-tauri/src/modules/instance/clone.rs`)
- **Imp-10**: Keyring & Account Scrubbing on Clone (`src-tauri/src/modules/instance/clone.rs`)
- **Imp-11**: Stale Lock & Dead PID Purging (`src-tauri/src/modules/instance/registry.rs`)
- **Imp-12**: Port Rebind Settling Delay on Switch (`src-tauri/src/modules/instance/switch.rs`)

## Status
`[COMPLETED]`

## Verification Summary
- **Imp-07**: `is_conscious_ide_pid(pid)` process verification implemented in `lifecycle.rs`.
- **Imp-08**: Graceful stop duration extended to 3000ms before taskkill fallback in `close.rs`.
- **Imp-09**: Fresh synthetic RFC4122 v4 UUIDs generated for `storage.json` device profile and SQLite machine ID in `clone.rs`.
- **Imp-10**: Enhanced account and auth session token scrubbing in `sanitize_session` in `db.rs` preventing session bleeding.
- **Imp-11**: Automatic stale lockfile and dead PID cleanup executed for non-running instances on startup in `registry.rs`.
- **Imp-12**: Explicit 500ms socket settling delay added before instance IDE spawn in `switch.rs`.
