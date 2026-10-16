# Subtask 01: Project picker listing

Owner: Worker 01. Files: `src-tauri/src/modules/telegram_inbound/send_listing.rs`, `02-spec/21-app/telegram-project-listing-and-ui-enhancement/01-architecture-spec.md`.

1. Describe the picker in the architecture spec: eight projects per page, newest conversation first, project name then conversation title, number selection, next page.
2. State that `list_all_prompts` SQL stays `ORDER BY created_at ASC, id ASC`.
3. Add `PickerProject`, `PickerPage`, `load_project_page`, and `format_project_list` for the send flow.
4. Filter by instance, sort a local `Vec` by `updated_at` then `id` descending, keep the newest prompt per `project_id`, and page by 8.
5. Build a picker-safe `value`, a project label, and a conversation title that never uses the session id.
6. Cover `project_label` and `page_slice` with in-memory tests.
