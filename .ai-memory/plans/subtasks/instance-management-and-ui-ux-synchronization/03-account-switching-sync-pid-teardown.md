# Subtask 03: Account Switching Synchronization & Pre-Switch PID Teardown

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `03-account-switching-sync-pid-teardown`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Fix the critical synchronization defect where switching an instance's account fails when a process is currently running or PID is mismatched. Ensure the running process is terminated cleanly before account credentials/tokens are updated and the instance is reloaded.

## Scope of Changes
- Hook account switching command/action to first inspect active PID.
- If PID is running (or detected via conscious process check), terminate the process tree completely.
- Wait for profile file locks on `workspaceStorage` / SQLite storage to be completely released.
- Apply new account configuration / OAuth credentials to target profile.
- Relaunch instance or update state safely, preventing token collision and state corruption.

## Verification
- Account switch gracefully stops running instance, swaps credentials, and restarts without lock conflicts.
- Audit entry generated recording account transition.
