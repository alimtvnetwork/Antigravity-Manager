---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "006"
title: install.sh gets the same pre-check contract
domain: installer
depends_on: 005-install-ps1-precheck-and-ladder.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#installer-pre-check
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/49-update-announced-without-platform-asset-rca.md
  ambiguity: none
target_files:
  - install.sh — `--check-update` (~L1082-1121) and the ladder
status: pending
---

# 006 — install.sh gets the same pre-check contract

## 1. Context
Parity with install.ps1 so macOS and Linux users get the same behavior.

## 2. Target files and symbols
- install.sh — `--check-update` (~L1082-1121) and the ladder

## 3. Steps
1. Implement asset HEAD verification with `curl -fsI` per candidate; skip and record failures.
2. `--check-update` prints the same JSON keys and exit codes as subtask 005 (build JSON without requiring `jq`; use `jq` only if present).
3. Print the same human 'skipped' lines.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only; no hardcoded repository owner, read it from one constant.
- Cross-platform: Windows, macOS, Linux behavior must match; no platform-only assumption without a fallback.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
bash ./install.sh --check-update
```

## 7. Done When
- [ ] Output keys identical to install.ps1.
- [ ] Exit codes identical.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
