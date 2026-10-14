# Task-07: Telegram — `/send` flow + colorful `/help`

## Status
DONE

## Objective
Implement the two-step `/send` flow and a professional colorful `/help` in
`src-tauri/src/modules/telegram_inbound/` (+ `notification_hub/` if the
help renderer lives there).

## `/send` two-step flow
1. User issues `/send`. Bot replies with a destination picker: machine →
   instance → conversation (inline keyboard or numbered list).
2. User picks the destination. Bot confirms the destination explicitly
   ("Prompt will go to W3 → kino → <conversation>. Now send the prompt
   text.").
3. User sends the prompt text. Bot dispatches it to that exact conversation
   via the existing prompt-dispatch path.
4. Bot confirms delivery by navigating to the conversation and attaching a
   screenshot snippet as proof.

Destination MUST be explicit before any prompt text is accepted. Never
guess the destination.

## `/help` redesign
- Telegram HTML parse mode; grouped commands with emoji/color accents;
  professional tone.
- Every command gets a copy-paste button: an inline keyboard button whose
  text is the command template (e.g. `/send`) so the user taps, copies,
  edits the prompt part, and sends. Where Telegram limits apply, fall back
  to `<code>` blocks which are tap-to-copy.
- Must list: `/open`, `/screenshot`, `/send`, `/help`, plus existing
  commands (discover them via GitMap search — do not drop any).

## Acceptance
- [ ] `/send` enforces destination-first ordering.
- [ ] Delivery confirmation includes a screenshot.
- [ ] `/help` renders colorful HTML with copy-paste affordances.
- [ ] No existing command lost from help.
- [ ] `cargo fmt` clean; no silent errors.

## Constraints
- GitMap search only. Relative paths only. No secrets.
