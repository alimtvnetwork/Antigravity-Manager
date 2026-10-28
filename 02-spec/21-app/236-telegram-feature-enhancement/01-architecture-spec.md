# Architecture Specification: Telegram Feature Enhancement (crash + CSV)

## User Request (Verbatim)

Refresh status must show each instance with running/queued projects and prompts. Copy must deliver the whole prompt. CSV must download as a file. Screenshot must show IDEs, Instances, and Accounts without crashing the manager. Hover shade only. Isolated switch+restore E2E. Minor bump and release.

## Why it crashes

`src-tauri/src/modules/telegram_inbound/open_shot.rs` `WINDOW_CAPTURE_PS1` calls `ShowWindow` + `SetForegroundWindow` on the live AGM process while the Telegram daemon is inside that same process, then `CopyFromScreen`. Combined with `capture_agm_route` (`ui goto` Instances/Accounts), the manager window is stolen, WebView2 hangs, and the bot looks crashed.

CSV: when `sendDocument` fails, the fallback is another inline-keyboard of CSV **buttons** with no file — that is “it just copies the button”.

Copy: Telegram `copy_text` is capped at 256 characters. The whole prompt must be a `.txt` document (`i:cp:`).

## Invariants

- Relative Git paths only.
- Do not stop Default or `v2-3127`. Isolated `test-create` / `test-clean` only.
- FIFO restore: `ORDER BY created_at ASC, id ASC`.
- Callbacks ≤ 60 bytes.
- Hover: `.agm-hover-shade` color/shade only, no position translation.
- Attribution `@aukgit` only.
