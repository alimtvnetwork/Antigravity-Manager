---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "002"
title: Every add/import path uses the helper and reports skipped counts
domain: backend-rust+frontend
depends_on: 001-recipient-normalized-upsert-and-unique-index.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t09--email
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/52-email-recipient-duplicates-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/email_io.rs — bundle import
  - src-tauri/src/commands/email.rs — add/import commands
  - src-tauri/src/bin/agm.rs — email commands (~L4804-4817, ~L10256-10262, ~L10485-10508)
  - src/components/settings/EmailNotificationSettings.tsx (~L636-660)
status: pending
---

# 002 — Every add/import path uses the helper and reports skipped counts

## 1. Context
Each path inserts on its own. The user confirmed: if the notification email is already added, do not add it.

## 2. Target files and symbols
- src-tauri/src/modules/email_io.rs — bundle import
- src-tauri/src/commands/email.rs — add/import commands
- src-tauri/src/bin/agm.rs — email commands (~L4804-4817, ~L10256-10262, ~L10485-10508)
- src/components/settings/EmailNotificationSettings.tsx (~L636-660)

## 3. Steps
1. Replace direct inserts in all four places with the helper from subtask 001.
2. Import returns `ImportSummary { added, skipped_duplicates, failed }`; the CLI prints it and `--json` includes it.
3. GUI shows a short toast 'N added, M already existed' and no error for duplicates.
4. Grep the repo for other inserts into `notify_recipients` and route them as well.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only. The secrets repo is a sibling repository at `../repo-secrets/`; it has its own git history and is committed separately.
- NEVER print, log, echo, or commit a secret, token, key, password, or email. Print key names and counts only.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::email_io
npm run build
```

## 7. Done When
- [ ] No direct `INSERT INTO notify_recipients` outside the helper.
- [ ] Import summary shows the skipped count.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
