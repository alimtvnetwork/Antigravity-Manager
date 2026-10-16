# Plan: telegram-project-listing-and-ui-enhancement

Status: ACTIVE
Branch: main

## User Request (Verbatim)

Telegram Project Listing and UI Enhancement: high priority instruction, non-negotiable task.

In the Telegram, when we see the project names, the first thing we should have the project name, and then inside, we should have a conversation title, not the conversation Id, because we are not using the Id. Based on that, the user can select based on the number. All these numbers should appear, and the user can do a next. That would give the next eight projects on the list. The projects would be listed based on the recent activities, meaning the recent conversation time. Whoever has the recent conversation would be on the top of that list. We should be able to select the project based on the number, so there should be a number list. In some cases, the buttons need to be colored. When you say this machine, rather than saying a node number, try to mention it as this machine, with a green color if you're dealing with the current machine. If it has a different node number, and if the machine has gitmap, it would use the gitmap to get its node name, proper node name. In case the user didn't put the node aliasing, the system would try to get that, and it would also update to the SQLite database so that it does not have to do it all the time. It would only do it once because it's a cache behavior, and probably every day it would do once. That could be from a settings, like every 24 hours, it would do at least once. Improve the UI, and the project listing color and numbers. This can be a white color. Try to make it a little bit colorful in the conversation names, project names. It can have a project and sub-project for eight projects. This way it would look professional. At the end, try to do the minor bump and release.

## What is wrong today

`/send` step 3 is built in `src-tauri/src/modules/telegram_inbound/send_flow.rs` (`on_instance_picked`, `conversation_entries`).

- Labels are `{project_id} · {session_id first 8 chars}`. `ActivePrompt` has no title. Titles live in Antigravity `conversation_summaries` via `lookup_conversation_title` in `src-tauri/src/modules/repo_db/text_utils.rs`.
- Order is `list_all_prompts()` which is `ORDER BY created_at ASC, id ASC`. That FIFO query must stay. The picker then keeps the oldest eight.
- There is no next page. `MAX_PICK_BUTTONS` is 8 and the rest are dropped.
- The numbers in the message are not selectable. Selection is an inline button whose `callback_data` embeds the conversation id and is dropped when it exceeds 60 bytes.
- The machine line prints the raw alias. An unset alias looks like `Node-823632`.

## Behavior

1. One numbered row per project, eight projects per page, most recently updated conversation first (`updated_at` descending). The FIFO SQL query is not changed. Sorting happens in memory for this picker only.
2. Each row is the project name, then the conversation title underneath. The conversation id is not the label. Title comes from `lookup_conversation_title`. If that is empty, use the first line of `prompt_content`. If that is empty, use `Untitled conversation`.
3. Project name is the folder name of `repo_path`. If that is empty, strip a trailing `__{instance}` from `project_id`.
4. The user selects by number. The message shows `1.` through `8.`. The keyboard has those same numbers. Typing `1` through `8` during the pick also selects that row. Callback data is `send:n:{page}|{index}` so the id stays on the server.
5. `Next` loads the next eight projects (`send:next:{page}`). `Back` loads the previous page when the page is not the first.
6. The current machine is labeled `This machine` and its button uses Telegram `style: success` (green). The header uses the green marker plus bold text. Telegram message HTML has no hex color.
7. Any other machine uses a proper name. If the alias matches `Node-` plus digits, resolve a name once through `gitmap ssh nodes export-json`, store it in SQLite, and reuse it until `node_name_refresh_hours` elapses. Default is 24, stored on `TelegramConfig`.
8. Project lines use the active card palette mark. Conversation lines use the palette bullet. Numbers stay in bold.

## File ownership

- Worker 01: `src-tauri/src/modules/telegram_inbound/send_listing.rs`, `02-spec/21-app/telegram-project-listing-and-ui-enhancement/01-architecture-spec.md`, `.ai-memory/plans/subtasks/telegram-project-listing-and-ui-enhancement/01-listing.md`
- Worker 02: `src-tauri/src/modules/telegram_inbound/node_label.rs`, `src-tauri/src/modules/telegram_inbound/types.rs`, `02-spec/21-app/telegram-project-listing-and-ui-enhancement/02-component-spec.md`, `.ai-memory/plans/subtasks/telegram-project-listing-and-ui-enhancement/02-node-cache.md`
- Lead only: `send_flow.rs`, `mod.rs`, `02-spec/21-app/readme.md`, `.ai-memory/plans/readme.md`, release notes

## Out of scope

- Changing `list_all_prompts` ordering.
- Renaming conversations inside the IDE.
- Killing running instances.

## Release

After the picker is wired, `npm run bump minor` on main, changelog attribution only `(Thanks to @aukgit)`, README changelog sync, tag, push. Relative paths only.
