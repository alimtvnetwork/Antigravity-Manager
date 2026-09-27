# Subtask 03: Telegram /ff Backup+Reinjection, /prompt & /help Command Catalog

## Description
In `telegram_inbound.rs`:
- Update `/ff` handler to call `repo_db::backup_running_prompts("default")`, perform the switch, and call `repo_db::resend_all_running_commands(20)`.
- Verify `/prompt <project> <text>` and `/prompt <node> <proj> <text>`.
- Expand `/help` command catalog to document all commands, projects, nodes, and backup/restore controls.
- Ensure all responses use `send_telegram_message_chunked` for safe length limits.

## Actions
- Edit `src-tauri/src/modules/telegram_inbound.rs`.
