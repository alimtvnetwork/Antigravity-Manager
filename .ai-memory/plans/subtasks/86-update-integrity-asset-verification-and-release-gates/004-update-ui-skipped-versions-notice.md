---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "004"
title: Tell the user which versions were skipped and install the resolved one
domain: frontend
depends_on: 002-release-asset-resolver.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#update-check-result-rust-updateinfo-and-typescript-mirror
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/49-update-announced-without-platform-asset-rca.md
  ambiguity: none
target_files:
  - src/components/UpdateNotification.tsx
  - src/stores/use-update-store.ts
  - src/pages/Settings.tsx — update check call (~L434)
status: pending
---

# 004 — Tell the user which versions were skipped and install the resolved one

## 1. Context
The user wants to be told that a release was skipped because its files are missing, and be offered another release.

## 2. Target files and symbols
- src/components/UpdateNotification.tsx
- src/stores/use-update-store.ts
- src/pages/Settings.tsx — update check call (~L434)

## 3. Steps
1. When `skipped_versions` is non-empty, show a compact notice: 'vX skipped: no installer for this platform' with a details toggle listing each reason.
2. The Install action targets the resolved version shown in the banner; if there is none, show 'No installable update' and hide Install.
3. Offer 'Choose another release' that lists `candidates` (resolved plus older verified ones) and lets the user install a specific one.
4. Keep the existing badge/pill conventions; no new settings.

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
npm run build
```

## 7. Done When
- [ ] Skipped versions are visible with reasons.
- [ ] Install never targets an unverified version.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
