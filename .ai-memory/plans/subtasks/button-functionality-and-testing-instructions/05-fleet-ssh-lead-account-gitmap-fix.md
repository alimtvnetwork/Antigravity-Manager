# Subtask 05: Fleet Nodes SSH & GitMap Account Linkage

**Subtask ID**: `05-fleet-ssh-lead-account-gitmap-fix`  
**Parent Plan**: `.ai-memory/plans/button-functionality-and-testing-instructions.md`  
**Status**: QUEUED  

## Intent
Audit and fix GitMap SSH node configuration and FleetNodesTable so that the lead / main node correctly displays assigned Google accounts and Git link rather than showing `Unassigned` or misaligned machine profiles.

## Files
- `src-tauri/src/modules/ssh_manager.rs`
- `src/components/fleet/FleetNodesTable.tsx`
