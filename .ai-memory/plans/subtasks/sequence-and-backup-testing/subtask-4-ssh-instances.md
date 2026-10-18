# Subtask 4: Two-Machine SSH Connectivity & Instance-Account Binding Verification

## Objective
Verify multi-machine cluster integration across two machines (`w1` local, `w2` remote):
1. Verify SSH configuration and connectivity between `w1` and `w2`.
2. Determine how many instances are running on each machine.
3. Determine which Google account email is connected to each instance.
4. Ensure the UI and CLI display genuine instances and bound emails accurately.

## Files Involved
- `src-tauri/src/modules/ssh_manager/connections.rs`
- `src-tauri/src/modules/ssh_manager/exec.rs`
- `src-tauri/src/modules/ssh_manager/cluster.rs`
- `src/components/settings/EmailNotifications/SshGitmapSection.tsx`

## Verification Strategy
- Query GitMap cluster nodes (`gitmap cluster nodes --json`).
- Query SSH manager connections and test connectivity to `w1` and `w2`.
- Query `get_cluster_instances_and_balances` and verify returned instances and emails.
- Assert that neither machine shows duplicate records or false 100% quotas when offline.
