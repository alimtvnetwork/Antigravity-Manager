# Specification: Startup Auto-Switch Immediate Activation & Rich Telemetry Broadcast

**Spec ID:** 54-startup-auto-switch-and-rich-telemetry-broadcast  
**Parent Epic:** Account Automation & Notification Hub  
**Status:** In Progress / Implementing  
**Last Updated:** 2026-09-27  

---

## 1. User Request (Verbatim)

```text
Okay, so make sure that we don't have the email saved anywhere as a screenshot. So make sure of that. Make sure the emails are blurred out, okay, before saving. Always. And check if other images contains the emails. Maybe this email should blurred out. Okay? And do not keep any other history of these images. That's the first thing. Second, let's start with the problem. So whenever we open the antigravity manager, okay, the first thing I think we should have is that it's going to check the auto-switch condition, because there's a very good chance that it's in low credit and it's needed to switch. Okay? And it should also send an email if that it is found, switched, and everything. Okay? Remember that process. Startup always seeks for that activation. Okay? Just like it refreshes the UI, it will also check the activation, and if it matches, directly switch. Okay? And also give, let's say, a notification that it has switched, something like this. Yeah, that is very important. Okay. All right. After this happens, your next job is to check that other synchronization, like the Telegram. Telegram should also send the message if... I mean, what happened. Okay? So anytime you send an email, the email needs to have the following information, like why it switched, what was the previous account, what was the previous, let's say, percentage weekly and four-hour balance. And then the previous to the current one also needs to be in the email. So for example, I did a manual switch. The email that I received is not good enough. Okay? It needs to have more information, and also, I'm not sure if you can do that, the value fields. Can we have a copy button where I could copy things? Can you do that inside email? I don't know if that is possible or not. If possible, then all the email sections try to have a copy button for all these fields, actually. Okay. Field by field, I think it's better. So before/after account, switched accounts, current, let's say, four-hour balance and the weekly balance percentage both is required. And also, we send to the current email address, which is the main email and who is actually sending to itself that this email is in use by the VM IP. Everything will be in JSON format without any HTML. It will be sent to itself, so that, and also a subject to understand by other tools that you can read the email and see whoever is in use with other accounts right now, or let's say last one hour. So they will filter the email by last one hour by this format of the subject, like who switched, how switched. Okay. And based on that, it will not pick other instances, would not pick the same, let's say, account. Only if the email is added, but same thing can be done with Telegram. Same thing can be done with, let's say, Superbase. So when the Superbase is added, it should already do this checking, this type of logic. Do you understand the logic? First important thing is that you understand the logic. You write the logic to respect order, write the root cause analysis with the issues, why it happened, how it happened, and then you try to fix it. Is it clear? Do you understand it?
```

---

## 2. Architectural Overview & Requirements

### 2.1 Image Privacy & Sanitization
- **Strict Privacy Mandate**: Zero unblurred screenshots containing personal email addresses shall be committed or preserved in repository assets.
- If any reference images are saved to `assets/screenshots/`, any visible email identifiers must be blurred or redacted.

### 2.2 Proactive Startup Auto-Switch Activation
- When Antigravity Manager boots up and initializes the proxy / quota index, it must immediately evaluate the active bound account (both running instances and the `#1 Default` instance profile).
- If the current bound account has depleted or low credit (`<= critical_threshold` or `0%` quota with active cooldown):
  1. Trigger immediate fast-forward rotation to the highest-credit available candidate profile.
  2. Inject credentials into the IDE state storage.
  3. Dispatch notifications (Email & Telegram) marking the trigger mode as `Startup Auto-Switch (Low/Depleted Quota Detected)`.
  4. Broadcast the JSON in-use claim.

### 2.3 Rich Telemetry & Dual-Window (4-Hour + Weekly) Quota Extraction
- Every switch notification (both Manual and Automatic) must capture and present:
  - **Previous Account Email**: e.g., `previous_account@gmail.com` (or `(Standby / Initial)` if none).
  - **Previous Account 4-Hour Quota %**: e.g., `0.0% (Depleted / Cooling down: 3h 12m)`
  - **Previous Account Weekly Quota %**: e.g., `45.2%`
  - **Target (New) Account Email**: e.g., `target_account@gmail.com`
  - **Target Account 4-Hour Quota %**: e.g., `100.0%`
  - **Target Account Weekly Quota %**: e.g., `100.0%`
  - **Trigger Mode**: `Startup Auto-Switch`, `Periodic Auto-Switch`, or `Manual User Switch`.
  - **Reason**: Explaining why rotation happened (e.g. `Critical quota alert: model 'gemini-2.5-pro' dropped to 0.0%`).
  - **Node & Host IP**: Machine hostname and local IP.
  - **Field-by-Field Selectable / Copy Blocks**: Dark monospace `<pre style="..."><code>...</code></pre>` blocks formatted for single-click selection and easy copying across email clients (Gmail, Outlook, Apple Mail).

### 2.4 Machine-Readable JSON Self-Broadcast Email (1-Hour Lease Filter)
- Whenever an account switch or startup activation claims an account, a machine-readable email is dispatched to the active sender mailbox (`self`):
  - **Subject Format**:  
    `[Antigravity | IN-USE | <HOSTNAME> | <LOCAL_IP>] <ACCOUNT_EMAIL> (lease: 1h)`
  - **Body Content**: Raw JSON (Content-Type: `text/plain` or raw JSON without HTML tables) containing:
    ```json
    {
      "event": "account_in_use_lease",
      "version": "v4.80.0",
      "claimed_account": "user@example.com",
      "previous_account": "prev@example.com",
      "node_name": "W3",
      "node_ip": "192.168.1.12",
      "claimed_at_utc": "2026-09-27T07:15:00Z",
      "lease_expires_at_utc": "2026-09-27T08:15:00Z",
      "quota_4h_percent": 100.0,
      "quota_weekly_percent": 100.0,
      "trigger_mode": "Startup Auto-Switch",
      "reason": "Startup rotation from depleted account"
    }
    ```
  - Enables peer VM instances and CLI watchers to filter emails received within the last 1 hour, parse claimed accounts, and exclude them from candidate pools.

### 2.5 Telegram Switch Alert Parity
- Format Telegram messages with clear HTML `<code>` copyable tags for both accounts, previous and target 4-hour and weekly percentages, trigger mode, and reason.

---

## 3. Data Contracts & Struct Changes

### 3.1 `SwitchNotificationDetails` in `src-tauri/src/modules/notification_hub.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchNotificationDetails {
    pub previous_email: Option<String>,
    pub previous_quota_4h: Option<f64>,
    pub previous_quota_weekly: Option<f64>,
    pub predicted_next_email: Option<String>,
    pub selected_email: String,
    pub target_quota_4h: Option<f64>,
    pub target_quota_weekly: Option<f64>,
    pub credit_before_switch: Option<f64>,
    pub threshold_activated: Option<f64>,
    pub instance_id: String,
    pub instance_name: String,
    pub instance_mode: String,
    pub reason: String,
    pub is_auto: bool,
}
```

---

## 4. Verification Gates

1. **Gate 1: Startup Auto-Switch Verification**: On boot, an account with 0% quota must be rotated immediately to an account with >0% quota.
2. **Gate 2: Dual-Window Quota Telemetry**: Both Email and Telegram notifications must display 4h and weekly quota percentages for both previous and new accounts.
3. **Gate 3: Self-Broadcast JSON**: Outbound email includes a pure JSON payload to `self` with `[Antigravity | IN-USE | ...]` subject.
4. **Gate 4: Image Privacy**: Zero personal emails saved unblurred in repo assets.
