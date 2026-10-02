# Task history split database

Account adds and account switches write an audit row. The UI page is Audit, beside Settings. The CLI is `agm history` (`agm audit`).

## Layout

Same rule as `backup_prompts_db`: if `data/task-history` exists next to the process working directory, that folder is the store. Otherwise the store is `{data_dir}/task-history/`.

- `task_index.db` is the root catalog. Table `split_catalog` has `id`, `kind`, `file_path`, `created_at`, `closed_at`, `row_count`, `is_current`.
- Each batch is `history-<8 chars>.db` with table `tasks`: `id`, `action`, `status`, `subject`, `detail`, `instance_id`, `created_at`, `finished_at`.
- A split closes at 500 rows. The next event opens a new file and a new catalog row. The catalog is the only place that knows the path.

## Enqueue

`AuditTask::start` inserts `status = queued`. `succeed` or `fail` sets the final status. If the task is dropped still open, the row becomes `fail` with `stopped before the task finished`.

`record` is the one-shot used when add-account has already succeeded.

Hooks:

- `account::add_account` and the update branch of `account::upsert_account` — action `add_account`
- `account::switch_account` — default-instance switch
- `instance::switch_account_to_instance` — instance switch, including auto-switch of a non-default instance

A history write must not fail the account operation. Errors are logged as `[History]`.

## Read

- CLI: `agm history [--page n] [--json]`. Page size is 100. The command prints each split path.
- UI: `list_task_history` in `src/pages/Audit.tsx`. Page size is 100. Previous and Next walk the same order: newest split first, then older splits.
- Module: `src-tauri/src/modules/task_history_db.rs`
