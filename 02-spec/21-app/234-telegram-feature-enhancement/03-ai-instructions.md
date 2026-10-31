# AI Instructions: Telegram Feature Enhancement

## Do

- GitMap search only (`gitmap aum search`, `gitmap find`, `gitmap cat`). No rg/ripgrep/grep/Select-String.
- Relative Git paths only in specs, plans, commits, changelogs, README, release notes.
- Isolated E2E instance via `agm instances test-create`. Never stop Default PID or v2-3127.
- FIFO enqueue for Telegram rerun/automation.
- Clamp `copy_text` to 256. Full prompt = `sendDocument`.
- CSV = real file download via `sendDocument`.
- Hover: color/shade/shadow animation only; no translate on hover. Yellow hover text is black.

## Do not

- Invent a second prompt pipeline. Read `list_all_prompts` / instance registry / live projects.
- Embed full prompt text in `callback_data`.
- Leave `hover:-translate-y-0.5` on instance cards.
- Close the caller's IDE. Use `--force` only on the sandbox id.
- Attribute anyone except `@aukgit` in changelogs.

## E2E (isolated)

1. `agm instances test-create` → `NEW_ID`.
2. Enqueue a distinct running prompt onto `NEW_ID`.
3. Switch that instance to a standby account.
4. Assert Stage 1 backed up from `NEW_ID` (not Default) and Stage 5 restored/dispatched the same prompt id.
5. `agm instances test-clean NEW_ID` only.
6. Telegram unit tests: markup ≤ 60 bytes, copy_text ≤ 256, instances CSV columns, export detector.

## Release

Minor bump `4.274.0` → `4.275.0`. Changelog + README both languages. `gitmap cpr`. Tag matches CHANGELOG heading.
