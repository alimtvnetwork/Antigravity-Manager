# Subtask 06: SSH Cluster & Node W2 Parity

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Subtask-06`  
**Status:** `READY`

### Objective
Resolve SSH communication and cluster UI visibility issues:
1. In `gitmap-ssh-nodes.json`, correct node `w2` to `"os": "windows"`.
2. In `src/pages/Instances/components/FleetMachinesTable.tsx`, do not collapse table into `<FleetDisabledState />` when remote mesh nodes are offline. Render them with offline status indicators.
3. In `src-tauri/src/modules/ssh_manager/exec.rs`, allow fallback to GitMap SSH (`gitmap ssh exec <alias>`) when native OpenSSH key authentication fails.

### Target Files
- `gitmap-ssh-nodes.json`
- `src/pages/Instances/components/FleetMachinesTable.tsx`
- `src-tauri/src/modules/ssh_manager/exec.rs`
