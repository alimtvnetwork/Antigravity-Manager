# Component Spec: Node Display Cache and Machine Labels

- **Spec path:** `02-spec/21-app/telegram-project-listing-and-ui-enhancement/02-component-spec.md`
- **Task:** Task-04
- **Owner:** Worker 02
- **Module:** `src-tauri/src/modules/telegram_inbound/node_label.rs`
- **Settings:** `src-tauri/src/modules/telegram_inbound/types.rs` (`TelegramConfig.node_name_refresh_hours` only)

## Purpose

Telegram machine buttons and headers show `This machine` for the machine that is serving the bot. Every other machine shows a proper name. A generated alias that matches `Node-` plus digits is resolved once through `gitmap ssh nodes export-json`, stored in SQLite, and reused until the refresh window elapses.

## Settings

`TelegramConfig` gains one field:

```rust
pub(crate) fn default_node_name_refresh_hours() -> u64 { 24 }

#[serde(default = "default_node_name_refresh_hours")]
pub node_name_refresh_hours: u64,
```

`Default` sets the same 24-hour value. Existing JSON that omits the field deserializes to 24. The bot token is never written to logs from this component.

## Public API

| Function | Contract |
| --- | --- |
| `is_current_machine(alias)` | True when `alias` equals `local`, `localhost`, `open_shot::local_machine_label()`, or `email_watcher::detect_machine_name()`, compared without regard to ASCII case. `detect_machine_name` is already public in `src-tauri/src/modules/email_watcher/detection.rs`. No second hostname detector is added. |
| `machine_button_text(alias)` | `This machine` for the current machine. Otherwise `display_name_for(alias)`. |
| `machine_header_html(alias)` | Current machine: `🟢 <b>This machine</b>`. Other machines: `🖥️ <b>{escaped name}</b>`. `&`, `<`, and `>` in the name are escaped. |
| `button_json(text, callback_data, style)` | JSON object with `text` and `callback_data`. When `style` is `Some`, add `style` with that string. Allowed style strings are `success`, `primary`, and `danger`. |
| `is_generated_node_alias(alias)` | Pure check: the trimmed alias matches `^Node-\d+$` with ASCII case folding. |
| `display_name_for(alias)` | Name resolution described below. |

## Name resolution

1. If the alias is not a generated `Node-` plus digits label, the alias is the display name. The cache row is still upserted.
2. If it is generated, read `node_display_cache`. Load `node_name_refresh_hours` from the Telegram config. When that load fails, use 24 and route the error through `crate::error::record_ignored` with a `// Justification:` comment. A row whose `updated_at` is still inside that window is returned as-is and GitMap is not run.
3. Otherwise run GitMap once: `std::process::Command::new("gitmap").args(["ssh", "nodes", "export-json"])`. On Windows set creation flag `0x08000000` (`CREATE_NO_WINDOW`), matching `src-tauri/src/modules/ssh_manager/sync.rs`. `Command` has no timeout.
4. Parse stdout as JSON into `SshNodeExportItem` (`src-tauri/src/modules/ssh_manager/types.rs`), including a `nodes` array, a `mainMachine` object, a bare array, and a JSON envelope.
5. Prefer the item whose alias equals the requested alias when that alias is not generated. When the requested alias is generated, prefer an item whose alias is not `Node-` plus digits and whose IP equals the generated item's IP when that IP is present.
6. When no better alias exists, use `detect_machine_name()` if that result is not a generated label. Otherwise keep the original alias.
7. Upsert the resolved name. Cache and GitMap failures are best-effort: `record_ignored` plus a justification comment. They never use a bare `let _ =` on a `Result`.

## SQLite

- File: `node_display_cache.db` inside the directory from `account::get_data_dir` (the same helper `ssh_manager/paths.rs` uses for `ssh_nodes.json`).
- Schema: `CREATE TABLE IF NOT EXISTS node_display_cache (node_key TEXT PRIMARY KEY, display_name TEXT NOT NULL, updated_at INTEGER NOT NULL)`.
- `updated_at` is Unix seconds. Lookup is case-insensitive on `node_key`.

## Tests in this module

One `#[cfg(test)]` test checks that `button_json(..., Some("success"))` includes `style: success`, and that a non-`Node-` alias is returned unchanged. That path does not spawn GitMap because `is_generated_node_alias` returns first.

## Wiring left to the lead

`src-tauri/src/modules/telegram_inbound/mod.rs` is not edited here. The lead adds `pub mod node_label` (and a re-export if callers need the functions through the parent module).

`src/services/telegramService.ts` does not list `node_name_refresh_hours`. Missing JSON still deserializes to 24 through serde, the same way `card_palette` already survives an omitted field. `palette.rs` `missing_palette_field_defaults_to_riseup` constructs `TelegramConfig` from JSON and stays valid because of that default. No other file is edited to add the field.
