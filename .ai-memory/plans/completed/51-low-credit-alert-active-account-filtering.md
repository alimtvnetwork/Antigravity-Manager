# PLAN-051: Low Credit Alert Active Account Filtering & Quota Drop Deduplication

## Status: COMPLETED
**Date**: 2026-09-27
**Target Version**: v4.77.1
**Severity**: High (Unwanted alert email spam for inactive profiles)

---

## 1. Problem Statement
Operators received "Low Quota Warning" emails (e.g. for `james.riseup.tech@gmail.com` at 11.0%) even though that profile was inactive, not currently in use, and not bound to any running workspace. The developer was actively working with a different primary profile (`marufssp@gmail.com` at 80% quota).

---

## 2. Root Cause
1. `check_quota_drop_sensor` in `email_watcher.rs` executed `for acc in accounts` returned by `account::list_accounts()`, blindly inspecting all 35 stored profiles.
2. Inactive accounts with old/cached quota percentages below threshold triggered alert emails claiming to be "The active profile".
3. The sensor ran even if Antigravity IDE, isolated instances, and proxy services were completely closed.
4. There was no deduplication state tracking the last alerted percentage, re-alerting identically on every expired cooldown window.

---

## 3. Implementation Details
1. **`src-tauri/src/modules/auto_switcher.rs`**:
   - Enhanced `get_active_in_use_account_ids()` to always include `account::get_current_account_id()`.
   - Implemented `is_antigravity_or_instance_running(Option<&str>) -> bool` to verify active processes, running instances, repo projects, active prompt queue, and local proxy.
   - Implemented `is_account_in_use(&Account) -> bool` to check if an account matches the primary active profile or is bound to a running/active instance.
2. **`src-tauri/src/modules/email_watcher.rs`**:
   - Gated `check_quota_drop_sensor` with `is_antigravity_or_instance_running(None)`.
   - Filtered candidate accounts with `if !is_account_in_use(&acc) { continue; }`.
   - Added thread-safe deduplication cache `LAST_QUOTA_ALERTED_PERCENT`. Only alerts if not alerted yet or if quota drops by >= 1.0% further. Clears on quota recovery.
3. **`src-tauri/src/modules/email_vault_db.rs` & `EmailNotificationSettings.tsx`**:
   - Standardized default `quota_drop_threshold_percent` from 15 to 25.
4. **Specification & Documentation**:
   - Created RCA specification `02-spec/22-app-issues/14-low-credit-alert-active-account-filtering-rca.md`.
   - Updated `02-spec/22-app-issues/01-index.md`.

---

## 4. Verification Results
- `modules::auto_switcher::tests::test_is_account_in_use_logic` -> PASS
- `modules::email_watcher::tests::test_quota_drop_deduplication_tracking` -> PASS
- `modules::email_watcher::tests::test_detect_machine_and_ip` -> PASS
- `cargo fmt -- --check` -> PASS
- `cargo clippy --bin agm --lib` -> PASS
- `npm run build` -> PASS
