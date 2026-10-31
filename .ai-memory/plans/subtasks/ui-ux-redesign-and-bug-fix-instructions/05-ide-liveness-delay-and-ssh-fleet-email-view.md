# Subtask 05: IDE Liveness Delay Gate & SSH Fleet Account Email Focus

**File**: `.ai-memory/plans/subtasks/ui-ux-redesign-and-bug-fix-instructions/05-ide-liveness-delay-and-ssh-fleet-email-view.md`  
**Parent Plan**: `.ai-memory/plans/ui-ux-redesign-and-bug-fix-instructions.md`  
**Status**: Pending  

## Objectives
1. IDE Liveness & 5-Second Delay Gate:
   - When user clicks `Send` (or resend) to dispatch a prompt to an IDE:
     - Check if target IDE is open. If not open, trigger launch.
     - Enforce a 5-second initialization delay with window/PID polling to ensure the IDE window is fully active and listening before injecting the prompt payload.
     - Guarantee prompts are not lost if IDE is closed at click time.
2. SSH Fleet Account/Email Focus:
   - In `src/components/instances/fleet/FleetMachineRow.tsx`:
     - Hide administrator usernames (`Administrator`, `root`, etc.) and passwords completely.
     - Display node alias, running instance count, and bound account emails prominently.
     - De-emphasize raw IP into the details popover.
     - Support cross-instance prompt transmission across connected nodes.
