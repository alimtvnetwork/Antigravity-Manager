# Task-04: Professional message rewrite

## Status
PENDING (needs Task-01 audit)

## Objective
Rewrite the bot's prompt-related messages into professional human-readable
chat messages.

## Requirements
- Replace "destination locked"-style confirmations with human phrasing
  stating what will happen: which prompt (preview), to which destination
  (machine → instance → conversation), IDE exe name, and what to do next.
- Replace raw `Id` + text dumps with formatted cards: title, prompt
  preview, destination, IDE exe, timestamp, status.
- Send receipts: what was sent, where, when, and how to rerun/verify.
- Error messages: plain-language explanation + what the user can do
  (never a stack trace or silent failure).
- Keep Telegram HTML formatting consistent and professional throughout.
  Centralize shared phrasing in `formatting.rs` where sensible.

## Acceptance
- [ ] No "destination locked" or raw Id dumps remain in prompt flows.
- [ ] Every message reads like a helpful assistant.

## Constraints
- GitMap search only. Relative paths only.
- Coordinate with Task-02/03 on shared files (lead arbitrates).
