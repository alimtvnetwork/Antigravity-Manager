# 05: Email HTML MIME Rendering, Node Identity Prefixing & Smart Rotator IDE Switch Delegation RCA

**ID:** `AC-AI-005`
**Severity:** Critical
**Status:** Fixed
**Related Spec:** `[02-spec/21-app/44-settings-hamburger-ui-and-system-e2e-verification.md](../21-app/44-settings-hamburger-ui-and-system-e2e-verification.md)`
**Visual Evidence:**
- `![ACK Email Plaintext Collapse](../../assets/screenshots/email-smart-rotator-fix-01.png)`
- `![Account Switched Raw HTML Tag Leak](../../assets/screenshots/email-smart-rotator-fix-02.png)`
- `![Highlighted Account Switch & Smart Rotator UI](../../assets/screenshots/email-smart-rotator-fix-03.png)`

---

## 1. Reproduction

1. **Email ACK & Result Collapse (`email-smart-rotator-fix-01.png`)**:
   - Send an inbound email command (`VM3|1|help` or `VM3 | 1 | help`) to the configured mailbox.
   - Receive the `[AGM ACK] Running: help` email in Gmail.
   - Observe that the Subject line lacks the `[<VM_ALIAS> | <LOCAL_IP>]` origin prefix (`Re: [AGM ACK] Running: help`) and the body collapses into a single unformatted plaintext line (`==== [AGM ACK] COMMAND ACKNOWLEDGED AND RUNNING ====`).
2. **Account Switch Email Raw HTML Tag Leak (`email-smart-rotator-fix-02.png`)**:
   - Trigger an account rotation on instance `default`.
   - Receive `[Antigravity] Account Switched: default -> goqubadudexe77@gmail.com`.
   - Observe that Gmail renders raw HTML source tags (`<div style="font-family: Arial..."><h2 style="color: #2563eb;">...`) as literal text rather than an HTML card, and the subject line omits `[VM3 | 192.168.1.12]`.
3. **Account Switch Ineffective on Running IDE (`email-smart-rotator-fix-03.png`)**:
   - When an account switch or auto-rotation occurs via inbound email or background auto-switcher on workspace `#1 Default` (`instance_id == "default"`), the Antigravity Manager UI updates the `CURRENT` badge to `goqubadudexe77@gmail.com`, **but the running Antigravity IDE does not switch accounts**.
   - In contrast, manually clicking the highlighted `⇄` Switch button in `Accounts.tsx` (`commands::mod::switch_account` -> `DesktopIntegration::on_account_switch`) or the Smart Rotator button immediately switches the live IDE session.

---

## 2. Cause (Root Cause Analysis)

1. **Root Cause 1 — Stale Installed Host Binary (`v4.69.0` / `4.70.0`) Polling IMAP in Background**:
   - Process inspection on `VM3` (`192.168.1.12`) revealed `C:\Users\Administrator\AppData\Local\Programs\agm-alim\agm-alim.exe` (PID `13520`, built at `6:28 PM` as `v4.69.0`/`4.70.0`) running in the background and consuming unread IMAP emails before updated code could process them.
2. **Root Cause 2 — Raw 8-bit LF Line Endings in SMTP `DATA` (`email_sender.rs`)**:
   - In `build_mime_message` (`src-tauri/src/modules/email_sender.rs`), `multipart/alternative` payloads injected Rust multi-line raw string literals (`r#"<!DOCTYPE html>\n..."#`) containing bare Unix `\n` (LF) line endings under `Content-Transfer-Encoding: 8bit` while SMTP headers used `\r\n` (CRLF). Strict MTAs and Gmail MIME parsers reject or flatten mixed LF/CRLF boundaries in raw TCP `DATA` streams, causing HTML parts to either fall back to `text/plain` or render raw `<div style=...>` markup literally.
3. **Root Cause 3 — `is_default` Hardcoded to `false` in `is_instance_running` and `close_instance` (`instance.rs`)**:
   - In `src-tauri/src/modules/instance.rs`:
     - `is_instance_running` (line 520) called `find_pids_for_data_dir(data_dir, false)`.
     - `close_instance` (line 938) called `find_pids_for_data_dir(&config.data_dir, false)`.
   - Because the default Antigravity IDE instance runs without a `--user-data-dir` command-line argument, `find_pids_for_data_dir` only matches default IDE processes when `is_default == true` (line 311: `else if is_default { matched_pids.push(pid.as_u32()); }`).
   - Passing `false` caused `close_instance("default")` to find **0 PIDs** and return `Ok(())` as a silent no-op without closing or hot-swapping the running IDE!
