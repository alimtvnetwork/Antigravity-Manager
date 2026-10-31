# Subtask 06: Caller Preservation, Transient Test CLI & E2E Testing

## Objectives
1. Implement and verify caller process detection in `src-tauri/src/modules/instance/close.rs` via 24-level ancestor tree walk (`detect_caller_instance_id`).
2. Enforce `refuse_if_caller_instance` across all CLI commands (`agm instances stop`, `restart`, `delete`, `switch`, `rm-test`) returning exit code 2 unless `--force` is specified.
3. Verify transient test instance lifecycle commands:
   - `agm instances create-test --name <name>`
   - `agm instances rm-test --id <id>` / `agm instances rm-test --all`
4. Run comprehensive end-to-end test script `scripts/test-instance-lifecycle-e2e.ps1` to verify caller preservation, transient creation, stopping PID, switching, and audit logging.

## Owned Files
- `src-tauri/src/bin/agm.rs`
- `src-tauri/src/bin/agm/instance_routes_b.rs`
- `scripts/test-instance-lifecycle-e2e.ps1`

## Verification
- `scripts/test-instance-lifecycle-e2e.ps1` runs and exits with code 0.
