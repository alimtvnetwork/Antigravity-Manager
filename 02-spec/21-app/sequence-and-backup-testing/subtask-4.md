# Subtask 4: Two-Machine SSH Connectivity & Instance-Account Binding Verification

## 1. Context & Invariants
- **Target**: `src-tauri/src/modules/ssh_manager/`, `src/components/settings/EmailNotifications/SshGitmapSection.tsx`.
- **Core Invariant**:
  - Two machines (`w1` local, `w2` remote) must be discoverable and queryable via SSH / GitMap cluster nodes.
  - Telemetry must report:
    - Number of running instances on each machine.
    - Bound Google account email for each instance (or `unbound`).
  - Zero false 100% quotas on unreachable nodes (fallback strictly to 0.0%).
  - Local node must not be duplicated or excluded due to IP mismatch.

## 2. Testing Procedure
1. Query GitMap cluster nodes:
   `gitmap cluster nodes --json`
2. Test SSH connectivity to `w1` and `w2`:
   `agm ssh test w1`, `agm ssh test w2` or equivalent probe.
3. Query `get_cluster_instances_and_balances`:
   Verify returned JSON items for `w1` and `w2` contain `machine_alias`, `instance_id`, `instance_name`, `bound_email`, and `running`.
4. Validate that UI dropdowns in `SshGitmapSection.tsx` list genuine instances and emails.
