# Subtask 02: Backend Candidate Selection & Live Verification Refactor

- [x] In `src-tauri/src/modules/auto_switcher.rs`:
  - [x] Add check for `acc.proxy_disabled` alongside `acc.disabled || acc.validation_blocked`.
  - [x] Eliminate 15% fallback pass in `select_candidate_profiles`.
  - [x] Require `quota >= 100.0 || (is_period_finished && quota >= threshold)`.
  - [x] In `select_and_verify_next_best_profile`, await `workspace_lease_manager::list_active_leases()`.
  - [x] Live refresh candidates directly from Google API (`fetch_quota_with_retry`).
  - [x] Strictly reject any candidate with live 4h quota `< 100.0%`.
  - [x] Delete `verified_fallback` buffer in `select_and_verify_next_best_profile`.
- [x] In `src-tauri/src/commands/mod.rs`:
  - [x] Update `toggle_proxy_status` to ensure `index.json` is updated synchronously with the account file.
