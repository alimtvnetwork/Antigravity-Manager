# Subtask 04: Notification Deduplication & Formatting
Traceability ID: Task-05, Task-06, Task-10
Spec Reference: [02-spec/21-app/74-ui-email-telegram-fixes-revisit.md](../../../02-spec/21-app/74-ui-email-telegram-fixes-revisit.md)
Target Files: src-tauri/src/modules/email_sender.rs, src-tauri/src/modules/telegram_inbound.rs
Action: Deduplicate project names using a Set/dictionary. Fix HTML tags in Telegram. Expand prompt text preview to >= 200 words in email/Telegram.
Acceptance Criteria: No duplicate names in alerts. Proper HTML formatting without Telegram API rejection.
Targeted Verification: Verified HashSet project deduplication, word-based prompt preview expansion, and HTML tag balancing across chunk boundaries.

Status: COMPLETED
Outcome:
1. `src-tauri/src/modules/email_sender.rs`:
   - Deduplicated workspace/project names using `HashSet<String>` in `render_idle_projects_email`.
   - Replaced 80-char truncation with `extract_words_preview` providing >= 200-word previews with word count badges.
2. `src-tauri/src/modules/telegram_inbound.rs`:
   - Resolved HTML formatting breakdown by implementing `get_unclosed_html_tags` and auto-balancing open/closing tags across split chunk boundaries without cutting inside `<...>`.
   - Expanded prompt preview cards to >= 200 words.
