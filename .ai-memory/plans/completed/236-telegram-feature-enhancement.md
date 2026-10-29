# Master Plan 236: Telegram crash, CSV download, instance cards

## Status: COMPLETED
Moved from `pending/` on 2026-10-29. Shipped `1ce618741`, tagged `v4.276.0` (`b2cfc5596`). Live proof same day on installed `v4.276.0`. Leftovers: Telegram card Copy/Full (`i:cp:`) and inbound `/screenshot` mosaic were not live-clicked (Computer Use `sidecar_unavailable`). JSON switch honesty is issue 66, not this plan.

Slug: `telegram-feature-enhancement`. Spec: `02-spec/21-app/236-telegram-feature-enhancement/`.

## User Request (Verbatim)

Refresh status must show each instance with running/queued projects and prompts. Copy must deliver the whole prompt. CSV must download as a file. Screenshot must show IDEs, Instances, and Accounts without crashing the manager. Hover shade only. Isolated switch+restore E2E. Minor bump and release.

## Owned files

- `src-tauri/src/modules/telegram_inbound/open_shot.rs` — no self-focus; 8s timeout; local mosaic
- `src-tauri/src/modules/telegram_inbound/instance_cards.rs` — Copy/Full = `i:cp:` document
- `src-tauri/src/modules/telegram_inbound/media.rs` — octet-stream + empty caption
- `src-tauri/src/modules/telegram_inbound/daemon.rs` — CSV retry then text fallback, never buttons-only
- `src/App.css` — `.agm-hover-shade` color/shade only

## Acceptance

- [x] Screenshot PowerShell does not call `ShowWindow` / `SetForegroundWindow` on AGM (`open_shot.rs`)
- [x] Capture helper times out at 8s instead of hanging WebView2
- [x] `/status` one card per instance with running/queued projects and prompts (`agm telegram observe` delivered)
- [x] Copy and Full send a `.txt` document of the whole prompt (no `copy_text` on cards) — source shipped; Telegram button click not live-proven
- [x] CSV `sendDocument` delivered a real file (`agm-instances-e2e.csv`, empty caption, message 1403)
- [x] Hover shade animates color/shadow/filter only (`transform: none`)
- [x] Isolated `test-create` switch+restore; Default PID and `v2-3127` untouched. Bound email changed only on the non-JSON switch path.
- [x] Minor bump `4.276.0` with `@aukgit` only and relative Git paths

## Non-goals

- Do not invent a second improvements flywheel
- Do not stop Default or `v2-3127`
- Do not run `cargo test --lib` or full `tauri build` locally (OOM)
