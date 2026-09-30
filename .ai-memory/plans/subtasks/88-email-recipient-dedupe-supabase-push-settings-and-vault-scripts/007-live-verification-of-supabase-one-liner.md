---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "007"
title: Run and verify the one-liner on this machine
domain: e2e
depends_on: 006-one-liner-powershell-file-uses-agm-cli.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#acceptance-criteria
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - scripts/setup-supabase.ps1 (run only)
status: pending
---

# 007 — Run and verify the one-liner on this machine

## 1. Context
The user wants it verified to work, end to end, not just written.

## 2. Target files and symbols
- scripts/setup-supabase.ps1 (run only)

## 3. Steps
1. Run the one-liner against the vault Supabase JSON (explicit path). Record exit code and step results (endpoint ids and counts only).
2. Run `agm supabase status` and `agm supabase test`; both must exit 0.
3. Run the one-liner a second time to prove idempotence (no duplicate endpoint).
4. Run once without `-VaultPath` to prove a clear message and non-zero exit.
5. Write the observed results (no secrets) into the plan status.

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
pwsh -NoProfile -File ./scripts/setup-supabase.ps1 -VaultPath <vault-json-path>
agm supabase status
agm supabase test
```

## 7. Done When
- [ ] Exit code 0 on the live run.
- [ ] Second run is idempotent.
- [ ] Missing-path run exits non-zero with a clear message.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
