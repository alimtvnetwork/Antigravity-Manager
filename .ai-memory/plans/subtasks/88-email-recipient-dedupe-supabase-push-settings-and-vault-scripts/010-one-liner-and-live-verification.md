---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "010"
title: Verified one-liner for Supabase on this machine
domain: scripts+e2e
depends_on: 009-shared-script-guard-helpers.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#acceptance-criteria
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - ../repo-secrets/03-supabase/readme.md — 'Run it' section
  - ../repo-secrets/02-antigravity-and-event-manager/readme.md
status: pending
---

# 010 — Verified one-liner for Supabase on this machine

## 1. Context
The user wants a one-liner PowerShell file that loads the Supabase config into the tool, aliases the machine, and is verified to work.

## 2. Target files and symbols
- ../repo-secrets/03-supabase/readme.md — 'Run it' section
- ../repo-secrets/02-antigravity-and-event-manager/readme.md

## 3. Steps
1. Write the one-liner in the readme: `pwsh -NoProfile -File .\03-supabase\connect-supabase.ps1 -Alias <name> -Sync` (use `./` separators in the repo docs where cross-platform).
2. Run it on this machine. Capture: exit code, the step list, `agm supabase status` output (alias and connected endpoint names only).
3. Run it a second time to prove idempotence (no duplicate endpoint, alias unchanged).
4. Run it once with the CLI missing from `PATH` to prove the failure message and non-zero exit.
5. Record the verified command and the observed result (without secrets) in the plan status and in the learned memory entry.

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
pwsh -NoProfile -File ../repo-secrets/03-supabase/connect-supabase.ps1 -Alias <name> -Sync
agm supabase status
```

## 7. Done When
- [ ] Exit code 0 on the live run.
- [ ] Second run is idempotent.
- [ ] Missing-CLI run exits non-zero with a clear message.

## 8. Ambiguities and interim defaults
- Ambiguity 02 defaults apply; ask the user for the alias if one is not supplied.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
