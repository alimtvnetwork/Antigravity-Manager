---
plan: 117-instance-clone-settings-security-presets-and-card-progress-ui
subtask: "003"
title: Backend projects, recent paths, and workspace storage replication
domain: backend/workspaces
target_files:
  - src-tauri/src/modules/instance.rs
status: pending
---

# 003 — Backend Projects, Recent Paths, and Workspace Storage Replication

## Scope
1. In `src-tauri/src/modules/instance.rs`:
   - In `merge_state_vscdb_recent_paths`:
     - Open source SQLite connection with `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI` and URI parameter `file:...state.vscdb?mode=ro&immutable=1`.
     - Extract and merge modern VS Code recent paths (`history.recentlyOpenedPathsList`), `profileAssociations.*`, and `workbench.colorTheme` into the destination database.
   - In `copy_instance_projects`:
     - Resiliently clone each workspace directory under `User/workspaceStorage/`.
     - Ensure that if an individual SQLite database in a specific workspace is locked, it uses `safe_clone_sqlite_db` and logs without aborting the copying of other workspace directories.
   - Synchronize `storage.json` `openedPathsList` and `backupWorkspaces`.
