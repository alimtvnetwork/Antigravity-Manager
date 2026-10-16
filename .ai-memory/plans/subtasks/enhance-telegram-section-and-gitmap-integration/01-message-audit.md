# Task-01: Message audit (research, read-only)

## Status
PENDING

## Objective
Catalog every user-facing Telegram bot string related to prompts so the
rewrite (Task-04) is complete and nothing is missed.

## Method
Via GitMap search (no edits), find in
`src-tauri/src/modules/telegram_inbound/`:
- Destination confirmation messages ("destination locked" and friends) —
  file:line + verbatim text.
- Prompt display strings that emit raw `Id` + text — file:line + verbatim.
- Send receipts / delivery confirmations — file:line + verbatim.
- Any other prompt-related user-facing strings (history, queue, errors).

## Output
Report via send_message to the lead: a table of
string → file:line → current text → suggested professional rewrite.
Do not write files. Do not run git.

## Constraints
- Read-only. GitMap search only. Relative paths only.
