# Subtask 04: Cluster SSH First-Open Auto-Sync and Auto-Refresh

- **Parent Plan**: `.ai-memory/plans/pending/238-antigravity-manager-and-tools-ui-issues.md`
- **Slug**: `antigravity-manager-and-tools-ui-issues - Task 04`
- **Status**: PENDING

## Scope
1. Implement `auto_sync_cluster_on_startup()` in `src-tauri/src/modules/ssh_manager/`:
   - Inspect candidate seed paths (`repo-secrets/gitmap-ssh-nodes.json`, `04-ssh-nodes.json`).
   - If `ssh_nodes.db` is empty or stale on application launch, seed nodes automatically.
2. Hook `auto_sync_cluster_on_startup()` into `src-tauri/src/lib/setup_app.rs`.
3. Enhance `src/components/instances/fleet/useFleetMachines.ts` to automatically poll and refresh cluster SSH node health and machine stats every 30 seconds.
