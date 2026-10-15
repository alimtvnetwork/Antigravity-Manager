# RCA 15: Unguarded `/send` picker payloads vs Telegram's 64-byte `callback_data` cap

## Symptom (latent)
In the `/send` two-step picker, any inline button whose `callback_data`
exceeds Telegram's 64-byte cap makes the Bot API reject the **entire**
message (`BUTTON_DATA_INVALID`). The send failure was only
`record_ignored`-logged, so the user taps a machine/instance → spinner
dismisses → nothing arrives. With long machine labels or instance ids this
is a second, data-dependent way the send-prompt button "does not work".

## Root cause
Two gaps in `src-tauri/src/modules/telegram_inbound/send_flow.rs`
(`MAX_CALLBACK_BYTES = 60`):

1. The "✏️ Type it manually" button emitted
   `send:cman:{machine}|{instance}` with **no** `callback_fits` guard —
   every other picker button (`send:m:`, `send:i:`, `send:c:`) had one.
   `send:cman:` (10 bytes) + a long custom node label + instance id can
   exceed 64 bytes with real data (node alias is free text; only the OS
   hostname fallback is inherently short).
2. `machine_entries()` pushed the **local** machine label unconditionally,
   skipping the `picker_safe`/`callback_fits` guards applied to fleet-node
   aliases — a long or separator-containing local label would poison the
   whole step-1 message (and a `|`/`:` in it would additionally corrupt the
   `|`-split parsing downstream).

## How found
Audited every `callback_data` emission in `send_flow.rs` against the
64-byte cap: `send:m:` guarded in `machine_entries`, `send:i:`/
`send:c:` filtered at markup build time, `send:cman:` not guarded
anywhere. Noticed `on_manual_conversation` ignores its machine/instance
params (`let _ = (machine, instance);`) — they were redundant, since the
flow state (set at instance-pick time) already carries them — so the root
fix is removing the params, not adding another filter. Noticed the local
entry bypassed the guards fleet nodes obey.

Picker-safety (`|`/`:`) finding for the record: real instance ids
(registry, e.g. `default`) and credit-label buttons (static `cmd:/…`
payloads, no user data) cannot contain separators — no defect there. Unsafe
entries are *excluded* by `picker_safe`, never corrupted; the local-label
gap above was the only unguarded entry point.

## Fix
`src-tauri/src/modules/telegram_inbound/send_flow.rs`:
- `:254` — "✏️ Type it manually" now emits bare `send:cman` (10 bytes,
  safe by construction instead of by filter).
- `handle_send_callback` — accepts bare `cman` (params from flow state)
  **and** the legacy `cman:<machine>|<instance>` spelling, so buttons
  already delivered before the upgrade keep working.
- `:90` — new `machine_entry_usable()` helper (`picker_safe` +
  `callback_fits`); `machine_entries()` holds the local label to the same
  guards as fleet nodes, with a `tracing::warn!` when it is dropped.
- `:285` — `start_send_flow` returns a user-visible "⚠️ Send unavailable"
  message when the picker would be empty, instead of a button-less dead end.

## Verification
- `cargo fmt -- --check` exit 0.
- New tests: `send_callback_cman_accepts_short_and_legacy_payloads`
  (both wire forms drive the flow to `TypeConversation`);
  `machine_entry_usable_guards_picker_payloads` (separators, empty, and
  60-char labels rejected; 50-char accepted).
- Post-fix audit of all `callback_data` emissions: every one is now
  bounded — `send:cancel` (11 B, const), `send:m:` (guarded incl. local),
  `send:i:`/`send:c:` (filtered at markup build with real values),
  `send:cman` (10 B const).
- Full compile via CI (environment limitation as in RCA 14).

## Prevention
- Every `callback_data` emission must be bounded: guard with
  `callback_fits` **or** construct short by design. Never embed redundant
  params in a payload — if the state already holds it, don't re-send it.
- The local machine entry obeys the same guards as fleet entries; no
  unconditional pushes into picker lists.
