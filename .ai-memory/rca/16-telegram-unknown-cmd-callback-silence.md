# RCA 16: Unrecognized `cmd:` button taps died with zero feedback

## Symptom
Any `cmd:` inline-button tap whose payload did not resolve to a known
command produced **zero** user feedback: the spinner dismissed
(`answerCallbackQuery` always fires) and no message was ever sent. Before
RCA 14's fix this was exactly what the three main buttons did (`//send` →
`None`); after the fix it remains the behavior for genuinely unknown or
stale payloads (typo'd `callback_data`, buttons from an older menu, a
command removed since the menu was sent).

## Root cause
The daemon's callback branch only sent a reply `if let Some(reply)`; the
`None` arm fell straight through to `answerCallbackQuery`. A tapped button
is an explicit user action — answering it with silence violates the
no-silent-errors house rule at the UX surface (the error-module side was
already covered by `record_ignored`, but nothing user-visible existed).

## How found
Item-3 trace of each button's full path: for every fallible step, checked
whether failure reaches `AppError`/tracing/`record_ignored` **or** a bot
reply. The `None`-reply arm of the callback dispatch had neither a reply
nor — for unknown payloads — any meaningfully surfaced signal.

## Fix
`src-tauri/src/modules/telegram_inbound/daemon.rs:710` — when the unified
pipeline yields no reply for a `cmd:`-prefixed callback, the bot now sends
a user-visible notice:

> ⚠️ **Unknown button action:** `<payload>`
> The menu may be stale — send a fresh command to get new buttons.

The payload is HTML-escaped via `clean_for_telegram_html`; the send itself
is `record_ignored`-logged with a `// Justification:` comment. Scoped to
`cmd:` payloads only, so plain-text messages and non-button callbacks keep
their existing (intentionally quiet) behavior.

## Verification
- `cargo fmt -- --check` exit 0.
- The three known buttons resolve through the pipeline (covered by RCA 14's
  normalization test); the fallback triggers only when the pipeline returns
  `None` for a `cmd:` tap — verified by code path inspection.
- Full compile via CI (environment limitation as in RCA 14).

## Prevention
- Every new callback payload family must have a user-visible fallback;
  `deliver_ui_or_command` returning `None` on a `cmd:` tap can no longer be
  silent.
- When wiring new buttons, prefer reusing existing commands over inventing
  payloads that the dispatcher doesn't know.
