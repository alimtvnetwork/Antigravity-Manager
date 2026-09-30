---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "009"
title: Shared guard helpers and vault self-check
domain: scripts
depends_on: 007-connect-supabase-ps1-hardening.md, 008-push-settings-ps1-fixes.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#acceptance-criteria
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: none
target_files:
  - ../repo-secrets/00-lib/agm-common.ps1 (new)
  - ../repo-secrets/verify-vault.ps1 (new)
status: pending
---

# 009 — Shared guard helpers and vault self-check

## 1. Context
The user asked for scripts that do not produce these failures again. One shared helper keeps the rules in one place.

## 2. Target files and symbols
- ../repo-secrets/00-lib/agm-common.ps1 (new)
- ../repo-secrets/verify-vault.ps1 (new)

## 3. Steps
1. Create `agm-common.ps1` exporting: `Get-VaultRoot`, `Get-AgmCli`, `Invoke-AgmChecked` (runs a native command, throws on non-zero with the step name), `Assert-VaultFile` (exists, parses JSON, required keys present).
2. Refactor `connect-supabase.ps1` and `push-settings.ps1` to use the helpers (no duplicated logic).
3. Create `verify-vault.ps1` that checks each expected vault file exists and has valid JSON or SQLite header, and runs `agm which-format` on JSON files; prints a one-line pass/fail per file using names only; exit non-zero on any failure.
4. List both scripts in the vault readme.

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
pwsh -NoProfile -File ../repo-secrets/verify-vault.ps1
```

## 7. Done When
- [ ] Both scripts use the shared helpers.
- [ ] `verify-vault.ps1` passes on the current vault and fails on a doctored file.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
