# Subtask 03: Self-Instance Safety Guard, CLI Test Commands & E2E Test Suite

## Task Metadata
- **Code**: `Task-03`
- **Owner**: Worker 03
- **Status**: PENDING
- **Owned Files**:
  - `src-tauri/src/bin/agm/instance_routes_a.rs`
  - `src-tauri/src/bin/agm/instance_routes_b.rs`
  - `src-tauri/src/bin/agm/instance_dispatch.rs`
  - `scripts/test-instance-autoswitch-e2e.ps1`

## Requirements
1. **Self-Instance Caller Detection & Guard**:
   - In `close.rs` (`detect_caller_instance_id`):
     - Support cloned binaries (`exe_name.starts_with("antigravity") || proc_name.starts_with("antigravity")`).
     - Check `ANTIGRAVITY_APP_DATA_DIR`, `ANTIGRAVITY_APP_DIR`, `VSCODE_PID`.
   - Propagate `refuse_if_caller_instance` across:
     - `route_instances_rm` in `instance_routes_b.rs`
     - `route_instances_switch` in `instance_routes_b.rs`
     - `route_instances_ff` in `instance_routes_b.rs`
2. **CLI Test Instance Commands**:
   - Implement `agm instances current [--json]`: Outputs detected caller instance ID, PID, and caller status.
   - Implement `agm instances create-test [--name <name>] [--json]`: Creates transient data-only test instance in `< 50ms`.
   - Implement `agm instances rm-test [<id>] [--all] [--json]`: Kills any running test process, purges folder, and cleans database entries.
3. **E2E Testing Script**:
   - Ensure `scripts/test-instance-autoswitch-e2e.ps1` runs safely:
     - Wraps calls with a 30s timeout.
     - Verifies caller detection.
     - Creates test instance, tests auto-switcher / stop, verifies PID termination, and cleans up.
