# Non-CI/CD Root Cause Analysis (RCA): Startup Auto-Switch Delay & Telemetry Deficiencies

**Issue Reference:** 08-startup-auto-switch-and-telemetry-gaps  
**Severity:** High  
**Status:** In Progress / Remediating  
**Category:** Business Logic, Quota Management, Telemetry & Notification Hub  
**Affected Modules:** `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/lib.rs`  

---

## 1. Reproduction & Symptom Summary

When the Antigravity Manager desktop application is launched:
1. **Startup Auto-Switch Bypass / Delay**:
   - The application starts up and displays accounts with their quotas. An active or currently bound account that is completely depleted (e.g. `0%` remaining quota with a 3-hour cooldown) remains bound in the IDE.
   - The auto-switcher daemon is registered in `setup()` with a 3-second initial sleep followed by a loop that checks only periodically. If `auto_profile_switcher.is_enabled` was not evaluated immediately during application bootstrap, the user is left on a depleted account, encountering quota exhaustion errors when issuing AI queries immediately after opening the application.
2. **Incomplete Email Notification Telemetry**:
   - When a manual or auto switch occurred, the dispatch email displayed only minimal fields (`Version`, `Target Account`, `Target Instance`, `Trigger Mode`, `Reason`, `Origin Node`).
   - The email omitted:
     - The previous account's email address.
     - The previous account's remaining credit breakdown across both the **4-hour window** and the **weekly window**.
     - The newly selected target account's credit breakdown across both the **4-hour window** and the **weekly window**.
     - Convenient click-to-copy code blocks for field values.
3. **Missing Cross-Instance Self-Broadcast**:
   - When an account was bound, other instances running across different VMs or IP addresses had no decentralized, low-latency method (via mailbox filtering of the last 1 hour) to detect that the account was actively claimed, risking concurrent quota contention.

---

## 2. Root Cause Analysis (Why & How)

### 2.1 Why Startup Did Not Immediately Switch Depleted Accounts
- In `src-tauri/src/modules/auto_switcher.rs`:
  ```rust
  pub fn start_auto_switcher() {
      tauri::async_runtime::spawn(async move {
          // initial check delayed by 3s or interval_seconds
          loop {
              tokio::time::sleep(Duration::from_secs(interval)).await;
              check_and_rotate_profiles(false, None).await;
          }
      });
  }
  ```
  - Quota data refreshed on startup is loaded asynchronously in `server.rs` / `account.rs`, but `check_and_rotate_profiles` was never invoked synchronously or immediately upon completion of the startup quota evaluation.
  - Furthermore, `check_and_rotate_profiles` relies on `monitored_instances`, which only checked running process instances. When the user opens the application before launching the IDE, the default bound account (`#1 Default`) is loaded, but if no IDE instance was actively running, the auto-switcher returned early (`monitored_instances.is_empty() -> return Ok(None)`), failing to rotate the primary default workspace profile even when it was at 0%.

### 2.2 Why Previous & Dual-Window Quota Balances Were Omitted
- In `src-tauri/src/modules/notification_hub.rs`, the `SwitchNotificationDetails` struct only stored:
  ```rust
  pub struct SwitchNotificationDetails {
      pub previous_email: Option<String>,
      pub predicted_next_email: Option<String>,
      pub selected_email: String,
      pub credit_before_switch: Option<f64>, // Single scalar, not separated into 4h vs weekly
      ...
  }
  ```
  - Google Cloud Code / Gemini Enterprise quotas operate on two distinct temporal sliding windows:
    1. **4-Hour Rapid Recovery Window** (per-period rate limits)
    2. **Weekly Total Allowance Window** (7-day overall budget)
  - The switch payload only captured a single aggregated float `credit_before_switch`, completely omitting the 4-hour percentage and weekly percentage for both the departing account and the incoming target account.

### 2.3 Why Email lacked Copyable Blocks and Machine Self-Broadcast
- The existing HTML email template generated plain table cells without styled `<pre><code>` click-to-copy or selectable code containers.
- There was no function to emit a pure JSON self-broadcast email to `self` with a structured, indexable subject line `[Antigravity | IN-USE | <node> | <ip>] <email> (expires in 1h)` for downstream automated instance arbiters.

---

## 3. Targeted Fix Architecture

1. **Immediate Startup Auto-Switch Sweep**:
   - Introduce `evaluate_and_execute_startup_rotation()`.
   - On startup (and immediately upon quota refresh completion), check both running IDE instances AND the primary default workspace account.
   - If the active account has depleted credit (`<= critical_threshold` or `0.0%`), immediately execute rotation to the highest-credit candidate, update IDE state, and dispatch notifications.
2. **Dual-Window (4-Hour & Weekly) Quota Telemetry**:
   - Update `SwitchNotificationDetails` to capture:
     - `previous_email: Option<String>`
     - `previous_quota_4h: Option<f64>`
     - `previous_quota_weekly: Option<f64>`
     - `target_email: String`
     - `target_quota_4h: Option<f64>`
     - `target_quota_weekly: Option<f64>`
   - Extract both windows directly from `account.quota` using `QuotaBucket.window` matching (`"4h"`, `"four_hours"`, `"weekly"`).
3. **Enhanced HTML Email with Selectable Monospace Copy Blocks**:
   - Provide field-by-field copy blocks with dark monospace styling and clear border outlines compatible with Gmail, Outlook, Apple Mail, and web clients.
4. **Machine-Readable JSON Self-Broadcast Email**:
   - When any switch occurs, dispatch a dedicated plain-text JSON email to the active mailbox:
     - **Subject:** `[Antigravity | IN-USE | <hostname> | <local_ip>] <account_email> (1h lease)`
     - **Body:** Pure formatted JSON containing `account_email`, `node_name`, `node_ip`, `switched_at_utc`, `lease_expires_at_utc`, `previous_email`, `quota_4h`, `quota_weekly`, `reason`.
   - Allows external scripts, CLI daemons, or sibling instances to query the mailbox for messages received within the last 1 hour and exclude active accounts.
5. **Telegram Alert Parity**:
   - Format Telegram messages with clear HTML `<code>` copyable tags for both accounts and their respective 4-hour and weekly balances.

---

## 4. Prevention & Quality Invariants

- **Invariant 1**: Depleted accounts (`0%` quota) must never remain bound on startup if auto-switcher is enabled and healthier accounts exist.
- **Invariant 2**: All notification dispatches (Email & Telegram) must always include both 4-hour and weekly quota balances for both prior and target accounts.
- **Invariant 3**: Self-broadcast emails must strictly contain valid JSON and zero HTML markup to ensure seamless parsing by external bots and peer instances.
