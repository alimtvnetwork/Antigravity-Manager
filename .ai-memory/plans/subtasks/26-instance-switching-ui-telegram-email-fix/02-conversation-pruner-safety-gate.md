# Subtask 02: Conversation Pruner Safety Gate & CLI/Telegram Parity

Traceability ID: Task-04, Task-05
Spec Reference: [02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md](../../../02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md)
Target Files: src-tauri/src/modules/agy_cleaner.rs, src-tauri/src/bin/agm.rs, src-tauri/src/modules/telegram_bot.rs
Action:
1. In `agy_cleaner.rs`, implement a strict safety check before deleting any conversation history:
   - Query running and queued prompts from `repo_db` / active prompts store. NEVER purge prompts that are currently running or queued.
   - For all active/running projects, protect and retain at least the latest 5 conversation session files (`.vscdb` / `state.vscdb` / `*.json`).
2. Add full CLI prune support in `agm.rs` (`agm prune --keep 5 --dry-run` or `agm clear-cache --prune-keep 5`), with clear terminal help documentation.
3. Add Telegram `/prune` command in `telegram_bot.rs` allowing operators to trigger a safe conversation prune remotely, reporting pruned count and preserved count.

Acceptance Criteria:
- Running and queued prompts are never deleted during conversation pruning.
- Active projects retain at least their latest 5 conversation sessions.
- `agm prune` CLI command and Telegram `/prune` command execute cleanly and report results.
Targeted Verification: git diff --stat src-tauri/src/modules/agy_cleaner.rs src-tauri/src/bin/agm.rs
