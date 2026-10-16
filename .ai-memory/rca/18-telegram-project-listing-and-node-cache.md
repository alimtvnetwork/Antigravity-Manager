# RCA 18: Telegram `/send` listed conversation ids, oldest first, as `Node-*`

Released in v4.211.0 (`92e8caf5`, tag `v4.211.0`). Read this before editing the Telegram send picker, machine labels, or `list_all_prompts`.

## Symptom

Step 3 of Telegram `/send` showed:

```text
Machine: Node-823632
Instance: default
1. project__default · conv-f6a
```

The user could not tell projects apart. The second token was the first eight characters of the conversation id. The list was the oldest eight rows. There was no next page. The numbers on the message did not select anything. The current computer was labeled with a generated node id.

## Root cause

One sentence: the picker printed `project_id` plus a slice of `session_id`, in the FIFO order of `list_all_prompts()`, and it used that same id as `callback_data`.

Facts in the code before the fix:

1. `conversation_entries` in `src-tauri/src/modules/telegram_inbound/send_flow.rs` built the label `{project_id} · {session_id first 8 chars}`. `ActivePrompt` has no title field. Titles live in Antigravity `conversation_summaries.db`, read by `lookup_conversation_title` in `src-tauri/src/modules/repo_db/text_utils.rs`.
2. `list_all_prompts()` in `src-tauri/src/modules/repo_db/project_queries.rs` is `ORDER BY created_at ASC, id ASC LIMIT 500`. That order is the dispatch FIFO invariant. The picker walked that vector and stopped at eight, so the user saw the oldest eight conversations.
3. Selection was an inline button `send:c:{machine}|{instance}|{value}`. Telegram rejects any `callback_data` over 64 bytes (`BUTTON_DATA_INVALID` drops the whole message). Long ids were skipped. The printed numbers had no handler. `consume_send_prompt` ignored plain text while the stage was `PickConversation`.
4. The machine line printed the raw alias. An unset alias is `Node-` plus digits (`local_machine_label` / `detect_machine_name`). Nothing asked GitMap for a proper name, and nothing cached the answer.

## Segment map (read these files, in this order)

| File | What it owns |
|---|---|
| `src-tauri/src/modules/telegram_inbound/send_listing.rs` | Page of eight projects. Newest `updated_at` first. Project name, then conversation title. Does not change SQL. |
| `src-tauri/src/modules/telegram_inbound/node_label.rs` | "This machine", GitMap lookup, SQLite cache. |
| `src-tauri/src/modules/telegram_inbound/send_flow.rs` | Wizard stages, callbacks `send:n:`, `send:next:`, `send:back:`, typed numbers. |
| `src-tauri/src/modules/telegram_inbound/types.rs` | `TelegramConfig.node_name_refresh_hours`, default 24. |
| `src-tauri/src/modules/telegram_inbound/palette.rs` | Emoji marks. Telegram HTML has no hex color. |
| `src-tauri/src/modules/repo_db/project_queries.rs` | `list_all_prompts`. FIFO. Do not reorder it for the picker. |
| `src-tauri/src/modules/repo_db/text_utils.rs` | `lookup_conversation_title`. |
| `02-spec/21-app/telegram-project-listing-and-ui-enhancement/` | Spec. |
| `.ai-memory/plans/telegram-project-listing-and-ui-enhancement.md` | Plan. |

Related commits that shipped in the same tag, and must not be reverted while "cleaning up" the picker:

