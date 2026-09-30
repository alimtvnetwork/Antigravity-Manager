# Subtask 01: Fix Instance Switching Logic Bug
Traceability ID: Task-01
Spec Reference: [02-spec/21-app/74-ui-email-telegram-fixes-revisit.md](../../../02-spec/21-app/74-ui-email-telegram-fixes-revisit.md)
Target Files: src-tauri/src/modules/instance.rs, src-tauri/src/bin/agm.rs
Action: Review `agm instances switch` and focus cross-contamination. Ensure `switch` properly handles the current instance ID instead of reverting to "worker alpha".
Acceptance Criteria: 
- `agm instances switch` targets the exact specified instance.
- No bleeding state between multiple running instances.
Targeted Verification: Verified exact string resolution and instance credential isolation in `instance.rs`.

Status: COMPLETED
Outcome: `resolve_instance_id` now strictly matches exact instance IDs/names and eliminates ambiguous substring or 1-based index regressions. `switch_account_to_instance` restricts credential synchronization to instance-local storage without touching the global user keyring.
