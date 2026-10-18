# Subtask 02: SSH Manager & Shared Cluster Node W2 Parity

## Scope
- Files: `src-tauri/src/modules/ssh_manager/exec.rs`, `src/components/instances/fleet/useFleetMachines.ts`

## Implementation Steps
1. In `src-tauri/src/modules/ssh_manager/exec.rs`:
   - When native OpenSSH exits with code 255 (auth failure), add fallback ladder to invoke `gitmap ssh exec <node_alias_or_ip> "<command>"`.
2. In `src/components/instances/fleet/useFleetMachines.ts`:
   - Decouple local SSH mesh nodes from the remote Supabase cloud sync toggle so local nodes remain visible and queryable.