| Commit | What it did |
|---|---|
| `94566699` | SSH Test Connection calls `test_ssh_node_connection`. It must not call `send_remote_ssh_prompt` (that command requires `instance_name` and raised E9001). |
| `077afdaf` | Updater console is its own window, 120 columns, paths on the next line. `install.ps1` strips ANSI with `[char]27`, not PowerShell 7 `` `e ``. |
| `bdf62306` | Instance table hover uses `group` on the `<tr>`. Prompt tree copies AI JSON / text / markdown and exports running or queued JSON. |
| `c7d1a4a2` | Audit panel shows the last two caller methods. |
| `92e8caf5` | This picker. |

v4.210.0 already shipped the Riseup card palettes (`card_palette`, `/theme`). Do not put `<blockquote>` back. Telegram paints blockquotes with the chat accent, which was the red and white cards.

## How the fix works

`load_project_page(instance_id, page)`:

1. Calls `list_all_prompts()`. On error it `record_ignored`s and returns an empty page. The manual-entry button stays.
2. Filters to that `instance_id`.
3. Sorts a local `Vec` by `updated_at` descending, then `id` descending. The SQL statement is untouched.
4. Keeps one row per `project_id`: the newest prompt.
5. Project name is the file name of `repo_path`. If that is empty, a trailing `__{instance_id}` is stripped from `project_id`.
6. Conversation title is `lookup_conversation_title(session_id)`. If that is empty, the first non-empty line of `prompt_content` (80 chars). If that is empty, `Untitled conversation`. The session id is never the title.
7. Page size is 8. `page` is 0-based. `has_next` and `has_back` come from `page_slice`.

`format_project_list` prints:

```text
1. {palette.mark} <b>project name</b>
   {palette.bullet} conversation title
```

Callbacks stay short so they cannot hit the 64-byte cap:

- `send:n:{page}|{index}` selects a row. The id is resolved again from `load_project_page` on the server.
- `send:next:{page}` and `send:back:{page}` turn the page.
- `send:cman` and `send:cancel` are unchanged.
- Typing `1` through `8`, or the word `next`, during `PickConversation` uses the same path. The `Mutex` in `consume_send_prompt` is dropped before any `.await` (see RCA 12).

Machine label:

- `is_current_machine` is true for `local`, `localhost`, `local_machine_label()`, or `detect_machine_name()`.
- The button text is `This machine` and the button JSON includes `"style": "success"` (Telegram green). The header is a green circle plus bold "This machine". Message text cannot be colored with hex.
- Any other alias that is not `Node-` plus digits is shown as itself.
- A `Node-<digits>` alias is read from `node_display_cache.db` in the data directory. The table is `node_display_cache(node_key, display_name, updated_at)`. If `updated_at` is inside `node_name_refresh_hours` (default 24), GitMap is not called. Otherwise the code runs `gitmap ssh nodes export-json` once, prefers a non-generated alias on the same IP, and upserts the row.
- Cache failures use `record_ignored` plus a `// Justification:` comment. The label still renders.

`PickerProject` derives `Clone`. `load_project_page` copies the page with `to_vec()`. Removing `Clone` does not compile (`error[E0277]`).

## Mistakes other AIs make on this segment

1. Reordering `list_all_prompts` to `updated_at DESC` or `created_at DESC`. That breaks FIFO restore and queue dispatch. Sort only inside `send_listing.rs`.
2. Putting the conversation id back in the label, or in `callback_data`, to "make selection reliable". Long ids make Telegram reject the whole keyboard. The number callback is the reliable path.
3. Showing every conversation as its own row and calling that eight projects. The page is eight projects. Each project shows its newest conversation title.
4. Printing `Node-823632` for the computer that is running the bot. If `is_current_machine` is true, the text is `This machine`.
5. Calling `gitmap ssh nodes export-json` on every message. That is what the SQLite cache and `node_name_refresh_hours` exist to stop.
6. Using `<blockquote>` or a hex color for "green". Blockquotes use the chat accent. Hex is not in Telegram HTML. Use the palette mark and `style: success|primary|danger` only.
7. Holding `SEND_FLOWS`'s `MutexGuard` across `.await`. That is `error[E0015]` / "future cannot be sent between threads" (RCA 11 and RCA 12).
8. Writing a test that expects the literal alias `W3` from `destination_locked_reply`. On a machine whose name is `W3`, the header becomes "This machine" and the assertion fails. Use an alias that is not this host, such as `remote-studio`.
9. Treating a passing unit test as a live Telegram proof. v4.211.0 was compiled (`cargo test --lib telegram_inbound::`, clippy, `npm run build`). Nobody tapped the bot in that release. Restart the installed manager or the old binary keeps the old picker.
10. Editing `send_flow.rs` to "simplify" and deleting `send:cman` legacy parsing. Old keyboards already in the chat still send `cman:machine|instance` (RCA 15).
11. Swallowing `Result` with `let _ =` or `.ok()`. Failures go through `record_ignored` with a justification comment, or they propagate.
12. Killing the `default` instance, or leaving the auto-switch threshold at 98, while testing this picker.