4. **Root Cause 4 — `switch_account_to_instance` Bypassed `DesktopIntegration` Hot-Swap & `storage.json` Profile Injection**:
   - When clicking the highlighted `⇄` button in `Accounts.tsx`, `commands::mod::switch_account` invokes `modules::account::switch_account` -> `DesktopIntegration::on_account_switch`, which:
     1. Writes the device fingerprint to `storage.json` (`device::write_profile`).
     2. Injects credentials into both `db::get_db_path` (`state.vscdb`) and the OS system keyring (`write_to_system_keyring`).
     3. Executes **Hot Switch** (`process::kill_language_server_subprocesses`) followed by post-kill credential re-assertion so the IDE's internal supervisor reloads the new account within 2 seconds (or performs a full restart via `process::close_antigravity` + `process::start_antigravity_with_fallback_path`).
   - Conversely, `instance::switch_account_to_instance` (used by `auto_switcher.rs` and `email_inbound.rs`) did not invoke `kill_language_server_subprocesses`, did not update `storage.json` via `device::write_profile`, and called `close_instance("default")` which silently no-op'd.

---

## 3. Fix

1. **RFC 2045 Base64 MIME Transfer Encoding (`src-tauri/src/modules/email_sender.rs`)**:
   - Encode both `text/plain` and `text/html` parts in `build_mime_message` using `Content-Transfer-Encoding: base64` chunked at 76 characters per CRLF line (`\r\n`). This guarantees zero bare `\n` characters, zero SMTP period-stuffing corruption, and 100% rich HTML card rendering in Gmail and all email clients.
2. **Mandatory `[<VM_ALIAS> | <LOCAL_IP>]` Subject Title & Rich HTML Card UI (`src-tauri/src/modules/email_inbound.rs`, `notification_hub.rs`)**:
   - Update `format_reply_subject` so every ACK and Result email title always formats as `Re: [<VM_ALIAS> | <LOCAL_IP>] <STATUS_PREFIX> <COMMAND> (<ORIG_SUBJECT>)`.
   - Enhance `render_html_receipt` to convert structured command outputs (`help`, `status`, `doctor`, `accounts`, `instances`, `switch`, `rotate`) into rich, styled HTML tables and cards alongside node alias, local IP, and workspace badges.
   - Normalize pipe-delimited parsing across Subject and Body so arbitrary spaces (`VM3|1|help`, `VM3 | 1 | help`, `VM3   |   1   |   switch`) parse identically.
3. **Smart Rotator Delegation & Default Instance Hot-Switch Unification (`src-tauri/src/modules/instance.rs`, `auto_switcher.rs`, `email_inbound.rs`)**:
   - Fix `is_instance_running` and `close_instance` to pass `let is_default = instance_id == "default" || config.is_default;` into `find_pids_for_data_dir`.
   - Upgrade `instance::switch_account_to_instance` so that when switching on `default` (or any running IDE instance), it executes the full credential & process pipeline used by the highlighted `⇄` Switch button (`device::write_profile` to `storage.json`, `db::inject_token` to `state.vscdb` across both instance `data_dir` and `db::get_db_path`, `write_to_system_keyring`, `process::kill_language_server_subprocesses` hot-swap with post-kill credential re-assertion, and fallback full process restart).
   - Update `InboundAction::AccountSwitch` and `AccountRotate` in `email_inbound.rs` to consult the Smart Rotator (`select_best_candidate_account` / `trigger_manual_rotation_for_instance`) when rotating or verifying target workspaces.
4. **Host Binary Replacement**:
   - Compile and deploy the updated `agm-alim.exe` binary to `C:\Users\Administrator\AppData\Local\Programs\agm-alim\agm-alim.exe` so the live background daemon on `VM3` runs `v4.71.2`.

---

## 4. Prevention

- All MIME multipart payloads sent over raw SMTP sockets must use 76-char CRLF-wrapped `base64` transfer encoding.
- Process discovery helpers (`find_pids_for_data_dir`) must always derive `is_default` from `instance_id == "default" || config.is_default` rather than hardcoding `false`.
- All account rotation paths (UI `⇄` Switch button, Navbar Fast-Forward `⏩` button, Auto-Switcher background loop, Inbound Email `switch`/`rotate`, and Training REST API) must converge on the unified `switch_account_to_instance` + hot-switch credential injection pipeline.
