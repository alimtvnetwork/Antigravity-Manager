# Issue 05: Email HTML MIME Rendering, Node Identity Prefixing & Smart Rotator IDE Switch Delegation

- **Canonical Tier 22 Spec:** `[02-spec/22-app-issues/05-email-html-rendering-and-smart-rotator-ide-switch-rca.md](../../../02-spec/22-app-issues/05-email-html-rendering-and-smart-rotator-ide-switch-rca.md)`
- **Status:** Fixed
- **Summary:**
  1. Upgraded `email_sender::build_mime_message` to use RFC 2045 `Content-Transfer-Encoding: base64` (76-char CRLF lines) for `text/plain` and `text/html` MIME parts so Gmail and strict SMTP servers never flatten HTML cards or leak raw `<div style=...>` tags.
  2. Standardized `format_reply_subject` and `render_html_receipt` in `email_inbound.rs` so every ACK and Result receipt carries `[<VM_ALIAS> | <LOCAL_IP>]` in the Subject line and renders structured HTML cards/tables.
  3. Fixed `is_instance_running` and `close_instance` in `instance.rs` to pass `is_default = instance_id == "default" || config.is_default` into `find_pids_for_data_dir`, and unified `switch_account_to_instance` with `DesktopIntegration` hot-switch (`kill_language_server_subprocesses`, `storage.json` profile write, `state.vscdb` injection, and keyring update) and Smart Rotator candidate selection.
