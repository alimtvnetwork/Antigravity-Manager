# Subtask 04: Supabase-Free Fleet Identity & Unified Telemetry Header

## Metadata
- **Subtask ID:** `04`
- **Slug:** `04-supabase-free-fleet-identity-and-telemetry-header`
- **Parent Plan:** `.ai-memory/plans/telegram-chat-enhancement-and-integration-solutions.md`
- **Status:** Pending
- **Owner:** Antigravity AI
- **Target Subsystems:**
  - `src-tauri/src/modules/telegram_inbound/reports_a.rs`
  - `src-tauri/src/modules/email_inbound/parse_legacy.rs`
  - `src-tauri/src/modules/email_inbound/parse_pipe.rs`
  - `src-tauri/src/modules/email_inbound/handlers_b.rs`
  - `src-tauri/src/modules/notification_hub/email_alerts.rs`

---

## 1. Objectives & Scope
Provide complete multi-machine, instance, and account identity visibility when operating without Supabase PostgreSQL sync. Users operating AGM across desktop machines, remote compute boxes, and CI runners must be able to instantly query and identify node topology via Telegram bot commands (`/fleet`, `/whoami`, `/instances`, `/status`, `/accounts`) and inbound Email commands (`FLEET`, `STATUS`), with all communications standardized around a unified Telemetry Header.

---

## 2. Target Files & Line Numbers

1. **`src-tauri/src/modules/telegram_inbound/reports_a.rs`**
   - Lines 319-392: `format_fleet_identity_report()`
   - Lines 394-428: `format_instances_report()`
   - Lines 430-482: `format_accounts_report()`
2. **`src-tauri/src/modules/email_inbound/parse_legacy.rs`**
   - Lines 139-144: Inbound subject parser for status and queries
3. **`src-tauri/src/modules/email_inbound/parse_pipe.rs`**
   - Lines 176-180: Pipe grammar parsing for `status` and `fleet`
4. **`src-tauri/src/modules/email_inbound/handlers_b.rs`**
   - Lines 238-316: `handle_status_query`
5. **`src-tauri/src/modules/notification_hub/email_alerts.rs`**
   - Lines 98-102: Email alert subject tuple generation
   - Lines 210-265: HTML email body topology card

---

## 3. Specifications & Invariants

### 3.1 Standardized Telemetry Header
```text
🖥️ Machine: <hostname> (<ip>) | 🏷️ Instance: <instance_id> | 👤 Bound: <email>
```
- **Hostname Resolution**: Checks `COMPUTERNAME` (Windows), `HOSTNAME` (Linux/macOS), falling back to `email_watcher::detect_machine_name()`.
- **IP Resolution**: `email_watcher::detect_local_ip()`.
- **Instance ID**: Active instance name and ID from `instance::load_registry()`.
- **Bound Account**: Active bound email address from instance configuration.

### 3.2 Inbound Telegram Reports Enhancements
1. **`/fleet` / `/whoami` / `/machines`**:
   - Monospace Telemetry Header.
   - Instance state (`🟢 RUNNING` with live OS PIDs from `find_pids_for_data_dir()`, or `⚪ IDLE`).
   - Active account dual-window quota balances (`4h` and `weekly`).
   - Counts of total local instances and accounts.
2. **`/instances`**:
   - Formatted list of all sandbox profiles.
   - Visual status emojis (`🟢` / `⚪`).
   - Data directories and bound account emails.
3. **`/accounts`**:
   - List of all accounts with subscription tiers (`PRO`, `FREE`, `ULTRA`).
   - 4-hour rolling and weekly quota percentages via `auto_switcher::extract_dual_window_quotas`.

### 3.3 Email Command Grammar Extensions
1. **`parse_legacy.rs`**:
   Recognize `fleet`, `whoami`, `cluster`, `machines`, `agm fleet`, `agm whoami` alongside `status`.
2. **`parse_pipe.rs`**:
   Support pipe-delimited targets `* | fleet`, `<node> | whoami`, `default | status`.
3. **`handlers_b.rs` (`handle_status_query`)**:
   Format the output report to include the standardized Telemetry Header, instance running PIDs, and split-DB health.

### 3.4 Email Alert Subject Standard
Update `email_alerts.rs` subject formatting:
```rust
let subject = format!(
    "[AGM {} | {} ({}) | {} | {}] Account Switched: {} -> {}",
    pkg_ver, m_name, m_ip, details.instance_id, selected_display, from_display, selected_display
);
```

---

## 4. Acceptance Criteria

- [ ] Sending `/fleet`, `/whoami`, or `/machines` in Telegram returns the Standalone Fleet Identity card.
- [ ] Live PIDs are resolved and formatted for the active instance data directory.
- [ ] Quotas display both 4h rolling and 7-day weekly balances.
- [ ] Email commands with subject `FLEET` or `* | fleet` parse into `InboundAction::StatusQuery`.
- [ ] Email receipts include the canonical Telemetry Header.
- [ ] Email switch notification subjects contain the complete machine/instance/account tuple.
- [ ] Zero negative booleans; all variables follow positive naming conventions (`is_running`, `is_active`, `has_match`).

---

## 5. Verification Steps

1. **Unit Tests**:
   - Execute email inbound parsing tests:
     ```bash
     cd src-tauri && cargo test modules::email_inbound::parse_legacy
     cd src-tauri && cargo test modules::email_inbound::parse_pipe
     ```
   - Verify `parse_single_command_string("fleet", "")` returns `InboundAction::StatusQuery`.
   - Verify `parse_single_command_string("* | whoami", "")` returns `InboundAction::StatusQuery`.
2. **Preflight Checks**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   cd src-tauri && cargo fmt -- --check
   ```
