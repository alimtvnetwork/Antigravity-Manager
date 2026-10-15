# RCA 14: Telegram `cmd:` buttons silently dead — daemon `//cmd` normalization mismatch

## Symptom
All three Telegram inline buttons — 🚀 Prompt Wizard (`cmd:/send`),
🏷️ Instances (`cmd:/instances`), 💰 Account Balances (`cmd:/balance`) —
"do not work properly" at runtime: tap → the loading spinner dismisses →
**nothing happens**. No reply, no error. Task-01's static gap analysis found
every handler wired (no dead callbacks), so the breakage was in live dispatch.

## Root cause
`daemon.rs` had **two** callback dispatch sites with **divergent**
normalization:

- `poll_telegram_updates_once` (one-shot/manual poll path): 
  `format!("/{}", stripped.trim_start_matches('/'))` → `cmd:/send` → `/send` ✅
- `start_telegram_daemon` (the **live** background loop, started at app
  setup `src-tauri/src/lib/setup_app.rs:220` and `src-tauri/src/commands/telegram.rs:21`):
  `format!("/{}", stripped)` — no slash stripping → `cmd:/send` → `//send` ❌

`process_telegram_command_text("//send")` strips only ONE leading slash
(`cmd_no_slash = "/send"`), matches no command arm, and its `_` arm returns
`None` (not a seq code, no `cmd:`/`ff:`/node prefix). The markup match on
`/send` also falls to `_ => None`. So the daemon computed `(None, None)` and
sent nothing — while `answerCallbackQuery` still fired and dismissed the
spinner. The user-visible result: a tap that dies silently.

The same bug hit every slash-form button (`cmd:/fleet`, `cmd:/accounts`,
`cmd:/nodes`, `cmd:/status`, `cmd:/export …`, `cmd:/ssh …`, `cmd:/ff`).
Payload conventions were already inconsistent in the codebase (`cmd:/fleet`
vs `cmd:fleet` in the ping markup), which is why one site "worked" and the
other didn't depending on spelling.

## How found
Per the gap-analysis flag, compared the two dispatch expressions
character-by-character; confirmed `start_telegram_daemon` is the live runtime
path (setup + telegram command callers); traced `//send` through
`process_telegram_command_text` (`processing.rs`) to the `None` return and
through the markup match to `_ => None`; verified `answerCallbackQuery` is
sent unconditionally afterwards — the exact "spinner then silence" UX.

## Fix
`src-tauri/src/modules/telegram_inbound/daemon.rs`:
- `:16` — new `normalize_cmd_callback_payload()`: strips **all** leading
  slashes after `cmd:`, so both `cmd:/send` and `cmd:send` normalize to
  `/send`; non-`cmd:` payloads pass through untouched.
- `:489` (one-shot site) and `:689` (daemon site) — both call sites now use
  the single shared function; the inline duplicates are gone.
- `:689` — the daemon callback path now routes through the same unified
  `deliver_ui_or_command` pipeline the one-shot path uses (previously it
  called only `resolve_telegram_command_reply_and_markup`, which also meant
  `cmd:/export …` buttons got text previews but never the document in the
  daemon). `prepare_local_ui_action` returns `None` for `/send`/`/instances`/
  `/balance`, so behavior for the three buttons is unchanged apart from
  working.
- `:710` — unknown-`cmd:` fallback (see RCA 16).

## Verification
- `cargo fmt -- --check` exit 0 on `src-tauri`.
- New unit test `test_normalize_cmd_callback_payload` (`tests.rs`): slash
  form, non-slash form, doubled slashes, payload-with-args, and
  non-`cmd:` passthrough.
- Full `cargo clippy`/compile not possible on this machine (missing
  `gdk-3.0` system lib; `apt` has no install candidate — environment
  limitation documented in RCA 11); CI is the compile gate.
- Audit behavior untouched: `actor=telegram` entries in `ui_remote.rs`
  not modified.

## Prevention
- Never duplicate normalization inline at two call sites — one shared
  function, tested for both spellings.
- When adding new `cmd:` buttons, either spelling now works; keep the
  dominant `cmd:/…` convention.
- Any future dispatch-site addition must route through
  `deliver_ui_or_command`, not a partial pipeline.
