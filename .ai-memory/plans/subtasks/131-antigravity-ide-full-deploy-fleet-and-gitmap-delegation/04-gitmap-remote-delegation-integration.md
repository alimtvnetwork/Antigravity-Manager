# Subtask 04: GitMap Remote Delegation Integration

## Deliverable:
Maintain and verify the GitMap delegation bridge:
- `scripts/gitmap-delegate-deploy.ps1`

## Capabilities:
- Single command remote delegation to any fleet node (`w1`, `w2`, `w3`, `u1`, `final-network-machine`):
  ```powershell
  powershell scripts/gitmap-delegate-deploy.ps1 -Target <node> -Preset turbo -Theme "Default Dark Modern"
  ```
- Bridges with GitMap CLI's `gitmap cluster run-script <target> <script>` and `gitmap agy deploy <target> --all`.
- Real-time streaming output and reachability checks.

## Verification:
- Script executes dry-run simulation against `localhost` and remote aliases.
- Exit code 0 on verified dispatch.
