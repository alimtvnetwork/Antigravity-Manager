# Subtask 05: CLI & Telegram Commands Expansion
Traceability ID: Task-07, Task-08, Task-09
Spec Reference: [02-spec/21-app/74-ui-email-telegram-fixes-revisit.md](../../../02-spec/21-app/74-ui-email-telegram-fixes-revisit.md)
Target Files: src-tauri/src/modules/telegram_inbound.rs, src-tauri/src/bin/agm.rs
Action: Implement `/update` and `/prune` commands in Telegram bot. Update CLI help with GitMap SSH examples and top-level SQLite query routing.
Acceptance Criteria: `/update` and `/prune` handled by bot. CLI help displays SSH examples. Top-level `agm query` routes to prompt search.
Targeted Verification: Verified Telegram bot menu registration and command routing, and `agm.rs` router/help expansion.

Status: COMPLETED
Outcome:
1. `src-tauri/src/modules/telegram_inbound.rs`:
   - Registered `/prune` and `/query` in `setMyCommands` payload alongside `/update`.
   - Handled `/update`, `/prune`, `/query`, `/observe` commands.
2. `src-tauri/src/bin/agm.rs`:
   - Mapped top-level `"query" | "search" | "find"` in command router to `cmd_prompts_query`.
   - Updated `print_help()` and `print_help_json()` with GitMap SSH commands (`gitmap ssh <node> "<cmd>"`, `agm ssh <node> <cmd>`, key export/deploy) and SQLite prompt query commands.
