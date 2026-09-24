# Subtask 01: Fix 5-Hour Quota Calculation & Multi-Click UI Vanishing
Traceability ID: Task-01, Task-02
Spec Reference: [02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md](../../../02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md)
Target Files: src-tauri/src/modules/quota.rs, src/utils/quotaDisplay.ts, src/pages/Accounts.tsx, src/components/accounts/AccountTable.tsx
Action:
- Restore canonical 5H bucket logic in `quota.rs`: return 5h bucket `Some(h)` whenever weekly is not exhausted (`> 0.001`).
- Fix `quotaDisplay.ts` so 5h remaining fraction and reset time are used for 5h window.
- Remove redundant triple-call `await refreshQuota` in `Accounts.tsx`.
- Prevent table row disappearance on rapid clicks by debouncing and guarding state updates.
Acceptance Criteria:
- 5H quota shows rolling hours/minutes (e.g. `4h 56m 100%`) instead of days (`6d 9h`).
- Rapid clicking does not collapse the UI or cause blanking.
Targeted Verification: git diff check and cargo check.
