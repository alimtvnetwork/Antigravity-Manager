# AI Instructions: Add New Option for 100 Improvements

Sequence `235`. Slug `add-new-option-for-100-improvements`. Spec folder: `02-spec/21-app/235-add-new-option-for-100-improvements/`. Parent plan: `.ai-memory/plans/pending/235-add-new-option-for-100-improvements.md`.

Reuse the existing Category 30 flywheel. Do not invent a second methodology.

## Do

- GitMap search only: `gitmap aum search`, `gitmap find`, `gitmap cat`, `gitmap lf`. Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, `findstr`, `Get-ChildItem`, `dir`, and `ls` for repository search.
- Relative Git paths only in specs, plans, commits, changelogs, README, and release notes (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`, `src/...`, `01-prompts/...`). Never write absolute filesystem paths or `file:///` URIs.
- Isolated E2E only. Prefer `--dry-run` so no prompt is dispatched and no IDE is stopped. If a live dispatch is required, `agm instances test-create` → operate on `NEW_ID` → `agm instances test-clean NEW_ID` only.
- Never stop Default PID or `v2-3127`. Caller-IDE commands (`agm instances stop`, `rm`, `switch`) targeting the caller instance fail-fast unless `--force` is explicit, and `--force` is allowed only on the sandbox id.
- FIFO for every enqueue, restore, dispatch, and queue query: `ORDER BY created_at ASC, id ASC`. LIFO (`ORDER BY updated_at DESC`) is forbidden.
- No unused `mut` except loop counters. Assign every variable once. Prefer `let` over `let mut`. Build result objects by copy or constructor, not in-place mutation.
- Changelog and README attribution is `@aukgit` only (`(Thanks to @aukgit)`). No other GitHub handles.
- Route fallible work through `AppError` / `captureError` / `trackWarning`. No silent `let _ =`, `.ok()`, empty `catch`, or fake success.
- Load prompt bodies from repo-relative paths under `01-prompts/30-improvements/`. Dispatch through existing `agm prompt send` / FIFO enqueue. Wrap live execution with `01-prompts/14-execute/02-execute-parent-task-with-n-steps-v6.md` plus `.agents/skills/implement-next-10-improvements/`.
- Hover shade: color, border, box-shadow, filter only. Class `.agm-hover-shade`. Never `hover:-translate-y-*`. Amber hover text is `#090611`.
- Telegram `callback_data` ≤ 60 bytes. Use the `im:` namespace (`im:pl`, `im:n10`, `im:st`, `im:ck`). Do not embed prompt text in callbacks.
- `--dry-run` must not dispatch prompts, must not write live queue rows, and must still persist a dry-run-annotated ledger snapshot or print the planned ledger without claiming a batch as `running`.

## Do not

- Invent a second improvements pipeline, a second prompt catalog, or a parallel backlog outside `.ai-memory/plans/pending/` and `.ai-memory/plans/subtasks/`.
- Close, kill, or rotate the caller's IDE (Default / `v2-3127`) during implementation or E2E.
- Run `cargo test --lib`, `cargo check --bin agm`, or `tauri build` on this 12 GB host (rustc OOM). Write tests; leave full-lib compile to CI. Targeted `cargo fmt -- --check` and `cargo clippy` remain allowed when the implementer can finish them without OOM.
- Attribute anyone except `@aukgit` in `changelog.md`, `changelog_en.md`, `readme.md`, `readme_en.md`, or GitHub release notes.
- Leave unused `mut`, unused imports, or unused variables except a loop index.
- Swallow errors. Best-effort Telegram follow-ups still call `crate::error::record_ignored` with a `// Justification:` comment.
- Cross-edit another subtask's exclusive files. Subtasks are independently revertable.

## GitMap search examples

```text
gitmap lf src-tauri/src/modules
gitmap find "improvements*" src-tauri
gitmap aum search "fn cmd_improvements" src-tauri/src/bin/agm
gitmap aum search "setMyCommands" src-tauri/src/modules/telegram_inbound
gitmap cat 01-prompts/30-improvements/07-implement-next-10-improvements.md
```

## E2E (dry-run first)

1. `agm improvements --help` lists `plan`, `next-10`, `status`, `check`, `detail`.
2. `agm improvements status --json` returns a PascalCase envelope with `Kind`, `Wave`, `Total`, `BatchSize`, `NextBatch`, `Batches`.
3. `agm improvements plan --dry-run` materializes or prints 10 batch stubs and does **not** call `agm prompt send`.
4. `agm improvements next --dry-run` (alias `next-10`) names the next pending batch and does **not** dispatch `07-implement-next-10-improvements.md`.
5. Default and `v2-3127` remain running. No `agm instances stop` / `rm` / `switch` against the caller.
6. If a live prompt is ever sent: `agm instances test-create` → dispatch onto `NEW_ID` FIFO → `agm instances test-clean NEW_ID` only.

## FIFO

Prompt enqueue, restore, and queue inspection across `active_prompts`, `prompt_backups`, and `repo_prompts.db` must use `ORDER BY created_at ASC, id ASC`. Flywheel batch claim is also FIFO: lowest `batch_index` with status `pending` wins.

## Release

Do not bump during subtasks 01–03. Parent E2E task records evidence only. Changelog paths stay repository-relative. Attribution `(Thanks to @aukgit)` only. Tag, if later requested, must match the CHANGELOG heading character-for-character.
