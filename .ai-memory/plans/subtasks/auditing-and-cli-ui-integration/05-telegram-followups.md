# Task-05: Telegram follow-ups

## Status
DONE — verified by lead (15 tests added, fmt clean; unified with lead backend via HTTP POST)

## Objective
Harden and extend the Telegram bot from the previous program
(`open_shot.rs`, `send_flow.rs`, colorful `/help`).

## Requirements
- Add bot-native equivalents or passthrough for the new UI capabilities:
  `/ui goto <route>`, `/ui ls`, `/ui scroll <id>` — via the same
  underlying routes as the CLI (parity).
- Harden the fixes from RCA 11/12: review `send_flow.rs` and
  `reports_b.rs` for any remaining non-`Send` or format-string risks;
  add unit tests where the previous program left gaps.
- Keep the destination-first ordering for `/send`; keep `actor=telegram`
  in audit entries for bot-driven actions.
- "No Silent Errors"; `cargo fmt` + `clippy` clean.

## Acceptance
- [ ] `/ui` subcommands work from Telegram.
- [ ] No new CI failures; hardening tests added.

## Constraints
- GitMap search only. Relative paths only. No secrets.
