---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "006"
title: Remove endpoint keys from `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1`
domain: scripts
depends_on: 005-agm-supabase-push-settings.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t08t11--supabase-and-vault-scripts
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: none
target_files:
  - scripts/setup-supabase.ps1 — parameter defaults for root/secondary URL and key
  - scripts/supabase-setup.ps1 — same
status: pending
---

# 006 — Remove endpoint keys from `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1`

## 1. Context
Both scripts embed default endpoint keys as parameter defaults and duplicate each other. Keys must come from the vault or environment.

## 2. Target files and symbols
- scripts/setup-supabase.ps1 — parameter defaults for root/secondary URL and key
- scripts/supabase-setup.ps1 — same

## 3. Steps
1. Delete the literal default keys and URLs; read from `-ConfigFile`, else `AGM_SUPABASE_VAULT`, else stop with a clear message naming the missing input.
2. Merge the two scripts into one (`scripts/setup-supabase.ps1`) that delegates to `agm supabase push-settings`; remove the duplicate file.
3. Note in the commit that the previously committed keys must be rotated by the owner (publishable keys still count as credentials); do not print them.
4. Check history references with `git grep -n 'sb_publishable'` and remove remaining literals.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only. The secrets repo is a sibling repository at `../repo-secrets/`; it has its own git history and is committed separately.
- NEVER print, log, echo, or commit a secret, token, key, password, or email. Print key names and counts only.
- PowerShell scripts: `Set-StrictMode -Version Latest`, `$ErrorActionPreference = 'Stop'`, check `$LASTEXITCODE` after every native call, no empty `catch`.
- Scripts must work under Windows PowerShell 5.1 and PowerShell 7 (`pwsh`).

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
git grep -n "sb_publishable" -- scripts
pwsh -NoProfile -File ./scripts/setup-supabase.ps1 -WhatIf
```

## 7. Done When
- [ ] No key literal remains in tracked scripts.
- [ ] One script remains.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
