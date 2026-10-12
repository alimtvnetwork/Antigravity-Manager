# Telegram "System Updated!" Notification UI (follow-on)

**Version:** 1.0.0
**Updated:** 2026-10-13
**AI Confidence:** High
**Ambiguity:** None
**Supersedes context:** The 2026-10-13 `supabase-connection-and-ui-ux-enhancement` task arrived as a repeat of an already-completed plan (see `.ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md`, status COMPLETED, shipped in v4.195.0). This spec covers only the genuinely new follow-on: the Telegram notification UI.

---

## Keywords

`telegram-notify-ui` · `link-preview` · `notification_hub`

---

## Purpose

Specifies the rework of the "🚀 Antigravity Manager: System Updated!" Telegram message so it is scannable on narrow screens: no mid-word wrapping, no truncated link-preview card.

---

## Problem

Production message (v4.193.0, machine W3) renders `Machine: W3 (192.168.1.3) | Instance: default-copy-5597` on one line, wrapping mid-word (`default-` / `copy-5597`), and the GitHub releases `<a href>` emits a truncated web-preview card. Composer: `src-tauri/src/modules/notification_hub/system.rs`; sender: `src-tauri/src/modules/telegram_inbound/formatting.rs`.

## Changes

1. `formatting.rs`: `send_telegram_message_with_markup` gains `disable_preview: bool`; sets `disable_web_page_preview` on both the HTML and plain-text-retry payloads. New `send_telegram_message_no_preview` wrapper. Existing callers (`telegram_alerts.rs`, `telemetry.rs`, `daemon.rs`) pass `false` — behavior unchanged.
2. `system.rs`: telemetry header split across two lines (Machine / Instance); update alert sent via `send_telegram_message_no_preview`.

## Acceptance

- [ ] `cargo fmt -- --check` and `cargo clippy --all-targets --all-features` clean.
- [ ] Next update notification: Machine and Instance on separate lines; tappable releases link with no preview card.
- [ ] Targeted commit + push to `origin/main`. No release cut.
