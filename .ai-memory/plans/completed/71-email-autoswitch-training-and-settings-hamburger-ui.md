# Completed Plan 71: Email Base64 HTML Cards, Smart Rotator IDE Switch, Training REST API & Settings Hamburger UI

## 1. Objective
Verify and complete:
1. **Rich HTML Email Cards & RFC 2045 Base64 MIME Encoding**: Ensure all outbound emails (ACK receipts, command executions, quota alerts, and self-tests) render as clean HTML UI cards without raw `<div style=...>` tags or collapsed plaintext `====` dividers, and include `[<VM_ALIAS> | <LOCAL_IP>]` in the Subject line.
2. **Flexible Pipe Whitespace Parsing**: Support zero, single, or multiple spaces around pipes (`VM3|1|help`, `VM3 | 1 | help`, `VM3   |   1   |   switch`).
3. **Smart Rotator & Highlighted `⇄` Switch Button (`AccountTable.tsx`) Unification**: Ensure `switch_account_to_instance`, `trigger_manual_rotation_for_instance`, and `InboundAction::AccountSwitch` use the Smart Rotator candidate scoring algorithm (`select_best_candidate_account`) and execute the exact `Kill-First -> Write-Second -> Start-With-Args-Third` credential & process restart pipeline for `default` (`Antigravity.exe`) as well as custom `--user-data-dir` instances.
4. **Remote Control & Training Telemetry REST API (`/v1/remote/control`, `/v1/training/telemetry`)**: Provide runtime toggles in `ProxyConfig` and UI cards in `Settings.tsx`.
5. **Settings Top Navigation & Hamburger Menu (`Settings.tsx`)**: Primary tabs (`General`, `Account`, `Proxy`, `Email-Alerts`, `Supabase`) + Hamburger Dropdown (`Advance`, `Debug`, `About`) and concise `Save` / `Backup` buttons.

## 2. Subtasks Completed
- **01 — Email E2E & Base64 HTML Card Rendering**: Completed in `src-tauri/src/modules/email_sender.rs` and `src-tauri/src/modules/email_inbound.rs`.
- **02 — Smart Rotator & Default Instance Switch Unification**: Completed in `src-tauri/src/modules/instance.rs` and `src-tauri/src/commands/mod.rs`.
- **03 — Training & Remote Control REST API**: Completed in `src-tauri/src/proxy/handlers/remote_training.rs` and `src-tauri/src/proxy/server.rs`.
- **04 — Settings Hamburger Menu (`Advance`, `Debug`, `About`)**: Completed in `src/pages/Settings.tsx`.
- **05 — Release & Host Daemon Update (`v4.71.2`)**: Completed with RCA (`02-spec/22-app-issues/05-email-html-rendering-and-smart-rotator-ide-switch-rca.md`), version bump (`v4.71.2`), and changelog updates.
