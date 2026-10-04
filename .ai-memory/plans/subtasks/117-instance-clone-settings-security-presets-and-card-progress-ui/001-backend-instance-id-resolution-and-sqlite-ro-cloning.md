---
plan: 117-instance-clone-settings-security-presets-and-card-progress-ui
subtask: "001"
title: Backend instance ID resolution and resilient read-only SQLite cloning
domain: backend/cloning
target_files:
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/commands/instance.rs
  - src-tauri/src/modules/cli.rs
status: pending
---

# 001 — Backend Instance ID Resolution and Resilient Read-Only SQLite Cloning

## Scope
1. In `src-tauri/src/modules/instance.rs`:
   - In `copy_instance_with_options`: call `resolve_instance_id(source_id)` to resolve `"default"`, `"Default"`, sequence numbers, or name aliases before querying the registry.
   - In `safe_clone_sqlite_db`:
     - Open source connection with `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI`.
     - Use URI parameters `?mode=ro&immutable=1` to allow read access even when the source database is exclusively locked by a running instance.
     - Execute incremental backup `step(100)` with backoff retries (up to 2.5 seconds) to handle transient `SQLITE_BUSY` states.
     - Fall back to shared file copy copying `.vscdb`, `-wal`, and `-shm` sidecars with non-exclusive file sharing flags.
2. In `src-tauri/src/commands/instance.rs`:
   - In `copy_instance`, call `resolve_instance_id`.
3. In `src-tauri/src/modules/cli.rs`:
   - In `copy-profile` (`dp`) and `instance copy` (`clone`), call `resolve_instance_id`.
