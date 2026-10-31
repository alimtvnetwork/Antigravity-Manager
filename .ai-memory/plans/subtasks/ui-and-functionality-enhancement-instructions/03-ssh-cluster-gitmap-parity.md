# Subtask 03: SSH Cluster & Node W2 Parity

Parent Task: `ui-and-functionality-enhancement-instructions`
Task-ID: `Task-04`
Owner: Worker 03

## Objectives
1. In `src-tauri/src/modules/supabase_sync/query.rs`, allow returning local machine + SSH mesh nodes via `build_local_and_ssh_fleet()` when Supabase sync is disabled.
2. In `useFleetMachines.ts`, remove the hard abort when `!is_sync_enabled`, enabling local mesh cluster view.
3. In `FleetMachinesTable.tsx`, allow table rendering when `machines.length > 0` in local mesh mode.
4. Expand candidate seed node paths in `src-tauri/src/modules/ssh_manager/paths.rs` to include `04-ssh-nodes.json` and repo-secrets.
5. Support W2 connection repair via GitMap SSH fallback mechanics.
