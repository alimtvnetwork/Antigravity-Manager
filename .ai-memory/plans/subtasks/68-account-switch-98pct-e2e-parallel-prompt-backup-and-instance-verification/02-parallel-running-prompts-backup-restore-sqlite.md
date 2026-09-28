# Subtask 02: Parallel Running Prompts Backup & Restore to SQLite Engine

Traceability ID: Task-03
Spec Reference: [02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md](../../../02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md)
Target Files: src-tauri/src/commands/agm_cli.rs, src-tauri/src/services/prompt_runner.rs, src-tauri/src/database/prompts_split_db.rs
Action:
- Implement parallel prompt backup command in AGM CLI: `agm prompts backup` (and `export`).
- Discover all running AGY project workspaces concurrently using Tokio tasks.
- For each active workspace, serialize the current single running prompt including text prompt, image payload base64 / path, model name, and project directory.
- Persist backup records into SQLite database (`running_prompts` table in `prompts.db`), ensuring fast ACID batch inserts.
- Implement corresponding restore command: `agm prompts restore` to reload serialized state.

Acceptance Criteria:
- `agm prompts backup` executes parallel collection across all active workspaces.
- Attached images and metadata are preserved without data corruption.
- Database entries are indexed by project name (no cryptic internal IDs).
- `agm prompts restore` successfully reads and unmarshals saved prompts.

Targeted Verification:
- Unit test `modules::backup_prompts_db::tests::test_backup_db_lifecycle`: PASSED (1 passed).
- CLI verification with `agm backup`: PASSED (5 active prompts snapshotted to split SQLite database).
- CLI verification with `agm backup ls`: PASSED (batches and records indexed by friendly project name).
- CLI verification with `agm restore`: PASSED (5 prompts restored and re-enqueued into workspace execution queue).

Status: Completed
Verification Date: 2026-09-28
