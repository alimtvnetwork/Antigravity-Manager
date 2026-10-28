# Master Plan 236: Telegram crash, CSV download, instance cards

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

- [ ] Screenshot PowerShell does not call `ShowWindow` / `SetForegroundWindow` on AGM
- [ ] Capture helper times out at 8s instead of hanging WebView2
- [ ] `/status` one card per instance with running/queued projects and prompts
- [ ] Copy and Full send a `.txt` document of the whole prompt (no `copy_text` on cards)
- [ ] CSV `sendDocument` uses `application/octet-stream`; failed upload shows bytes, not buttons
- [ ] Hover shade animates color/shadow/filter only (`transform: none`)
- [ ] Isolated test-create switch+restore; Default and `v2-3127` untouched
- [ ] Minor bump `4.276.0` with `@aukgit` only and relative Git paths

## Non-goals

- Do not invent a second improvements flywheel
- Do not stop Default or `v2-3127`
- Do not run `cargo test --lib` or full `tauri build` locally (OOM)
