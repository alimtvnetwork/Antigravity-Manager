# Subtask 07: CLI UI Manipulation Parity

- **Parent Plan**: `.ai-memory/plans/pending/238-antigravity-manager-and-tools-ui-issues.md`
- **Slug**: `antigravity-manager-and-tools-ui-issues - Task 07`
- **Status**: PENDING

## Scope
1. Verify and enhance `agm ui` CLI commands in `src-tauri/src/bin/agm/ui_cmds.rs`:
   - `agm ui open`: Opens/focuses the GUI window.
   - `agm ui goto <route>`: Navigates running GUI to `/instances`, `/fleet`, `/improvements`.
   - `agm ui ls`: Lists active GUI windows and connection status.
   - `agm ui click <selector>`: Programmatic UI interaction.
2. Ensure headless IPC routes in `src-tauri/src/` communicate with running GUI webview.
