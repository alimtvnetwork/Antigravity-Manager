---
plan: 236-telegram-feature-enhancement
subtask: "01"
title: Screenshot no-focus hang + CSV sendDocument retry
domain: backend-rust
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/236-telegram-feature-enhancement/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/236-telegram-feature-enhancement/02-component-spec.md
  ai_instructions: ../../../../02-spec/21-app/236-telegram-feature-enhancement/03-ai-instructions.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/telegram_inbound/open_shot.rs
  - src-tauri/src/modules/telegram_inbound/media.rs
  - src-tauri/src/modules/telegram_inbound/daemon.rs
  - src-tauri/src/modules/telegram_inbound/tests.rs
status: completed
---

# Subtask 01 — Crash (screenshot) + CSV download

Stop the live-AGM screenshot hang and make CSV a real file. Independently revertable from 02 (`instance_cards.rs`) and 03 (`App.css` / release).

## Exclusive owned files

| File | Role |
| :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/open_shot.rs` | Local `WINDOW_CAPTURE_PS1` no focus steal; 8s `output_with_timeout` kill; `capture_agm_route` 400ms + `spawn_blocking`. |
| `src-tauri/src/modules/telegram_inbound/media.rs` | `send_telegram_document`: `application/octet-stream`; empty caption first, then caption retries. |
| `src-tauri/src/modules/telegram_inbound/daemon.rs` | **Only** export `sendDocument` path: empty caption → caption retry → 800-char text fallback `markup=None`. |
| `src-tauri/src/modules/telegram_inbound/tests.rs` | **Only** additive export / MIME / no-buttons-only assertions. |

Do not edit `instance_cards.rs`, `src/App.css`, changelog, or README. Do not bump.

## Screenshot (crash)

Root cause: `WINDOW_CAPTURE_PS1` called `ShowWindow` + `SetForegroundWindow` on the live AGM process (same process as the Telegram daemon) then `CopyFromScreen` — hangs WebView2.

Required:

- Remove `[AgmWinShot]::ShowWindow` and `[AgmWinShot]::SetForegroundWindow` from **local** `WINDOW_CAPTURE_PS1` only.
- Keep `GetWindowRect` + `CopyFromScreen`; fall back to primary-screen bounds if the handle is empty.
- `output_with_timeout(..., Duration::from_secs(8))`; on timeout `child.kill()` via `record_ignored` + `// Justification:`.
- `capture_agm_route`: `ui goto` → 400ms sleep → `tokio::task::spawn_blocking(capture_local_once)`.
- `/screenshot` with no args stays local. Fleet requires `/screenshot all`. Do not change that contract.

Unit test in `open_shot.rs`: `local_capture_script_does_not_steal_focus` asserts the two Win32 calls are absent from `WINDOW_CAPTURE_PS1`.

Leave `REMOTE_CAPTURE_PS1` and IDE `AgmIdeShot` focus calls alone (not the live-AGM hang).

## CSV

Root cause: `sendDocument` failed, then the daemon fell back to CSV **buttons** with no file.

Required sequence in `daemon.rs` export arm:

1. `send_telegram_document(..., caption="", markup=None)`.
2. On failure, retry with `doc.caption` still `markup=None`.
3. On failure, 800-char UTF-8 preview as a **text** message with `markup=None` (real error in the body). Never attach the CSV inline keyboard to a failed upload.

`media.rs`: MIME for `.csv` (and non-json/non-txt) is `application/octet-stream`. Helper tries empty caption before HTML caption. Do not swallow the last HTTP error.

## Coding rules

- GitMap-only search. Relative paths only.
- No unused `mut` except loop counters.
- No silent errors.

## Verification (required)

Write tests; do **not** run `cargo test --lib` on this host.

- [ ] `WINDOW_CAPTURE_PS1` has no `[AgmWinShot]::ShowWindow` / `SetForegroundWindow`.
- [ ] Capture helper bounded to 8s and killed on timeout.
- [ ] `capture_agm_route` sleeps 400ms then `spawn_blocking`.
- [ ] CSV upload empty-caption first; caption retry; 800-char text fallback; `markup=None`.
- [ ] MIME `application/octet-stream` for CSV.
- [ ] Default and `v2-3127` still running.
- [ ] Exclusive file box respected.

## Done when

- [ ] Local screenshot cannot steal AGM focus.
- [ ] Failed CSV still delivers bytes or an 800-char error text — never buttons-only.
- [ ] No version bump. No `instance_cards.rs` / `App.css` edits.
