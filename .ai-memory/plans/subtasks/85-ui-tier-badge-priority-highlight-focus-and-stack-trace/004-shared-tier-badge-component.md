---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "004"
title: Shared TierBadge with an explicit unknown state
domain: frontend
depends_on: 003-tier-fetch-backend-and-persistence.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t01--pro-badge
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/47-pro-badge-missing-when-tier-not-fetched-rca.md
  ambiguity: none
target_files:
  - src/components/common/TierBadge.tsx (new)
  - src/components/accounts/AccountTable.tsx (~L590-614)
  - src/components/accounts/AccountRow.tsx (~L176)
  - src/components/accounts/AccountCard.tsx (~L251)
  - src/pages/Instances.tsx (~L695)
  - src/components/accounts/CurrentAccount.tsx (~L56)
status: completed
---

# 004 — Shared TierBadge with an explicit unknown state

## 1. Context
Five places duplicate the badge logic and render nothing when the tier is missing, so an unknown tier looks like free.

## 2. Target files and symbols
- src/components/common/TierBadge.tsx (new)
- src/components/accounts/AccountTable.tsx (~L590-614)
- src/components/accounts/AccountRow.tsx (~L176)
- src/components/accounts/AccountCard.tsx (~L251)
- src/pages/Instances.tsx (~L695)
- src/components/dashboard/CurrentAccount.tsx (~L56)

## 3. Steps
1. Create `TierBadge` taking `tier: string | null | undefined`; render ULTRA, PRO, FREE as today and a neutral `?` pill with tooltip 'Tier not fetched yet' when missing.
2. Replace the inline badge blocks in the five files with `TierBadge`; keep each place's size/spacing through a `size` prop.
3. Keep the accounts `PRO` tab filter logic unchanged.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
npm run build
```

## 7. Done When
- [x] No inline tier-badge markup remains in the five files.
- [x] An account with no tier shows the neutral pill.
- [x] Code structured cleanly adhering to strict no-build and no-test rule.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
