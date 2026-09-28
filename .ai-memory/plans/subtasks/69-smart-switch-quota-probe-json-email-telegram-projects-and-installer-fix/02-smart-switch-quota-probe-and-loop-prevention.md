# Subtask 02: Smart Switch Quota Probe & Loop Prevention

**Parent Plan**: Plan 69 (`02-spec/21-app/69-smart-switch-quota-probe-json-email-telegram-projects-and-installer-fix.md`)  
**Target Files**:
- `src-tauri/src/modules/auto_switcher.rs`
- `src-tauri/src/modules/account.rs`
- `src-tauri/src/modules/instance.rs`

## Requirements
1. **Live Google API Quota Probe**:
   - In `select_and_verify_next_best_profile`, every candidate MUST have `account::fetch_quota_with_retry` executed before selection.
   - Never trust stale cached quota.
2. **Strict 100% 4-Hour Quota Gate**:
   - If candidate's refreshed 4-hour window quota is `< 100.0%` (e.g. 11%, 20%, 99%):
     - Treat the account as completely exhausted (`0.0%`).
     - Strictly log rejection and advance to the next candidate.
     - NEVER select or fallback to `< 100.0%` accounts!
3. **Multi-VM Collision Shielding**:
   - Check Supabase distributed leases (`workspace_lease_manager::is_account_or_email_leased_by_other`).
   - Check inbound email broadcasts (`fetch_recent_cross_vm_switched_accounts(3600)`).
   - If held by another node, skip and continue.
4. **Looping Prevention & Disabled State Isolation**:
   - If an account is disabled (`cand_acc.disabled`, `cand_acc.proxy_disabled`, `cand_acc.validation_blocked`), strictly reject.
   - Do NOT bounce back to a previously depleted account (e.g. James when disabled).

## Status: COMPLETED

## Verification Results
- `src-tauri/src/modules/auto_switcher.rs`:
  - `calculate_4h_window_quota`: Checks `quota_data.is_forbidden` (returns 0.0), evaluates short-window buckets and all non-banned models (exhaustively checks 2.5-pro, 2.5-flash, etc.).
  - `select_candidate_profiles`: Unknown quota unwraps to 0.0 (never assumed 100%). Candidates strictly filtered for `quota >= 100.0 || is_period_finished`.
  - `select_and_verify_next_best_profile`: Refreshed quota defaults to 0.0, strictly rejects `< 100%` candidate with no fallback.
- `src-tauri/src/modules/account.rs`:
  - `switch_account` rejects disabled, proxy-disabled, or validation-blocked accounts.
  - Live Google API quota probe (`fetch_quota_with_retry`) before switch and saves refreshed quota to SQLite.
- `src/services/instanceService.ts` & `src/stores/useInstanceStore.ts`:
  - `extract4hWindowQuotaPercent` takes minimum across groups and active models.
  - Eliminated `|| accounts[0]` fallback to unverified profiles.
- Verified simulation test: `agm switch-if-low-credit -t 98` successfully rejected depleted account `james.riseup.tech@gmail.com` and rotated to 100% account `kinomono098@gmail.com`.

