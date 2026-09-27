# Subtask 03: Frontend Store & CLI Invariant Alignment

- [x] In `src/stores/useInstanceStore.ts`:
  - [x] Force fresh account fetch before candidate selection (`await useAccountStore.getState().fetchAccounts()`).
  - [x] Include `Boolean(acc.proxy_disabled)` in `isDisabled` filter.
  - [x] Require `fresh4hQuota >= 100` in probe loop.
  - [x] Remove `bestFallbackCandidate` and `eligibleAccounts[0]` fallbacks; fail-safe error if no 100% account available.
- [x] In `src-tauri/src/bin/agm.rs`:
  - [x] Ensure CLI candidate selection accounts for `proxy_disabled` and 100% quota requirement.
