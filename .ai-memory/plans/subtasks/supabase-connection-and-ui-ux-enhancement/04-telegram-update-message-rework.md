# Subtask 04 — Telegram "System Updated!" message rework

## Goal

The Telegram update notification becomes scannable and usable on narrow screens.

## Steps

1. Edit the composer in `src-tauri/src/modules/notification_hub/system.rs` (the `🚀 Antigravity Manager: System Updated!` template).
2. Put Machine and Instance on separate lines (kills the mid-word wrap seen in production).
3. Suppress the web-page preview for the GitHub releases link (Telegram `disable_web_page_preview`) so no truncated preview card renders; keep a clean inline link.
4. Keep order: header → telemetry → release details → link. Verify with `cargo fmt -- --check` and `cargo clippy --all-targets --all-features`.

## Done when

Message has no mid-word wrapping and no link-preview card; template compiles.
