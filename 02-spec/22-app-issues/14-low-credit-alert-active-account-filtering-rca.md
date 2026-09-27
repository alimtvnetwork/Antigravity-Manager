# Root Cause Analysis: Low Credit Alert Triggering on Inactive Profiles

## 1. Problem Classification & Symptom
- **Component**: Email Watcher Telemetry Daemon (`email_watcher.rs`, `check_quota_drop_sensor`) & Auto-Switcher In-Use Resolvers (`auto_switcher.rs`).
- **Symptom**: Operators received unexpected "Low Quota Warning" emails (e.g. `[v4.75.0 | W3 | 192.168.1.12] [AGM Alert] Low Quota Warning (11.0%) - james.riseup.tech@gmail.com`) for inactive accounts that were not currently in use, while actively working on a completely different primary profile (`marufssp@gmail.com`) with healthy quota (80%).
- **Severity**: MEDIUM (False positive alerting spam; alerts sent for inactive accounts).

---

## 2. Root Cause Analysis (4-Part RCA)

### 2.1 Direct Trigger
Every 10 minutes (`now - *last_quota > 600`), the background `email_watcher` daemon executed `check_quota_drop_sensor`. The sensor iterated across ALL accounts in the database via `for acc in accounts` returned by `account::list_accounts()`.

### 2.2 Blind Iteration Without Usage Discrimination
`check_quota_drop_sensor` did not verify whether an account was actually in active use. Any idle profile that had dropped below the configured threshold (15% or 25%) in a previous session or day remained in the account list with its stale/cached quota percentage. The sensor matched `pct < threshold && pct > 0.0` on that inactive account and dispatched a low quota alert email claiming "The active profile '<inactive_email>' has dropped...".

### 2.3 Absence of Active Service / Process Gate
The sensor executed even if Antigravity IDE, isolated instances, active projects, and proxy services were completely closed. Even if no developer activity or prompt execution was occurring on the system, the sensor evaluated cached quota values from disk.

### 2.4 Missing Quota Alert Deduplication
The sensor had no state cache tracking the last alerted quota percentage per account. Every cycle where cooldown expired, an account with low quota would re-trigger an identical alert email, causing mailbox notification flooding.

---

## 3. Remediation Strategy

1. **Active Usage Verification (`is_antigravity_or_instance_running`)**:
   - `check_quota_drop_sensor` first checks `is_antigravity_or_instance_running(None)`. If Antigravity IDE, running instances, running projects in `repo_db`, running prompts, or proxy services are not active, the sensor exits immediately.

2. **Strict In-Use Account Filtering (`is_account_in_use`)**:
   - `check_quota_drop_sensor` verifies `is_account_in_use(&acc)` for every candidate account.
   - An account is only in use if it matches `account::get_current_account_id()` / `account::get_current_account()`, or is bound to a running/active instance in `list_running_or_active_instances()`.
   - Inactive, unselected accounts in the list are immediately skipped (`continue`).

3. **Quota Alert Deduplication State Machine (`LAST_QUOTA_ALERTED_PERCENT`)**:
   - Maintains a thread-safe cache mapping `account_email -> last_alerted_percent`.
   - Suppresses duplicate alerts if the quota percentage has not dropped further by at least 1.0%.
   - Automatically purges cache entries when quota recovers above threshold (e.g. following a 4-hour reset window).

4. **Safety Threshold Standardization**:
   - Standardized default `quota_drop_threshold_percent` from 15% to 25% in `email_vault_db.rs` and `EmailNotificationSettings.tsx` to align with the canonical 25.0% low quota threshold.

---

## 4. Verification & Quality Gates
- **Unit Tests**:
  - `modules::auto_switcher::tests::test_is_account_in_use_logic` passed (1 passed, 0 failed).
  - `modules::email_watcher::tests::test_quota_drop_deduplication_tracking` passed (1 passed, 0 failed).
  - `modules::email_watcher::tests::test_detect_machine_and_ip` passed (1 passed, 0 failed).
- **Format & Linters**:
  - `cargo fmt -- --check`: Clean (0 diffs).
  - `cargo clippy --bin agm --lib`: Clean (0 errors).
  - `npm run build`: Clean (built in 17.36s, 0 errors).
