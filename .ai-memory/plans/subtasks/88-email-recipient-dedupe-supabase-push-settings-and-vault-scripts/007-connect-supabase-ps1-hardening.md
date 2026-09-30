---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "007"
title: Harden `connect-supabase.ps1` in the vault repo
domain: scripts
depends_on: 005-agm-supabase-push-settings.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t08t11--supabase-and-vault-scripts
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - ../repo-secrets/03-supabase/connect-supabase.ps1
  - ../repo-secrets/connect-supabase.ps1 (root copy; replace with a thin wrapper that dot-sources the real script)
status: pending
---

# 007 — Harden `connect-supabase.ps1` in the vault repo

## 1. Context
The script computes the repo root one level too high, falls back to a hardcoded absolute path, never checks exit codes, never sets the alias, and does not verify after loading.

## 2. Target files and symbols
- ../repo-secrets/03-supabase/connect-supabase.ps1
- ../repo-secrets/connect-supabase.ps1 (root copy; replace with a thin wrapper that dot-sources the real script)

## 3. Steps
1. Resolve the vault root from `$PSScriptRoot` (walk up until the marker file `readme.md` with `03-supabase` sibling is found); remove every absolute path.
2. Locate the `agm` CLI through a shared function (`Get-Command`, then `AGM_CLI` env, then the standard install directory per OS).
3. Add parameters `-Alias`, `-Sync`, `-Json`, `-WhatIf`; when the CLI supports it call `agm supabase push-settings` instead of separate calls.
4. Check `$LASTEXITCODE` after each `agm` call and throw with the failing step name; final step runs `agm supabase status` and `agm supabase test` and fails the script on non-zero.
5. Replace the ad hoc discovery list with the order from ambiguity 02; report which file was used by relative name.
6. Put the one-liner example in the script header and the folder readme.

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
pwsh -NoProfile -File ../repo-secrets/03-supabase/connect-supabase.ps1 -WhatIf
```

## 7. Done When
- [ ] No absolute path remains.
- [ ] Every native call's exit code is checked.
- [ ] Script exits non-zero when verification fails.

## 8. Ambiguities and interim defaults
- Ambiguity 02 defaults apply.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
