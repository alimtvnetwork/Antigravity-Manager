---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "009"
title: Focus scrolls to the selected instance's account
domain: frontend
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t04--focus
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/54-focus-button-dual-scroll-mechanism-rca.md
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/07-focus-button-target-account.md
target_files:
  - src/pages/Accounts.tsx — focus effect (~L113-157), `handleFocusActiveAccount` (~L629-664), `currentAccountId` props (~L1131, ~L1162)
  - src/lib/resolve-focus-target.ts (new, pure)
status: done
---

# 009 — Focus scrolls to the selected instance's account

## 1. Context
Decision (ambiguity 07): target the selected instance's bound account, fall back to the global current account. Two mechanisms scroll (handler and effect); the target is the global current account, not the selected instance's account; filters and pagination can hide the row.

## 2. Target files and symbols
- src/pages/Accounts.tsx — focus effect (~L113-157), `handleFocusActiveAccount` (~L629-664), `currentAccountId` props (~L1131, ~L1162)
- src/lib/resolve-focus-target.ts (new, pure)

## 3. Steps
0. Fix the existing TypeScript error first: the focus effect in `Accounts.tsx` (~L157) reads `currentPage` before its declaration (TS2448/TS2454, found by `npx tsc --noEmit` on 2026-09-30). Move the declaration above the effect; this is likely part of why Focus fails.
1. Add `resolveFocusTarget(selectedInstanceAccountId, currentAccountId)` returning the bound account id, else the current account id, else null.
2. In `Accounts.tsx`, make the Focus handler only set `focusedAccountId` (and clear filters/search, navigate to the page that holds the row).
3. Keep one effect that scrolls the row into view once it is rendered, pulses a highlight for about 2 seconds, then clears `focusedAccountId`.
4. Remove the second scrolling path and any polling made redundant by it.
5. If no target exists, show a toast instead of silently doing nothing.

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
- [x] Focus scrolls to the row across pages and filters.
- [x] Only one scroll mechanism remains.
- [x] TypeScript error resolved and target resolver pure function unit tested.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
