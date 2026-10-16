# Subtask 02: Node Display Cache

> **Task:** Task-04
> **Agent:** Worker 02
> **Spec:** `02-spec/21-app/telegram-project-listing-and-ui-enhancement/02-component-spec.md`

## Owned files

- `src-tauri/src/modules/telegram_inbound/node_label.rs` (new)
- `src-tauri/src/modules/telegram_inbound/types.rs` (one settings field)
- `02-spec/21-app/telegram-project-listing-and-ui-enhancement/02-component-spec.md` (new)
- `.ai-memory/plans/subtasks/telegram-project-listing-and-ui-enhancement/02-node-cache.md` (this file)

## What landed

- `TelegramConfig.node_name_refresh_hours` defaults to 24 via serde and `Default`.
- `node_label.rs` labels the current machine as `This machine` (green circle header, optional button `style`).
- Generated `Node-<digits>` aliases resolve through `gitmap ssh nodes export-json` (Windows creation flag `0x08000000`) and are cached in `node_display_cache.db` under `account::get_data_dir`.
- Non-generated aliases are stored and returned unchanged. `is_generated_node_alias` runs before any process spawn.
- Failures use `crate::error::record_ignored` with a justification comment. The bot token is not logged.

## Handoff

The lead adds `pub mod node_label` in `src-tauri/src/modules/telegram_inbound/mod.rs`. This worker did not edit `mod.rs`, `send_flow.rs`, or `send_listing.rs`.

## Tests left untouched

`src-tauri/src/modules/telegram_inbound/palette.rs` (`missing_palette_field_defaults_to_riseup`) deserializes `TelegramConfig` from JSON that omits the new field. Serde default supplies 24, so that test was not edited.

`src/services/telegramService.ts` still omits `node_name_refresh_hours` (same pattern as `card_palette`). A save that drops the field resets the window to 24. That file is outside this worker's ownership.

No other `TelegramConfig { ... }` literal exists in `types.rs`.
