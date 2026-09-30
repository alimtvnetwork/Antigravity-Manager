# Subtask 04: 200-Word Running Prompts Caching, Email Preview, CLI & SSH Parity

Traceability ID: Task-09, Task-10
Spec Reference: [02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md](../../../02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md)
Target Files: src-tauri/src/modules/backup_prompts_db.rs, src-tauri/src/modules/repo_db.rs, src-tauri/src/modules/notification_hub.rs, src-tauri/src/bin/agm.rs, src-tauri/src/modules/telegram_bot.rs
Action:
1. In `backup_prompts_db.rs` / `repo_db.rs`, store running prompt content and make it queryable. Provide a helper to retrieve extended prompt text (up to 200 words).
2. In `notification_hub.rs`, include a 200-word running prompts preview section in the post-switch email and Telegram alert cards.
3. In `agm.rs`, expose `agm running-prompts --words 200` and clear terminal help examples showing how to query prompts from the SQLite database.
4. Add GitMap SSH parity examples in `agm --help` and Telegram `/help` demonstrating how to select machines and execute commands via SSH.

Acceptance Criteria:
- Switch emails and Telegram cards include up to 200 words of running prompt preview text.
- Running prompts are cached in SQLite for immediate querying without slow file walks.
- CLI provides clear help examples for prompt queries and GitMap SSH machine dispatch.
Targeted Verification: git diff --stat src-tauri/src/modules/backup_prompts_db.rs src-tauri/src/bin/agm.rs
