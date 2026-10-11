# Subtask 03: Telegram Rich Formatting, Copy Button & Fleet Identity

## Context
Improve Telegram chat messages with colorful, structured HTML, prompt code blocks, native copy button, free image reuse, and fleet identification without Supabase.

## Files to Modify
- `src-tauri/src/modules/notification_hub/telegram_alerts.rs`
- `src-tauri/src/modules/telegram_inbound/processing.rs`
- `src-tauri/src/modules/telegram_inbound/media.rs`

## Acceptance Criteria
1. Telegram account switch notifications render as rich HTML cards.
2. Prompt copy button attached via `InlineKeyboardMarkup` `copy_text`.
3. Standalone fleet identity card (Hostname, LAN IP, Instance ID, Account Email) embedded in notifications.
4. Telegram commands `/fleet`, `/instances`, `/accounts`, `/whoami` provide full fleet visibility without Supabase.
