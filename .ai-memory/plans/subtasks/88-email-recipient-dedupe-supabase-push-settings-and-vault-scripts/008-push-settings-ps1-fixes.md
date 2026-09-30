---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "008"
title: Fix `push-settings.ps1` remote transfer and verification
domain: scripts
depends_on: 007-connect-supabase-ps1-hardening.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t08t11--supabase-and-vault-scripts
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - ../repo-secrets/02-antigravity-and-event-manager/push-settings.ps1
  - ../repo-secrets/02-antigravity-and-event-manager/push-settings.sh
status: pending
---

# 008 — Fix `push-settings.ps1` remote transfer and verification

## 1. Context
The gitmap branch builds an archive but never transfers or extracts it, verification steps swallow errors, and the local target `~/.antigravity_tools` must match the data directory the app really reads.

## 2. Target files and symbols
- ../repo-secrets/02-antigravity-and-event-manager/push-settings.ps1
- ../repo-secrets/02-antigravity-and-event-manager/push-settings.sh

## 3. Steps
1. Confirm the app's data directory per OS from `modules::account::get_data_dir` (or equivalent) and make the script resolve the same path; add `-TargetDataDir` and `AGM_DATA_DIR` override handling that matches the app.
2. In the gitmap branch, upload the archive, extract it on the target, then delete the temp archive in `finally`; fail if any step returns non-zero.
3. Replace `try { agm status } catch { }` with checked calls whose failure sets a non-zero exit code and is summarized at the end.
4. Add a final Supabase step calling `agm supabase push-settings` (skippable with `-SkipSupabase`).
5. Mirror the fixes in `push-settings.sh`.

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
pwsh -NoProfile -File ../repo-secrets/02-antigravity-and-event-manager/push-settings.ps1 -VerifyAfterPush
```

## 7. Done When
- [ ] Remote push transfers and extracts files or fails.
- [ ] Verification failures exit non-zero.

## 8. Ambiguities and interim defaults
- Ambiguity 02 defaults apply.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
