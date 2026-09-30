---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "009"
title: Post-publish job re-fetches every URL in updater.json
domain: ci-cd
depends_on: 008-release-workflow-artifact-gate.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#release-workflow-gate
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../cicd-issues/39-release-published-with-missing-artifacts-rca.md
  ambiguity: none
target_files:
  - .github/workflows/release.yml — new `verify-release-assets` job
  - 03-ai-scripts/verify-release-assets.py (new; also usable locally)
status: pending
---

# 009 — Post-publish job re-fetches every URL in updater.json

## 1. Context
Even with a gate, a wrong URL in `updater.json` would announce an update that cannot be installed.

## 2. Target files and symbols
- .github/workflows/release.yml — new `verify-release-assets` job
- 03-ai-scripts/verify-release-assets.py (new; also usable locally)

## 3. Steps
1. Write `verify-release-assets.py` (cross-platform, standard library only): download `updater.json` for a tag, HEAD every URL, print a short table, exit non-zero on any non-2xx.
2. Add a `verify-release-assets` job that runs after publish and fails the workflow on error; on failure it marks the release as prerelease so the updater stops offering it.
3. Document the script in `03-ai-scripts/readme.md` (one line).

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only; no hardcoded repository owner, read it from one constant.
- Cross-platform: Windows, macOS, Linux behavior must match; no platform-only assumption without a fallback.
- Quiet output on success (one summary line); details only on failure.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
python 03-ai-scripts/verify-release-assets.py --help
```

## 7. Done When
- [ ] Script detects a doctored URL locally.
- [ ] Job exists and depends on publish.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
