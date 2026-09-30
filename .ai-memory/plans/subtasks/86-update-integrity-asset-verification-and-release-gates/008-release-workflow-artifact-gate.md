---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "008"
title: release.yml publishes only with every expected artifact
domain: ci-cd
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#release-workflow-gate
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../cicd-issues/39-release-published-with-missing-artifacts-rca.md
  ambiguity: none
target_files:
  - .github/workflows/release.yml — publish job (~L312, L397-496 updater.json generation, L539-571)
status: pending
---

# 008 — release.yml publishes only with every expected artifact

## 1. Context
Publish can run while matrix legs failed, and `updater.json` URLs are built from assumed names.

## 2. Target files and symbols
- .github/workflows/release.yml — publish job (~L312, L397-496 updater.json generation, L539-571)

## 3. Steps
1. Define the expected artifact list once (per platform: installer, updater signature) in a workflow-level env or a small script.
2. Add a step before publish that lists downloaded artifacts and fails if any expected item is missing.
3. Generate `updater.json` from the actual uploaded asset names, never from guessed names.
4. Create the release as a draft, publish it only after the gate passes.
5. Do not weaken, skip, or disable any existing CI check.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only; no hardcoded repository owner, read it from one constant.
- Cross-platform: Windows, macOS, Linux behavior must match; no platform-only assumption without a fallback.
- Do not edit CI to hide failures; only add gates.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
python 03-ai-scripts/06-cicd-local-runner.py --help
```

## 7. Done When
- [ ] A run with a missing artifact fails before publish.
- [ ] `updater.json` names match uploaded assets.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
