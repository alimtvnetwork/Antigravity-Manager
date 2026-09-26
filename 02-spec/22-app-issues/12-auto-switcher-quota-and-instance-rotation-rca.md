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

---

## 2. Cause

1. **First-Match Short-Circuit in Quota Calculation**:
   - In [`src-tauri/src/modules/auto_switcher.rs`](../../src-tauri/src/modules/auto_switcher.rs), `calculate_account_quota` and `evaluate_account_period_status` returned the first model matching `target` or `"flash"` instead of computing the **minimum** percentage across matching models and checking `quota_data.quota_groups` (immediate/hourly and weekly buckets) as well as any depleted active model in `quota_data.models`.
2. **Standby Candidate Rejection Under High Test Thresholds**:
   - In `select_next_best_profile`, standby accounts without cached quota defaulted to `.unwrap_or(0.0)` instead of `.unwrap_or(100.0)`, and no fallback existed when testing with a high threshold like `98%` where all accounts might be at `90%–97%`.
3. **Unresponsive Sleep Interval on Config Change**:
   - `start_auto_switcher` called `tokio::time::sleep(Duration::from_secs(interval_secs))` in a single block (up to 300 seconds) without polling in short ticks for threshold or enablement changes.
4. **Incomplete Switch Telemetry & CLI Email Dispatch**:
   - `notification_hub::dispatch_email_switch_alert` hardcoded `"old_email": ""` instead of resolving the previous account email prior to credential injection.
   - `agm email status` and `agm email help` only printed to stdout without dispatching the status/help email to configured recipients.
   - `agm recreate-project` / `agm recreate` did not prune matching conversation `.db` and `brain/<cid>` directories or seed the `"read all files and memory to understand the project"` bootstrap conversation prompt.

---

## 3. Fix

1. **Minimum-Bottleneck Quota & Bucket Evaluation (`src-tauri/src/modules/auto_switcher.rs`)**:
   - Updated `calculate_account_quota` and `evaluate_account_period_status` to take the **minimum** percentage across matching target models, minimum non-banned model percentage when any model is active/consumed, and minimum `remaining_fraction * 100.0` across `quota_data.quota_groups` buckets.
2. **Standby Candidate Default & High-Threshold Fallback (`src-tauri/src/modules/auto_switcher.rs`)**:
   - Updated `select_next_best_profile` to treat unqueried standby accounts as `100.0%` (`unwrap_or(100.0)`) and added a fallback that selects the highest-scoring candidate with `quota > 15.0` when testing with a high threshold where no candidate exceeds `threshold`.
3. **Reactive 5-Second Tick Loop (`src-tauri/src/modules/auto_switcher.rs`)**:
   - Updated `start_auto_switcher` to run an immediate check on startup and sleep in 5-second ticks, waking up immediately whenever `is_enabled` is toggled on or `low_quota_threshold_percent` is modified in the UI.
4. **Complete Switch Telemetry JSON & CLI Enhancements (`src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/bin/agm.rs`)**:
   - Resolved `old_email` and `instance_mode` (`"default"` vs `"isolated"`) before switching and populated `"old_email"`, `"new_email"`, `"instance_mode"`, `"switch_mode"`, `"condition"`, `"agm_version"`, `"vm_name"`, and `"local_ip"` in the `[JSON]` switch notification email.
   - Updated `agm email status` and `agm email help` to dispatch HTML status/help emails in addition to terminal output.
   - Updated `agm instances create` to clone default user data via `copy_instance` and clone the IDE binary unless `--data-only` (`do`) is passed.
   - Updated `agm recreate-project` and `agm recreate` to support comma-separated `<seq|id|alias|path>` targets, purge conversation `.db` and `brain/<cid>` caches alongside `workspaceStorage`, and seed `"read all files and memory to understand the project"` on relaunch.

---

## 4. Prevention

- Always evaluate multi-model and multi-bucket quota structures using minimum-bottleneck aggregation (`f64::min`) rather than first-match iteration (`break`).
- Ensure background daemons poll configuration changes in bounded 5-second ticks so UI slider adjustments take effect immediately without requiring application restarts.
- Preserve pre-mutation state (`old_email`, `instance_mode`) before executing destructive or state-mutating operations so telemetry payloads remain complete and auditable.
