---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "006"
title: Hide Priority at 50 and edit on double-click
domain: frontend
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#priority-t06--behavior-and-benefit
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: none (feature request)
  ambiguity: none
target_files:
  - src/components/accounts/AccountTable.tsx (~L630-632 badge; add inline editor)
  - src/components/accounts/PriorityBadge.tsx (new)
  - src/stores/useAccountStore.ts — `updateAccountPriority` (existing, reuse)
status: pending
---

# 006 — Hide Priority at 50 and edit on double-click

## 1. Context
Backend sorts lower priority first inside a tier (`proxy/token_manager.rs`); 50 is neutral. The badge always shows and cannot be edited outside the details dialog.

## 2. Target files and symbols
- src/components/accounts/AccountTable.tsx (~L630-632 badge; add inline editor)
- src/components/accounts/PriorityBadge.tsx (new)
- src/stores/useAccountStore.ts — `updateAccountPriority` (existing, reuse)

## 3. Steps
1. Create `PriorityBadge` that renders nothing when `priority === 50` (define `DEFAULT_ACCOUNT_PRIORITY = 50` once and import it).
2. When different, render a compact badge; double-click switches to a numeric input (1-100); Enter saves through `onUpdatePriority`, Esc or blur cancels; invalid input is rejected with inline hint.
3. To set a priority on a hidden (50) account, expose the edit through the existing row action or the details dialog; do not add a new always-visible control.
4. Use the badge in `AccountTable.tsx`; keep `AccountDetailsDialog.tsx` editing as is.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.
- Do not change the backend default or sort order.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
npm run build
```

## 7. Done When
- [ ] All-50 list shows no priority badges.
- [ ] A non-50 account shows the badge and double-click edits and persists.
- [ ] `npm run build` passes.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
