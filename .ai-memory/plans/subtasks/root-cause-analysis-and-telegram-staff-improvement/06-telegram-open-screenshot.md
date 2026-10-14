# Task-06: Telegram — `/open` and `/screenshot` commands

## Status
PENDING

## Objective
Implement two new Telegram bot commands in
`src-tauri/src/modules/telegram_inbound/` (command parsing/dispatch) with
any shared helpers in the appropriate module:

- `/open [machine]` — open the AGM UI on the target machine. If AGM is not
  running, launch it; if running, bring its window to the foreground.
  With no argument, targets the default/current machine.
- `/screenshot [machine]` — capture the current screen of the target
  machine (all machines if omitted) and send the image(s) back to the
  Telegram chat. If AGM is not open, open it first, then screenshot.

## Requirements
- Machine/instance targeting uses the existing fleet model
  (`src-tauri/src/modules/` fleet/machine/instance types) — discover via
  GitMap search; do not invent new identity schemes.
- Every fallible op routes errors into the error module (repo "No Silent
  Errors" rule): `crate::error::record_ignored()` + `// Justification:`
  comment for best-effort; propagate with context otherwise.
- Bot replies confirm what happened ("Opened AGM on W3", "Screenshot from
  W3/W4 attached") or the real error text.
- `cargo fmt` clean on touched files.

## Acceptance
- [ ] `/open` launches/focuses AGM per machine.
- [ ] `/screenshot` returns an image per targeted machine.
- [ ] Errors surfaced with real messages, never silent success.
- [ ] Targeted checks pass (fmt; clippy on touched files if Rust).

## Constraints
- GitMap search only. Relative paths only. No secrets in code.
- Workers never run git; lead commits.
