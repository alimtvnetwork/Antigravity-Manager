# Auto-Switcher 98% Threshold Evaluation, Candidate Fallback & Switch Telemetry RCA

**Version:** 1.0.0
**Updated:** 2026-09-26
**Severity:** Critical
**Status:** Fixed

---

## 1. Reproduction

1. Open Antigravity-Manager -> **Settings -> Auto-Switcher** and enable the Auto-Switcher toggle.
2. Adjust the **Low Quota Switch Threshold** slider to `98%` while the active account has consumed credits on its active model or immediate/weekly quota bucket (e.g., remaining quota is `95%` on an active model or `quota_groups` bucket, while an unused `gemini-*-flash` model variant in `quota.models` remains at `100%`), or run `agm switch-if-low-credit 98` (`agm swlc 98` / `agm sfc`).
3. Observe that:
   - `evaluate_account_period_status` and `calculate_account_quota` broke on the first model matching `"flash"` (returning `100.0%`) and ignored `quota_data.quota_groups` buckets, failing to recognize that an active model or immediate/weekly bucket had dropped below `98%`.
   - When standby accounts had no cached model array (`calculate_account_quota` returning `None`) or were also below an artificially high test threshold (`98%`), `select_next_best_profile` evaluated `None` as `0.0%` and rejected all standby accounts.
   - `start_auto_switcher` slept for the full `interval_secs` (up to 300s) before its first check and did not wake up early when the user changed `low_quota_threshold_percent` in the UI.
   - When a switch did occur, `dispatch_email_switch_alert` in `notification_hub.rs` emitted `"old_email": ""` instead of the previous account email and omitted `instance_mode` (`"default"` vs `"isolated"`) and `condition`.
   - In the React GUI (`src/components/common/BackgroundTaskRunner.tsx`), only `auto_refresh` and `auto_sync` were monitored — there was no `useEffect` monitoring `config?.auto_profile_switcher` (`is_enabled`, `low_quota_threshold_percent`), so adjusting the slider to `98%` never triggered the UI Smart Fast-Forward (`smartRotateProfileAccount`) action or updated the UI stores.
   - Switching the `default` instance via `commands::switch_account` (called by UI Smart Fast-Forward `>>`) did not invoke `notification_hub::record_previous_email` or `notification_hub::notify_account_switched`.
   - `instance::resolve_instance_id` did not resolve 1-based list positions (`#1`, `#2`) when `seq_num` was unset and silently fell back to `active_instance_id` on unknown specifiers.

---

## 2. Cause

1. **First-Match Short-Circuit in Quota Calculation & Unsaved Live Quota Fetch**:
   - In [`src-tauri/src/modules/auto_switcher.rs`](../../src-tauri/src/modules/auto_switcher.rs), `calculate_account_quota` and `evaluate_account_period_status` returned the first model matching `target` or `"flash"` instead of computing the **minimum** percentage across matching models and checking `quota_data.quota_groups` (immediate/hourly and weekly buckets) as well as any depleted active model in `quota_data.models`. Furthermore, `check_and_rotate_with_options` did not call `account::save_account(&bound_acc)` after `fetch_quota_with_retry`.
2. **Missing Frontend Auto-Switcher Effect for Smart Fast-Forward (`>>`)**:
   - [`src/components/common/BackgroundTaskRunner.tsx`](../../src/components/common/BackgroundTaskRunner.tsx) had no `useEffect` watching `config?.auto_profile_switcher`, so the UI never invoked `useInstanceStore.getState().smartRotateProfileAccount` when account quota dropped at or below `low_quota_threshold_percent` (e.g., `98%`).
3. **Standby Candidate Rejection Under High Test Thresholds**:
   - In `select_next_best_profile`, standby accounts without cached quota defaulted to `.unwrap_or(0.0)` instead of `.unwrap_or(100.0)`, and no fallback existed when testing with a high threshold like `98%` where all accounts might be at `90%–97%`.
4. **Unresponsive Sleep Interval on Config Change**:
   - `start_auto_switcher` called `tokio::time::sleep(Duration::from_secs(interval_secs))` in a single block (up to 300 seconds) without polling in short ticks for threshold or enablement changes, and `save_app_config` / `update_auto_switcher_config` did not trigger an immediate evaluation check.
5. **Incomplete Switch Telemetry & Instance Resolution Gaps**:
   - `notification_hub::dispatch_email_switch_alert` hardcoded `"old_email": ""` instead of resolving the previous account email prior to credential injection, and `commands::switch_account` did not trigger `notify_account_switched` when switching the `default` instance.
   - `instance::switch_account_to_instance` only wrote `device_profile` to `storage.json` for the default instance, and `instance::resolve_instance_id` lacked 1-based positional fallback and strict error return on unknown specifiers.

---

## 3. Fix

1. **Minimum-Bottleneck Quota, Persisted Live Quota & Reactive Check (`src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/instance.rs`)**:
   - Updated `calculate_account_quota` and `evaluate_account_period_status` to take the **minimum** percentage across matching target models, minimum non-banned model percentage when any model is active/consumed, and minimum `remaining_fraction * 100.0` across `quota_data.quota_groups` buckets.
   - Persisted `bound_acc` via `account::save_account(&bound_acc)` after `fetch_quota_with_retry` in `check_and_rotate_with_options`, and spawned an immediate `auto_switcher::check_and_rotate_if_needed()` task when `save_app_config` or `update_auto_switcher_config` is invoked with `is_enabled = true`.
2. **Frontend Auto-Switcher Smart Fast-Forward Hook (`src/components/common/BackgroundTaskRunner.tsx`)**:
   - Added a reactive `useEffect` in `BackgroundTaskRunner.tsx` watching `config?.auto_profile_switcher` (`is_enabled`, `low_quota_threshold_percent`, `critical_threshold_percent`, `check_interval_seconds`, `target_model`) that evaluates active instance quota immediately on threshold change and on interval ticks, invoking `useInstanceStore.getState().smartRotateProfileAccount` (Smart Fast-Forward `>>`) and syncing UI stores whenever remaining quota is `<= low_quota_threshold_percent`.
3. **Standby Candidate Default & High-Threshold Fallback (`src-tauri/src/modules/auto_switcher.rs`)**:
   - Updated `select_next_best_profile` to treat unqueried standby accounts as `100.0%` (`unwrap_or(100.0)`) and added a fallback that selects the highest-scoring candidate with `quota > 15.0` when testing with a high threshold where no candidate exceeds `threshold`.
4. **Complete Switch Telemetry JSON, Instance Isolation & CLI Enhancements (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/bin/agm.rs`)**:
   - Wired `record_previous_email` and `notify_account_switched` into both `commands::switch_account` (default instance UI switch / Smart Fast-Forward) and `instance::switch_account_to_instance` (isolated instance switch).
   - Seeded default `User/settings.json` on `create_instance`, wrote `device_profile` to `storage.json` for isolated instances, and fixed `resolve_instance_id` to support 1-based positional index (`#1`, `#2`) and return `Err` on unmatched specifiers.

---

## 4. Prevention

- Always evaluate multi-model and multi-bucket quota structures using minimum-bottleneck aggregation (`f64::min`) rather than first-match iteration (`break`), and persist refreshed account quota to disk immediately.
- Pair backend auto-switcher daemons with frontend store synchronization (`BackgroundTaskRunner.tsx`) so UI sliders (`98%` threshold) immediately trigger Smart Fast-Forward (`>>`) and reflect switched accounts in real time.
- Preserve pre-mutation state (`old_email`, `instance_mode`) before executing destructive or state-mutating operations across both default and isolated instance switch paths.
