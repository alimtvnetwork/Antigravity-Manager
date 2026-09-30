---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "006"
title: One-liner PowerShell file that imports keys through the AGM CLI
domain: scripts
depends_on: 005-agm-supabase-push-settings.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t08t11--supabase-and-vault-scripts
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - scripts/setup-supabase.ps1 — replace with a thin one-liner wrapper around `agm supabase push-settings`
  - scripts/supabase-setup.ps1 — delete (duplicate)
status: pending
---

# 006 — One-liner PowerShell file that imports keys through the AGM CLI

## 1. Context
Both scripts embed default endpoint keys as parameter defaults and duplicate each other. The user wants the keys imported through the AGM CLI instead. Existing vault scripts (`../repo-secrets/...`) are fine and are not changed.

## 2. Target files and symbols
- scripts/setup-supabase.ps1 — replace with a thin one-liner wrapper around `agm supabase push-settings`
- scripts/supabase-setup.ps1 — delete (duplicate)

## 3. Steps
1. Rewrite `scripts/setup-supabase.ps1` to one line of logic: `agm supabase push-settings --vault $VaultPath`, with `-VaultPath` mandatory (default from `AGM_SUPABASE_VAULT`); exit with the CLI's exit code.
2. Delete `scripts/supabase-setup.ps1`.
3. Put the runnable one-liner in the script header: `pwsh -NoProfile -File ./scripts/setup-supabase.ps1 -VaultPath <vault-json-path>`.
4. Confirm no key literal remains: `git grep -n "sb_publishable" -- scripts`. The previously committed keys must be rotated by their owner (publishable keys still count as credentials); do not print them.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only. The secrets repo is a sibling repository at `../repo-secrets/`; it is read, not edited, by this plan.
- NEVER print, log, echo, or commit a secret, token, key, password, or email. Print key names and counts only.
- No machine alias handling anywhere (resolved ambiguity 02).
- PowerShell 5.1 and 7 compatible; check `$LASTEXITCODE`.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
git grep -n "sb_publishable" -- scripts
pwsh -NoProfile -File ./scripts/setup-supabase.ps1 -VaultPath <vault-json-path>
```

## 7. Done When
- [ ] No key literal remains in tracked scripts.
- [ ] One script remains and calls the AGM CLI.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
