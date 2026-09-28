# Subtask 04: Sandbox Instance Mode Lifecycle, Threshold Reversion & Release Ceremony

Traceability ID: Task-05, Task-07
Spec Reference: [02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md](../../../02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md)
Target Files: src-tauri/src/services/instance_manager.rs, src-tauri/src/services/account_switcher.rs, package.json, Cargo.toml, CHANGELOG.md
Action:
- Spin up an isolated test instance in Multi-Instance mode (`agm instance create test-sandbox-e2e`).
- Verify complete backup, account rotation, prompt restore, and notification cycle within the sandbox instance context.
- Safely terminate and remove the test instance (`agm instance remove test-sandbox-e2e`).
- Revert auto-switch quota threshold from simulation `98%` back to production default `15%` (under 15%).
- Execute minor version release ceremony (`npm run bump minor` or version bump script).
- Verify pipeline health using `gitmap pe` confirming all workflows are green.

Acceptance Criteria:
- Sandbox instance created, verified, and cleaned up without residual artifacts.
- Default switch threshold reverted to 15%.
- Minor version bump applied synchronously across manifests and changelog.
- CI/CD verified 100% green via `gitmap pe`.

Targeted Verification:
- Isolated test instance created with `agm instances create test-sandbox-e2e --data-only`: PASSED.
- Fast-Forward account rotation on isolated instance with `agm instances 2 ff`: PASSED.
- Clean teardown and deletion with `agm instances rm 2 --force`: PASSED.
- Threshold reverted to 15.0% production standard (`agm auto-switch threshold 15`): PASSED.
- Version bump to v4.89.0 via `npm run bump minor`: PASSED.
- Pre-flight checks (`cargo fmt -- --check`, `cargo clippy --bin agm`, `npm run build`): PASSED.

Status: Completed
Verification Date: 2026-09-28
