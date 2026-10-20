# Subtask 01: Fleet Caching, Lead Alignment & Cross-Machine Dispatch

**Subtask ID**: `sub-01-fleet-cache`  
**Parent Plan**: `.ai-memory/plans/telegram-backup-and-fleet-management-enhancements.md`  
**Status**: IN_PROGRESS  

## Objectives
1. Implement 2-tier cache for fleet machines:
   - Frontend: `localStorage` (`agm_fleet_machines_cache_v2`) populated synchronously on mount, eliminating the stuck skeleton state `<FleetLoadingState />`.
   - Backend: 30s TTL in-memory cache in `src-tauri/src/modules/ssh_manager/exec.rs`.
2. Machine naming and lead alignment:
   - Detect local machine IP (`192.168.1.3` from `gitmap ip`) and assign `w1` as `Lead` and `Local` node.
   - Sort `w1` with top primacy in `sortFleetMachines`.
   - Map `main` as remote worker node (`192.168.1.20`).
3. Cross-machine prompt sending in Shared Cluster section:
   - Add prompt dispatch action in `FleetMachineRow.tsx` so users can send prompts to instances on other machines.
