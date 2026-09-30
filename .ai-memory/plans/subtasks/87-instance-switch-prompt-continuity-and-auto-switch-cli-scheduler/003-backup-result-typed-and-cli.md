---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "003"
title: Typed backup result and `agm brp --instance`
domain: backend-rust
depends_on: 002-per-instance-prompt-discovery.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t07--instance-switch
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/50-instance-switch-loses-running-prompt-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/repo_db.rs — `backup_running_prompts` (~L748+)
  - src-tauri/src/modules/backup_prompts_db.rs
  - src-tauri/src/bin/agm.rs — `brp` command
status: pending
---

# 003 — Typed backup result and `agm brp --instance`

## 1. Context
The user wants the running prompt backed up to SQLite through the CLI before anything is closed. Current backup swallows errors and cannot be asserted.

## 2. Target files and symbols
- src-tauri/src/modules/repo_db.rs — `backup_running_prompts` (~L748+)
- src-tauri/src/modules/backup_prompts_db.rs
- src-tauri/src/bin/agm.rs — `brp` command

## 3. Steps
1. Return `BackupOutcome { found, backed_up, failed: Vec<..> }` from `backup_running_prompts(scope)`; mark each prompt `backed_up` in `repo_prompts.db` and write the row into `backup-prompts.db` in one transaction.
2. Seed `.antigravity_resume_task.json` atomically (write temp file then rename) in each bound workspace root.
3. `agm brp --instance <id> [--json]` prints the outcome; exit non-zero when `failed` is non-empty.
4. Callers must handle the result; add `#[must_use]` on the type.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; new logic goes in its own module or function, not appended to the large files.
- Relative paths only; cross-platform (Windows, macOS, Linux) path and process handling.
- No silent failure: results are typed and propagated; never `let _ =` on a backup, close, or inject result.
- Match processes only by `--user-data-dir` of the target instance, never by executable name.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::repo_db
cd src-tauri && cargo run --bin agm -- brp --instance default --json
```

## 7. Done When
- [ ] Backup outcome is typed, persisted, and visible in JSON.
- [ ] Failures exit non-zero.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
