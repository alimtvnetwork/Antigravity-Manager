# Canonical App Specification: Email Verification, Auto-Switcher Hardening, Training API, and Settings UI/UX Overhaul

## 1. User Request (Verbatim)

```text
Can you please confirm that if you have the email that is working fine, the command you can run, you can check, receive back. Can you please test it and confirm? Also confirm that the auto-switching is working. I don't think that you made it correctly. I still am in doubt. So make sure that you fix it properly. Another thing. Make sure that you can create, let's say, a REST API endpoint. I'm not sure if this feature is there, but you can actually, and also there will be a training API endpoint that anyone can seek into and read, and then based on that, it can send and learn and create a response and modify the machines. So can you please do that? That's another thing. Also, these can be switched on and off from the settings, and also the settings menu is very dirty. You don't have any UI/UX concept in the settings menu. I think you need to fix it. The Save button just say, "Save," and Backup just say, "Backup." You don't have to write the whole thing. Why you are writing Supabase Sync? Write Supabase, okay? Here, write Email-Alerts. Here, write Proxy, okay? So minimize this stuff. And also the Advance and Debug, just put it as a hamburger icon, a drop-down icon. If someone clicks, they'll see Advance and Debug option. And I said before, the debug should be crossed because you are adding the debug to the top level where our debug option is there. Put everything related to debug on that place. Do you understand?

Swap muiltiple agents to do it after?? clear???
```

## 2. Visual Reference & Screenshot Ingestion

User uploaded layout guidance screenshot:
`assets/screenshots/settings-ui-overhaul-01.png`

Annotated changes from screenshot:
- "Proxy Settings" tab -> Rename to **"Proxy"**
- "Email & Alerts" tab -> Rename to **"Email-Alerts"**
- "Supabase Sync" tab -> Rename to **"Supabase"**
- "Backup & Restore" button -> Rename to **"Backup"**
- "Save Settings" button -> Rename to **"Save"**
- "Debug" tab crossed out with red arrow pointing to top header debug bug icon -> Remove from primary tab row and consolidate into top header bug badge/modal or dropdown.
- "Advanced" & "Debug" grouped under a hamburger/dropdown trigger.

## 3. Architectural Design & Deliverables

### Deliverable 1: Inbound Email Polling & Reply E2E Test (Task-01)
- Verify `poll_unread_messages` IMAP socket reading with tagged responses (`read_imap_tagged_response`) to eliminate socket buffer desynchronization.
- Support 3-part command syntax: `<node> | <instance> | <command>` (e.g. `VM3 | 1 | help`).
- Ensure sender email normalization (`clean_recipient_email`) to prevent RFC 5321 501 syntax rejections on reply envelope `RCPT TO:<devorg.bd@gmail.com>`.
- Deliver live test proof via `agm test-email` verifying full execution and outbound delivery.

### Deliverable 2: Auto-Switcher Engine Verification & Hardening (Task-02)
- Inspect candidate selection formula:
  $$\text{Score} = \text{Tier Multiplier} \times (\text{Quota Remaining}) \times \text{Health Score}$$
- Enforce strict cooldown guards, pre-activation OAuth token refresh validation, and seamless active account switching.
- Verify through live CLI command `agm ff` / auto-switcher unit checks.

### Deliverable 3: Machine Training & Control REST API Endpoint (Task-03)
- Expose REST API routes on proxy/gateway (`/api/v1/training` or `/training`):
  - `GET /api/v1/training/telemetry`: Query active machine nodes, accounts, instances, token utilization, and health metrics.
  - `POST /api/v1/training/learn`: Ingest training feedback, prompt outcomes, and reinforcement signals.
  - `POST /api/v1/training/machines`: Remotely modify machine configuration, model route mappings, and active profiles.
- Guard with setting toggle (`is_training_api_enabled: bool`, default `false` or configurable) controllable via GUI and configuration.

### Deliverable 4: Settings Menu UI/UX Overhaul & Tab Minimization (Task-04)
- Minimalist, high-density settings navigation:
  - Tab 1: `General`
  - Tab 2: `Account`
  - Tab 3: `Proxy`
  - Tab 4: `Email-Alerts`
  - Tab 5: `Supabase`
  - Tab 6: `More` dropdown (Hamburger/Chevron icon) containing `Advanced` and `Debug Actions`.
  - Right Actions: `Backup` (icon + "Backup"), `Save` (icon + "Save").
- Remove redundant standalone `Debug` tab from main row; route all debug diagnostics through top header bug badge or the `More` dropdown menu.

## 4. Verification & Acceptance Criteria
- [ ] Email inbound parses `VM3 | 1 | help`, executes action, and sends threaded reply without SMTP 501 errors.
- [ ] Auto-switcher candidate scoring and rotation pass live verification without panics.
- [ ] REST training API endpoints respond with correct status codes when enabled and return 403/disabled when turned off.
- [ ] Settings tab labels strictly match: "Proxy", "Email-Alerts", "Supabase", "Backup", "Save".
- [ ] No compilation errors across Rust (`cargo check`) and TypeScript (`npm run build`).
