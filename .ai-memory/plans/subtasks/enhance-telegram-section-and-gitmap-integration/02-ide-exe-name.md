# Task-02: IDE exe name on prompts

## Status
PENDING (needs Task-01 audit)

## Objective
Show the IDE executable name with every prompt the bot displays.

## Requirements
- Resolve the IDE exe basename from the instance's configured exec path
  (e.g. `Antigravity-v3-3334.exe`). Fallback: instance IDE type label —
  never blank, never a raw full path dump.
- Apply everywhere a prompt is shown: send receipts, queue entries,
  history entries, rerun confirmations.
- Discover the instance exec-path source via GitMap search; follow the
  existing instance model (don't invent a new lookup).

## Acceptance
- [ ] Every prompt card ends with the IDE exe name.
- [ ] Fallback is sensible when the path is unknown.

## Constraints
- GitMap search only. Relative paths only. No silent errors.
- `cargo fmt` + `clippy` clean.
