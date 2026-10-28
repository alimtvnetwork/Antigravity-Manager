# Component Specification

## Screenshot

- Do not `ShowWindow` / `SetForegroundWindow` the AGM window during capture.
- Bound PowerShell capture to 8s; kill on timeout.
- `/screenshot` with no args stays local. Fleet requires `/screenshot all`.
- Status header **All shots** → `cmd:/screenshot` (local mosaic: desktop, Instances, Accounts, IDEs).

## CSV

- `send_telegram_document` prefers `application/octet-stream` and retries with an empty caption before any button fallback.
- Failed upload must still attach bytes or show the real error — never buttons-only.

## Instance cards

- 📋 Copy and 📄 Full both send the full prompt as a `.txt` document (`i:cp:`).
- Do not put `copy_text` on the instance card (256-char cap rejects the whole keyboard).
- Keep 100-word expandable preview; at most 2 running + 2 queued snippets so HTML stays under 3900.

## Files

- `src-tauri/src/modules/telegram_inbound/open_shot.rs`
- `src-tauri/src/modules/telegram_inbound/media.rs`
- `src-tauri/src/modules/telegram_inbound/instance_cards.rs`
- `src-tauri/src/modules/telegram_inbound/daemon.rs`
- `src-tauri/src/modules/telegram_inbound/tests.rs`
