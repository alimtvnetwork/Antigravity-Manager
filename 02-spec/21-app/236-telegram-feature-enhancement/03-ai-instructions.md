# AI Instructions: Telegram Feature Enhancement (crash + CSV)

Sequence `236`. Slug `telegram-feature-enhancement`. Spec folder: `02-spec/21-app/236-telegram-feature-enhancement/`. Parent plan: `.ai-memory/plans/pending/236-telegram-feature-enhancement.md`.

This sequence repairs the live Telegram crash, CSV download, and Copy-whole-prompt regressions. It does not invent a second status pipeline.

## Do

- GitMap search only: `gitmap aum search`, `gitmap find`, `gitmap cat`, `gitmap lf`. Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `findstr`, `Get-ChildItem`, `dir`, and `ls` for repository search.
- Relative Git paths only in specs, plans, commits, changelogs, README, and release notes (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`, `src/...`). Never write absolute filesystem paths or `file:///` URIs.
- Isolated E2E only. Prefer `--dry-run`. If a live instance is required, `agm instances test-create` → operate on `NEW_ID` → `agm instances test-clean NEW_ID` only.
- Never stop Default PID or `v2-3127`. Caller-IDE commands (`agm instances stop`, `rm`, `switch`) targeting the caller instance fail-fast unless `--force` is explicit, and `--force` is allowed only on the sandbox id.
- FIFO for every enqueue, restore, dispatch, and queue query: `ORDER BY created_at ASC, id ASC`. LIFO (`ORDER BY updated_at DESC`) is forbidden.
- No unused `mut` except loop counters.
- Changelog and README attribution is `@aukgit` only (`(Thanks to @aukgit)`). No other GitHub handles.
- Route fallible work through `AppError` / `captureError` / `trackWarning`. Best-effort Telegram follow-ups still call `crate::error::record_ignored` with a `// Justification:` comment.
- Hover shade: color, border, box-shadow, filter only. Class `.agm-hover-shade`. Never `hover:-translate-y-*`. Amber hover text is `#090611`.
- Telegram `callback_data` ≤ 60 bytes. Instance namespace is `i:<op>:<id8>`. Never embed prompt text in `callback_data` or `copy_text`.
- Screenshot PowerShell must not call `ShowWindow` or `SetForegroundWindow` on the live AGM process. Bound capture to 8s and kill on timeout.
- CSV/txt sendDocument MIME is `application/octet-stream`. Empty caption first. Failed upload must send bytes as text, never buttons-only.
- Copy and Full both send the complete prompt as a `.txt` document (`i:cp:`).

## Do not

- Steal AGM window focus during capture.
- Fall back from a failed `sendDocument` to another CSV inline-keyboard with no file.
- Put `copy_text` on instance cards (256-char Bot API cap rejects the whole keyboard).
- Close, kill, or rotate the caller's IDE (Default / `v2-3127`) during implementation or E2E.
- Run `cargo test --lib`, `cargo check --bin agm`, or `tauri build` on this 12 GB host (rustc OOM). Write tests; leave full-lib compile to CI.
- Attribute anyone except `@aukgit` in `changelog.md`, `changelog_en.md`, `readme.md`, `readme_en.md`, or GitHub release notes.
- Invent a second flywheel methodology.

## GitMap search examples

```text
gitmap lf src-tauri/src/modules/telegram_inbound
gitmap aum search "WINDOW_CAPTURE_PS1" src-tauri/src/modules/telegram_inbound
gitmap aum search "fn send_telegram_document" src-tauri/src/modules/telegram_inbound
gitmap aum search "copy_text" src-tauri/src/modules/telegram_inbound/instance_cards.rs
gitmap cat src/App.css
```

## E2E (dry-run first)

1. `/status` fans out one compact HTML card per registered instance with running/queued projects and prompts.
2. Copy / Full send a downloadable `.txt` of the whole prompt (not a 256-char clipboard button).
3. Instances CSV arrives as a real file (`application/octet-stream`), not a copy-button keyboard.
4. `/screenshot` (no args) captures this machine: AGM window, desktop, Instances, Accounts, IDE windows. Manager stays responsive.
5. Hover on instance/Telegram cards changes shade only (`transform: none`).
6. Isolated `agm instances test-create` → switch+restore on `NEW_ID` restores the same prompt id FIFO → `agm instances test-clean NEW_ID`. Default and `v2-3127` stay running.

## FIFO

Prompt enqueue, restore, and queue inspection across `active_prompts`, `prompt_backups`, and `repo_prompts.db` must use `ORDER BY created_at ASC, id ASC`.

## Release

Minor bump to `4.276.0` after crash/CSV/copy lands. Changelog paths stay repository-relative. Attribution `(Thanks to @aukgit)` only. Tag must match the CHANGELOG heading character-for-character (`v4.276.0`).
