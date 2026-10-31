# Subtask 3: Fleet Machine Detection (W2 Node) and Duplicate Button Removal

- **Subtask ID**: 242-03
- **File**: `02-spec/21-app/242-prompt-tree-and-ui-bug-fixes/03-subtask-fleet-machine-detection-and-duplicate-button.md`
- **Category**: Backend Supabase Sync / Frontend Fleet Table / Instances Page
- **Status**: Ready for Execution

---

## 1. Objectives & Scope
1. Enforce IP-level deduplication in `collect_all_peer_connections()` (`query.rs`) so that `~/.ssh/config` alias `server` pointing to `192.168.1.3` is discarded or merged into canonical cluster node `w1`.
2. Ensure local node alias resolution (`node.rs`) checks `~/.gitmap/variables.json` and falls back to matching local IP `192.168.1.7` against `GITMAP_CLUSTER_NODES` (`"w2"`).
3. Update `useFleetMachines.ts` to identify the local machine by matching `localNodeInfo.ip_address` or `localNodeInfo.node_alias` from `getLocalNodeInfo()`.
4. Restrict the `Lead` badge in `useFleetMachines.ts` and `FleetMachineRow.tsx` strictly to `alias === 'w1'`, removing the loose IP-based match so `server` never receives a false `Lead` badge.
5. Remove the redundant duplicate `+ New Instance` button from lines 677–694 in `InstancesHeader.tsx`, retaining the primary button in `InstancesStates.tsx`.

---

## 2. Target Files
- `src-tauri/src/modules/supabase_sync/query.rs`
- `src-tauri/src/modules/supabase_sync/node.rs`
- `src/components/instances/fleet/useFleetMachines.ts`
- `src/components/instances/fleet/FleetMachineRow.tsx`
- `src/pages/instances/InstancesHeader.tsx`
