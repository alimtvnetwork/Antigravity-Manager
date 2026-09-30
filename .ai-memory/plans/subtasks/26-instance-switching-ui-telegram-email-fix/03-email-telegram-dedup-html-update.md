# Subtask 03: Email & Telegram Deduplication, Telegram HTML Repair & Update Command

Traceability ID: Task-06, Task-07, Task-08
Spec Reference: [02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md](../../../02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md)
Target Files: src-tauri/src/modules/notification_hub.rs, src-tauri/src/modules/telegram_bot.rs, src-tauri/src/modules/email_sender.rs
Action:
1. In `notification_hub.rs` and `telegram_bot.rs`, deduplicate project names in email and Telegram cards using an `IndexSet` / `HashSet` to guarantee each project name appears exactly once.
2. Fix broken raw HTML tags in Telegram bot messages by strictly escaping dynamic data (`<`, `>`, `&`) so Telegram's HTML parser never errors out or falls back to displaying raw unrendered markup.
3. Add `/update` command in `telegram_bot.rs` allowing remote execution of AGM self-updater, and expose pending update notifications in `/status`.
4. Ensure text contrast in email and Telegram templates uses high-contrast colors (e.g. bright text on dark cards, no blue-on-blue links).

Acceptance Criteria:
- No duplicate project names in emails or Telegram messages.
- Telegram HTML rendering is clean and never shows raw unrendered `<b>` or `<code>` tags.
- Operators can check and trigger updates via Telegram `/update` and view update status in `/status`.
Targeted Verification: git diff --stat src-tauri/src/modules/notification_hub.rs src-tauri/src/modules/telegram_bot.rs
