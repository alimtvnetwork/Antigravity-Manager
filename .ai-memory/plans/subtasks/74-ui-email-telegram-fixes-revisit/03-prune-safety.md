# Subtask 03: Prune Safety for Active Prompts
Traceability ID: Task-04
Spec Reference: [02-spec/21-app/74-ui-email-telegram-fixes-revisit.md](../../../02-spec/21-app/74-ui-email-telegram-fixes-revisit.md)
Target Files: src-tauri/src/modules/cleaner.rs, src-tauri/src/modules/agy_cleaner.rs, src-tauri/src/bin/agm.rs
Action: Ensure clear/purge routines skip running or queued prompts.
Acceptance Criteria: Active prompts are never deleted by the conversation pruner. Guarantees retention of top 5 sessions per active workspace.
Targeted Verification: Verified prompt safety invariants in `prune_conversations_only` and `cmd_clear_cache`.

Status: COMPLETED
Outcome: `agy_cleaner.rs` guards active, queued, and executing prompts in `repo_db` and SQLite snapshots. Guarantees retention of the top 5 sessions per active workspace.
