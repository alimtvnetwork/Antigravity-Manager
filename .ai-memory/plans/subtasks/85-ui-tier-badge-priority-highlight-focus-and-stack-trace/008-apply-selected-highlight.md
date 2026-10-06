---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "008"
title: Apply the selected-state style everywhere
domain: frontend-design
depends_on: 007-selected-state-shared-style.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#selected-state-design-options-t05-user-picks-one-subtask-008-applies-it
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: none (design request)
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/05-selected-state-highlight-option-choice.md
target_files:
  - src/components/navbar/InstanceSelector.tsx (~L632-634 selected row)
  - src/pages/Instances.tsx (active card)
  - src/components/accounts/AccountTable.tsx (row highlight classes ~L270-285)
  - src/components/accounts/AccountRow.tsx
  - src/components/accounts/AccountCard.tsx
status: done
---

# 008 — Apply the selected-state style everywhere

## 1. Context
Applies the tokens from step 007 to the popover selected profile, the Instances active card, and the accounts `CURRENT` row.

## 2. Target files and symbols
- src/components/navbar/InstanceSelector.tsx (~L632-634 selected row)
- src/pages/Instances.tsx (active card)
- src/components/accounts/AccountTable.tsx (row highlight classes ~L270-285)
- src/components/accounts/AccountRow.tsx
- src/components/accounts/AccountCard.tsx

## 3. Steps
1. Replace the white selected row in `InstanceSelector.tsx` with `SELECTED_ROW_CLASSES`.
2. Replace the weak row highlight in the accounts table, row, and card views and the Instances active card.
3. Check both themes; keep the `ACTIVE` and `DEFAULT` badges readable.
4. Take a screenshot of each surface with fake data only and attach the description to the plan status (no real emails).

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
- [ ] Selected instance and current account are unmistakable in both themes.
- [ ] No inline white-selected styling remains.
- [ ] `npm run build` passes.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
