---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "005"
title: install.ps1 verifies assets before selecting a version
domain: installer
depends_on: 002-release-asset-resolver.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#installer-pre-check
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/49-update-announced-without-platform-asset-rca.md
  ambiguity: none
target_files:
  - install.ps1 — `-CheckUpdate` (~L786-885) and the version ladder (~L1207+)
status: pending
---

# 005 — install.ps1 verifies assets before selecting a version

## 1. Context
The ladder learns a release has no binary only after a failed download, and `-CheckUpdate` reports version only.

## 2. Target files and symbols
- install.ps1 — `-CheckUpdate` (~L786-885) and the version ladder (~L1207+)

## 3. Steps
1. Before downloading, HEAD the expected asset for each candidate (newest first); skip and record candidates that return non-2xx.
2. `-CheckUpdate` prints one JSON object with `current`, `latest_tag`, `resolved_version`, `asset_url`, `asset_exists`, `candidates_tried`, `skipped_versions`; exit 0 when a verified candidate exists, exit 1 otherwise.
3. Print a human line for every skipped version ('vX skipped: no installer file') so the user sees why.
4. Keep the fork/upstream owner logic from spec 22-app-issues/20 intact; read the owner from the same variable.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only; no hardcoded repository owner, read it from one constant.
- Cross-platform: Windows, macOS, Linux behavior must match; no platform-only assumption without a fallback.
- Script must not swallow errors; check exit codes and HTTP status.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
pwsh -NoProfile -File ./install.ps1 -CheckUpdate
```

## 7. Done When
- [ ] JSON contract printed with all fields.
- [ ] A version without an asset is skipped before any download.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
