---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "003"
title: `agm email dedupe` for rows that already exist
domain: backend-rust
depends_on: 001-recipient-normalized-upsert-and-unique-index.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#cli-surface-added-or-changed
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/52-email-recipient-duplicates-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/bin/agm.rs — email subcommands
  - src-tauri/src/modules/email_vault_db.rs — `find_duplicate_recipients`, `merge_duplicate_recipients`
status: pending
---

# 003 — `agm email dedupe` for rows that already exist

## 1. Context
Databases that already contain duplicates cannot get the unique index until they are merged. Deleting data needs an explicit user action.

## 2. Target files and symbols
- src-tauri/src/bin/agm.rs — email subcommands
- src-tauri/src/modules/email_vault_db.rs — `find_duplicate_recipients`, `merge_duplicate_recipients`

## 3. Steps
1. `agm email dedupe` (default dry run) lists duplicate groups as counts and masked addresses (first character plus domain), never full emails.
2. `agm email dedupe --apply` keeps the oldest row per normalized address, prefers `is_active = true` if any duplicate is active, deletes the rest in one transaction, then creates the unique index.
3. Take a timestamped copy of the DB file before `--apply`; print its relative location.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only. The secrets repo is a sibling repository at `../repo-secrets/`; it is read, not edited, by this plan.
- NEVER print, log, echo, or commit a secret, token, key, password, or email. Print key names and counts only.
- No machine alias handling anywhere (resolved ambiguity 02).
- Validate before deleting: take the backup first, abort if the copy fails.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::email_vault_db
cd src-tauri && cargo run --bin agm -- email dedupe --json
```

## 7. Done When
- [ ] Dry run changes nothing.
- [ ] `--apply` leaves one row per address and creates the index.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