## Checklist (do this before you edit)

- [ ] Read this file and RCA 12, RCA 14, and RCA 15.
- [ ] State which file in the segment map you will change. One owner per file.
- [ ] Confirm `list_all_prompts` still says `ORDER BY created_at ASC, id ASC`.
- [ ] Confirm new picker callbacks are `send:n:`, `send:next:`, or `send:back:`, each under 60 bytes.
- [ ] Confirm the visible line is project name then conversation title, and the title function refuses to return the session id.
- [ ] Confirm the current machine button text is `This machine` and its style is `success`.
- [ ] Confirm a `Node-<digits>` name is served from `node_display_cache` until `node_name_refresh_hours` elapses.
- [ ] Confirm `PickerProject` still derives `Clone`.
- [ ] Add or adjust a unit test that does not need the real data directory, except a cache test that opens `node_display_cache.db` on purpose.
- [ ] Run `cargo fmt` in `src-tauri`, then `cargo test -j 1 --offline --lib telegram_inbound:: -- --test-threads=1`.
- [ ] Do not claim the bot is fixed until the process running Telegram has been restarted on this version.

## Must-follow, so the app does not break

Do these. Skipping one of them is how this screen breaks again.

1. Keep FIFO in SQL. Newest-first belongs only in `projects_for_instance`.
2. Keep selection by page and index. Resolve `value` (`session_id`, or `project_id` when the session id contains `|` or `:`) at click time.
3. Keep `send:cman` with no parameters, and keep accepting the old `cman:` form.
4. Keep `machine_entry_usable` (`picker_safe` and `callback_fits`) on every machine alias that is placed in `callback_data`. Display text may be "This machine". The callback still carries the real alias, and that alias must stay short and free of `|` and `:`.
5. Drop the flow mutex before `.await`.
6. On a prompt-database error, return an empty page and leave "Type it manually" on the keyboard.
7. On a GitMap or cache error, show the previous name or the original alias. Do not fail the wizard.
8. Escape `&`, `<`, and `>` in names before they enter HTML. Use `clean_for_telegram_html` or `escape_html` in `node_label.rs`.
9. Add `node_name_refresh_hours` only with `#[serde(default = "default_node_name_refresh_hours")]` so old `telegram_config.json` files still load.
10. After a behavior change, bump with `npm run bump`, fill `CHANGELOG.md` and `CHANGELOG_EN.md`, and the latest-version line in `README.md` and `README_EN.md`. Attribution is only `(Thanks to @aukgit)`. Tag `vX.Y.Z` must match the `## [vX.Y.Z]` heading. Paths in notes stay relative.

## What was not verified

Live Telegram on a phone was not exercised for v4.211.0. Title quality depends on `conversation_summaries.db`. The GitMap export parser was matched to `SshNodeExportItem` and the `mainMachine` / `nodes` envelope, and it was not run against a live `gitmap ssh nodes export-json` during the release. Remote machines still list conversations from the local prompt database filtered by `instance_id`. A remote node with no local rows shows the empty state. Conversation rename inside the IDE, and refreshing cloned exes when the IDE binary changes, are not part of this fix.
