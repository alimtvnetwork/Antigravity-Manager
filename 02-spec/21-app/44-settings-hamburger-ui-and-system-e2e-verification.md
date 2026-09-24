# AC-APP-044: Settings Hamburger UI, Machine Training REST API, Rich HTML Email & Smart Rotator IDE Switch

## User Request (Verbatim)
```text
These below email section and screenshots are not fixed yet please fix it

https://prnt.sc/P4MLFuhMFm_5
https://prnt.sc/YYtqWnt8jPLb
https://prnt.sc/JvppT3cA_48d

You can improve these replies. So when you reply back with information which user does not have to respond back, this can actually come as a nice looking UI or HTML, and you can work on it, make it better. And also in the title, you need to mention where it is coming from, which VM, which IP, which node. Okay, so these are the information that is important. And user can give space in between the pipe or no space, it should still work. And user can give multiple space and it would still work. Do you understand? Okay, write the root cause analysis, update the code, update everything, and make a bump in the version and release. Is it clear?

Needs to have VM alias and ip to know and provide properly html body display please, clear????

The email I have received, but the account switch was not effective on the ID. Why? Because you didn't use the fast forward or smart rotator. So your job is to use the smart rotator when you switch. Okay, you find the workspace, you probably ask the smart rotator, "Is this the right one?" If smart rotator says yes, then you just switch, or else you find the next one. Okay? Or smart rotator says, "Which one is the best one?" You rotate using its rotate button or switch button. Okay? So you follow through that button. You are not using, or you are not delegating it properly. Why? I have highlighted the button which you should click on. Is it clear? So make sure that you fix it immediately
```

## Visual Context References
- `![Settings Hamburger UI](../../assets/screenshots/training-api-settings-ui-01.png)`
- `![ACK Email Plaintext Collapse](../../assets/screenshots/email-smart-rotator-fix-01.png)`
- `![Account Switched Raw HTML Leak](../../assets/screenshots/email-smart-rotator-fix-02.png)`
- `![Highlighted Account Switch & Smart Rotator Button](../../assets/screenshots/email-smart-rotator-fix-03.png)`
- Related Root Cause Analysis: `[02-spec/22-app-issues/05-email-html-rendering-and-smart-rotator-ide-switch-rca.md](../22-app-issues/05-email-html-rendering-and-smart-rotator-ide-switch-rca.md)`

## Core System Architecture & Invariants

### 1. Settings Menu UI/UX Specification
- **Visible Tabs**: `General`, `Account`, `Proxy`, `Email-Alerts`, `Supabase`, plus **Hamburger Dropdown** (`Menu` + `ChevronDown`) containing `Advance`, `Debug` (directing to top-level titlebar Bug icon), and `About`.
- **Top Actions**: `Backup` and `Save`.

### 2. Rich HTML Email & Node Identity Prefix Specification
- **Subject Title Invariant**: Every outbound email (ACK receipt, completion receipt, account switch alert, quota alert, test ping) MUST include `[<VM_ALIAS> | <LOCAL_IP>]` in the Subject title (e.g. `Re: [VM3 | 192.168.1.12] [AGM ACK] Running: help`, `Re: [VM3 | 192.168.1.12] [AGM Result] SUCCESS: help`).
- **Base64 MIME Transfer Encoding**: `email_sender::build_mime_message` MUST encode both `text/plain` and `text/html` parts using `Content-Transfer-Encoding: base64` wrapped at 76 characters with CRLF (`\r\n`) so Gmail and SMTP servers render HTML cards without leaking raw `<div style=...>` tags or collapsing lines.
- **Flexible Pipe Whitespace Parsing**: `parse_inbound_email` MUST accept zero, single, or multiple spaces around `|` delimiters (`VM3|1|help`, `VM3 | 1 | help`, `VM3   |   1   |   switch`) in both Subject and Body.

### 3. Smart Rotator & IDE Account Switch Delegation Specification
- **Default Instance Process Discovery**: `is_instance_running` and `close_instance` in `instance.rs` MUST pass `is_default = instance_id == "default" || config.is_default` into `find_pids_for_data_dir` so default Antigravity IDE processes (launched without `--user-data-dir`) are properly matched.
- **Unified Switch Execution (`switch_account_to_instance` + `DesktopIntegration`)**:
  - When switching an account on `default` or any active instance (via Inbound Email, Auto-Switcher, Smart Rotator, or UI `⇄` button):
    1. Consult Smart Rotator (`select_best_candidate_account`) to validate/select the highest-scoring candidate account.
    2. Refresh OAuth token (`oauth::ensure_fresh_token`) and ensure device profile isolation (`device::write_profile` to `storage.json`).
    3. Write credentials to OS keyring (`write_to_system_keyring`) and `state.vscdb` (`db::inject_token`) across both `instance.data_dir` and `db::get_db_path`.
    4. Execute **Hot Switch** (`process::kill_language_server_subprocesses`) followed by post-kill credential re-assertion (or full process restart via `process::close_antigravity` + `launch_instance`), matching the exact behavior of the highlighted `⇄` Switch button in `Accounts.tsx`.

### 4. Machine Training REST API Specification
- `GET /api/v1/training`, `GET /api/v1/status`, `GET /api/v1/training/telemetry`, `POST /api/v1/training/learn`, `POST /api/v1/training/machines`, `POST /api/v1/training/modify`.
- Controlled via `training_api_enabled` in `AppConfig`.
