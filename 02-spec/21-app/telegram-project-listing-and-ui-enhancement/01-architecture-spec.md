# Architecture Specification: Telegram Project Listing

> **Target:** `02-spec/21-app/telegram-project-listing-and-ui-enhancement/01-architecture-spec.md`
> **Status:** DRAFT
> **Slug:** `telegram-project-listing-and-ui-enhancement`
> **Scope:** Telegram inbound project picker page (`src-tauri/src/modules/telegram_inbound/send_listing.rs`)

---

## 1. Picker behavior

The `/send` project picker lists live tracked prompts for one instance. The operator sees eight projects per page. The newest conversation is first. Each row shows the project name, then the conversation title. The operator selects a row by its number on the current page and can move to the next page when more projects remain.

Ordering for display is applied in memory after the prompt rows are loaded. `list_all_prompts` in `src-tauri/src/modules/repo_db/project_queries.rs` keeps its SQL unchanged:

```sql
ORDER BY created_at ASC, id ASC
```

That FIFO query is the dispatch invariant. The picker must not rewrite it to `updated_at DESC` or any other order. Newest-first display is a local `Vec` sort only.

## 2. Public API

`send_listing.rs` exposes the page the send flow renders. `send_flow.rs` calls these items; this file does not own callback routing.

```rust
pub struct PickerProject {
    pub value: String,
    pub project_name: String,
    pub conversation_title: String,
    pub updated_at: i64,
}

pub struct PickerPage {
    pub rows: Vec<PickerProject>,
    pub page: usize,
    pub has_next: bool,
    pub has_back: bool,
}

pub fn load_project_page(instance_id: &str, page: usize) -> PickerPage

pub fn format_project_list(rows: &[PickerProject], page: usize) -> String
```

### 2.1 Load

1. Call `crate::modules::repo_db::list_all_prompts()`. On error, `crate::error::record_ignored` with a Justification comment, and return an empty page (`page` 0, no next, no back).
2. Keep rows whose `instance_id` equals the requested instance.
3. Sort that local `Vec` by `updated_at` descending, then `id` descending.
4. Keep one row per `project_id`: the newest prompt.
5. Sort those projects by that prompt's `updated_at` descending.
6. Page size is 8. `page` is 0-based. An out-of-range page clamps to the last page when the list is non-empty, otherwise 0.
7. `value` is `session_id` when it is non-empty and picker-safe (no `|` and no `:`). Otherwise `value` is `project_id` when that is picker-safe. Projects whose value is unsafe are skipped.
8. `project_name` is the file name of `repo_path` when `repo_path` is non-empty. Otherwise it is `project_id` with a trailing `__` plus the instance id removed when that suffix is present.
9. `conversation_title` comes from `crate::modules::repo_db::lookup_conversation_title` on the session id. If that returns nothing, use the first non-empty line of `prompt_content`, trimmed to 80 characters. If that is still empty, use `Untitled conversation`. The session id is never the title.

### 2.2 Format

`format_project_list` numbers each row from 1 inside the page:

```text
{n}. {palette.mark} <b>{project}</b>
   {palette.bullet} {title}
```

`palette` is `crate::modules::telegram_inbound::palette::active_card_palette()`. Project name and conversation title are passed through `clean_for_telegram_html` in `src-tauri/src/modules/telegram_inbound/formatting.rs` (it is `pub`).

`has_next` is true when a later page exists. `has_back` is true when `page` is greater than 0. Number selection and the next-page control are rendered by the send flow from `PickerPage`; this module supplies the rows and the HTML list.

## 3. Tests

`#[cfg(test)]` tests stay in memory. They call the `pub(crate)` helpers `page_slice` and `project_label`. They do not open the prompt database.

- `project_label` returns the repo path file name, and strips a trailing `__` plus instance id when the path is empty.
- A page slice of 9 items on page 0 yields 8 rows and `has_next`.
- The second page sets `has_back`.
