---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "001"
title: Normalized upsert-or-skip for notification recipients
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t09--email
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/52-email-recipient-duplicates-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/email_vault_db.rs — `notify_recipients` schema (~L231-240), `add_notify_recipient` (~L630-650)
status: pending
---

# 001 — Normalized upsert-or-skip for notification recipients

## 1. Context
`add_notify_recipient` does a plain `INSERT`; nothing enforces uniqueness, so every import path can add the same address again.

## 2. Target files and symbols
- src-tauri/src/modules/email_vault_db.rs — `notify_recipients` schema (~L231-240), `add_notify_recipient` (~L630-650)

## 3. Steps
1. Add a migration step (runs at `init`) that creates a unique index on `LOWER(TRIM(email))` only when no duplicates exist; if duplicates exist, skip index creation and record a `duplicates_pending` flag (handled by subtask 003).
2. Introduce `normalize_email(&str) -> String` (trim, lowercase) in one place.
3. Change `add_notify_recipient` to return `AddOutcome::{Added(NotifyRecipient), AlreadyExists(NotifyRecipient)}`; on an existing match it returns the existing row unchanged (no group or active-flag overwrite).
4. Unit tests: same address with different case and surrounding spaces yields one row; different groups for the same address do not add a second row.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only. The secrets repo is a sibling repository at `../repo-secrets/`; it is read, not edited, by this plan.
- NEVER print, log, echo, or commit a secret, token, key, password, or email. Print key names and counts only.
- No machine alias handling anywhere (resolved ambiguity 02).

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::email_vault_db
```

## 7. Done When
- [ ] Duplicate add is a no-op that reports `AlreadyExists`.
- [ ] Tests pass.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
