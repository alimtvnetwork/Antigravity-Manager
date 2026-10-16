# Task-03: Sent-prompt history + rerun

## Status
PENDING (needs Task-01 audit)

## Objective
Let users see previously sent prompts in Telegram and rerun them with one tap.

## Requirements
- A history view (command and/or button) listing sent prompts per instance /
  conversation: prompt preview, destination, IDE exe (Task-02), timestamp.
- Each entry gets a rerun button that re-dispatches the same prompt text to
  the same destination via the existing dispatch path (same as `/send`
  delivery — don't fork the logic).
- Rerun confirms with the professional receipt format (Task-04).
- Respect the 64-byte Telegram callback cap (RCA 15) — use compact
  callback payloads referencing stored ids, not inline text.
- Don't regress RCA 14 (dispatch) or RCA 16 (unknown-action fallback).

## Acceptance
- [ ] History lists sent prompts with previews.
- [ ] Rerun re-dispatches and confirms professionally.
- [ ] Callback payloads stay within limits.

## Constraints
- GitMap search only. Relative paths only. No silent errors.
- `cargo fmt` + `clippy` clean.
