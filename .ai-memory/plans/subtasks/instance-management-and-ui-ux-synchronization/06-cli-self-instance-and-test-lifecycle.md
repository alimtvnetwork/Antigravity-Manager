# Subtask 06: CLI Self-Instance Detection & Test Instance Lifecycle

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `06-cli-self-instance-and-test-lifecycle`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Implement CLI commands in `agm instance` to detect the currently running Antigravity IDE/agent instance (self-instance awareness) and provide lightweight, efficient test instance provisioning and teardown commands.

## Scope of Changes
- Add `agm instance current` / `agm instance self` command to detect:
  - Parent process tree to identify if caller is running inside Antigravity / VS Code.
  - Active `--user-data-dir` or workspace environment flags.
  - Return JSON and human-readable output identifying the self instance ID.
- Add safety checks to instance stop/kill commands: abort if target instance matches the self-instance ID unless `--force-self` is explicitly passed.
- Add `agm instance test-create --name <name>` and `agm instance test-destroy --name <name>` commands for rapid, zero-bloat test environments.
- Ensure all test instance creation and teardown updates the SQLite database immediately.

## Verification
- `agm instance self` accurately reports whether current shell is hosted inside an instance.
- Cannot kill own instance without explicit override flag.
- Test instances create in <500ms and destroy cleanly with zero disk leftover.
