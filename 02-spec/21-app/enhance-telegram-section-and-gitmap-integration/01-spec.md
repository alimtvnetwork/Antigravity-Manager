# Spec: Enhance Telegram Section and GitMap Integration

## slug
enhance-telegram-section-and-gitmap-integration

## Status
ACTIVE — plan enqueued at `.ai-memory/plans/enhance-telegram-section-and-gitmap-integration.md`

## 1. Objective

Make the Telegram bot's prompt section professional and human-readable, show
the IDE exe name with prompts, and let users see + rerun previously sent
prompts from Telegram. Finish with a minor bump and CI green.

## 2. Background — read before coding
- `.ai-memory/what-to-read.md` (Telegram `/ui`, `/shot`, `/target` notes;
  E2E blind pack).
- `.ai-memory/rca/14-telegram-cmd-callback-double-slash.md`,
  `15-telegram-picker-callback-64byte-cap.md`,
  `16-telegram-unknown-cmd-callback-silence.md` (recent Telegram button
  fixes — don't regress them).
- `src-tauri/src/modules/telegram_inbound/formatting.rs` (message
  formatting), `send_flow.rs` (two-step send), `reports_b.rs` (help/cards).
- GitMap integration: the bot already exposes `/gitmap` commands; keep
  parity.

## 3. Requirements

### 3.1 IDE exe name with prompts
- When the bot shows a prompt (sent, queued, or history), display the IDE
  executable name at the end (e.g. `Antigravity-v3-3334.exe`), so the user
  knows exactly which IDE binary the prompt went to / will go to.
- Resolve from the instance's configured exec path (basename), with a
  sensible fallback when unknown — never blank, never a raw path dump.

### 3.2 Sent-prompt history + rerun from Telegram
- The bot must be able to show prompts the user has sent (per instance /
  conversation), and each entry gets a rerun button that re-dispatches the
  same prompt text to the same destination.
- Rerun must follow the existing dispatch path (same as `/send` delivery)
  and confirm with the professional receipt format from 3.3.

### 3.3 Professional human-readable messages
- Rewrite bot messages that currently read like debug output:
  - "destination locked" style confirmations → professional human phrasing
    stating what will happen / what was checked / what was sent, e.g.
    which prompt, to which instance/conversation, and the IDE exe name.
  - The current `Id` + raw text dump → a formatted card: title, prompt
    preview, destination (machine → instance → conversation), IDE exe,
    timestamp, status.
- Every user-facing string should read like a helpful assistant, not a log
  line. Keep Telegram HTML formatting professional and consistent.

### 3.4 Release
- Minor bump via the repo release flow (`npm run bump minor`, changelog
  with `(Thanks to @aukgit)` only, README sync), tag, push tag,
  `gh release create -R alimtvnetwork/Antigravity-Manager`.
- CI watched until green; fix failures (minor bumps authorized).

## 4. Out of scope
- `.github/workflows/` changes (standing rule).
- Deletions (standing rule).
- Progress bar (standing off-limits).

## 5. Constraints (non-negotiable)
- GitMap-only search. Relative paths only. Lowercase filenames.
- Targeted `git commit <exact paths>` + `git push` by the lead only.
  NEVER `gitmap cpf`/`cpb`.
- No silent errors (AGENTS.md). FIFO invariant untouched.
- `cargo fmt` clean; clippy clean; `npm run build` for frontend changes.
- Don't regress RCAs 14–16 (callback dispatch, 64-byte cap, unknown-action
  fallback).
