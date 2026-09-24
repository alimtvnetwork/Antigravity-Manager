# Completed Plan 72: Multi-Agent Verification & Polish of Email E2E, Smart Rotator, Training API & Settings Crossed-Debug Hamburger Menu

Spec Reference: [02-spec/21-app/44-settings-hamburger-ui-and-system-e2e-verification.md](../../../02-spec/21-app/44-settings-hamburger-ui-and-system-e2e-verification.md)
Completed in: 6 pipeline steps across 2 parallel verification subagents.

## Completed Subtasks

### 01 — Settings Hamburger Dropdown with Crossed-Out Debug Option (`Task-04`)
- Verified primary tabs (`General`, `Account`, `Proxy`, `Email-Alerts`, `Supabase`) and action buttons (`Save`, `Backup`).
- Styled `Debug` inside the Hamburger Dropdown (`isMoreDropdownOpen`) in `src/pages/Settings.tsx` with `line-through decoration-2 decoration-rose-500` and a `Top Bar ↗` badge indicating Debug is moved to the top-level titlebar Bug icon, while clicking it invokes `openDebugModal()` from `useDebugConsole()`.
- Added the `Remote Control REST API (/api/v1/remote/control)` toggle switch alongside `Machine Training REST API (/api/v1/training)` in `Settings.tsx` (`General` tab).

### 02 — Backend Email Base64 HTML, Smart Rotator IDE Switch & Training REST API (`Task-01`, `Task-02`, `Task-03`)
- Verified `email_sender.rs` (`encode_mime_base64_body`, `wrap_html_email_card`, `build_mime_message` with `[<VM_ALIAS> | <LOCAL_IP>]` subject prefix) and `email_inbound.rs` (`parse_single_command_string` flexible pipe spacing).
- Verified `instance.rs` (`switch_account_to_instance` 3-phase `Kill-First -> Write-Second -> Start-With-Args-Third` sequence) and `auto_switcher.rs` (`score_candidate_account`, `select_next_best_profile`, `trigger_manual_rotation_for_instance`).
- Registered `/v1/remote/control`, `/api/v1/remote/control`, and `/v1/training/telemetry` route aliases in `src-tauri/src/proxy/server.rs` and `remote_control_api_enabled` in `src-tauri/src/models/config.rs`.
