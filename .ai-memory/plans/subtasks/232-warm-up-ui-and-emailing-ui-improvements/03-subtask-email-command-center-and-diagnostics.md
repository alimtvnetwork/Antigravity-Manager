# Subtask 03: Email Command Center, Health Diagnostics & Self-Healing

**Subtask ID:** `232-03`  
**Parent Task:** `232-warm-up-ui-and-emailing-ui-improvements`  
**Status:** `PENDING`  
**Improvements Covered:** Improvements 51–60, 81–90 (Clusters 6 & 9)  
**Target Specs:**  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/01-architecture-spec.md`  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/03-emailing-ui-improvements-spec.md`  

---

## Technical Scope & Deliverables
1. **Live Email Activity & Event Stream HUD (Improvement 51)**: Real-time event log streaming IMAP polls, outbound alerts, and ACK deliveries.
2. **Mailbox Connection Health Matrix (Improvement 52)**: Interactive card displaying RTT latency, TLS cipher suite, and error frequency.
3. **Adaptive Watcher Cadence Curve (Improvement 53)**: Visual graph showing dynamic switching between 60s idle and 5s active awaiting intervals.
4. **Machine Node & Multi-Instance Target Matrix (Improvement 54)**: Selector for routing commands to specific nodes or instances (`[node] | [instance]`).
5. **Two-Phase Delivery Receipt Inspector (Improvement 55)**: Side-by-side inspector for Phase 1 ACK and Phase 2 Result receipts.
6. **Unified Quota & Rate Limit Gauge Bar (Improvement 56)**: Real-time progress bar tracking daily SMTP limits and 10s debounce stack.
7. **Inbound Mailbox Unread Inspector (Improvement 57)**: Drawer listing unread subject headers directly from IMAP with purge controls.
8. **Automated Failover Circuit Breaker (Improvement 58)**: Visual toggles for mailbox failover pool and retry circuits.
9. **5-Stage Diagnostic Step Ladder Visualizer (Improvement 81)**: Step-by-step ladder (DNS -> TCP -> TLS -> Auth -> Ping) with per-stage latency and error detail.
10. **DNS MX & Port Auto-Discovery (Improvement 82)**: Automatic MX and SRV resolution from user email domain.
11. **TLS Certificate Inspector & Port Probe (Improvements 83, 84)**: Protocol version and alternative port probe (587, 465, 25, 2525).
12. **Corrupted Mailbox Self-Healing (Improvement 87)**: Safe message quarantine preventing parser crashes.

---

## File Boundaries
- `Exclusive Files`:
  - `src/components/email/EmailCommandCenter.tsx`
  - `src/components/email/DiagnosticLadder/DiagnosticLadderModal.tsx`
  - `src/components/email/MailboxHealthMatrix.tsx`
  - `src-tauri/src/commands/email/conn_test.rs`
- `Read-Only Context Files`:
  - `src/components/settings/EmailNotificationSettings.tsx`
  - `src-tauri/src/modules/email_watcher/`
  - `src-tauri/src/modules/notification_hub/`

---

## Acceptance Criteria
- [ ] 5-stage diagnostic ladder reports discrete pass/fail status per stage.
- [ ] Adaptive cadence curve reflects active reply awaiting states.
- [ ] Inbound mailbox unread inspector displays headers without marking them read.
