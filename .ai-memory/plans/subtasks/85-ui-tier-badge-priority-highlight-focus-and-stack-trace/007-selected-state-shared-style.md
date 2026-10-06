---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "007"
title: Shared selected-state style (Option A: dark slate with amber rail)
domain: frontend-design
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#selected-state-design-options-t05-user-picks-one-subtask-008-applies-it
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: none (design request)
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/05-selected-state-highlight-option-choice.md
target_files:
  - src/components/common/selectedState.ts (new; exports the class sets for row and card selected states, light and dark)
status: done
---

# 007 — Shared selected-state style (Option A: dark slate with amber rail)

## 1. Context
Decision (ambiguity 05): Option A. Dark slate row (`bg-slate-900`), 6 px amber rail on the left (`amber-400`), white text, amber `ACTIVE` pill. It matches the request for a yellow line with dark colors. Light theme uses `bg-slate-100` with the same amber rail.

## 2. Target files and symbols
- src/components/common/selectedState.ts (new; exports the class sets for row and card selected states, light and dark)

## 3. Steps
1. Create `selectedState.ts` exporting `SELECTED_ROW_CLASSES`, `SELECTED_CARD_CLASSES`, and `ACTIVE_PILL_CLASSES` (light and dark variants in one place).
2. Document the contrast ratio of each text/background pair in a comment; all pairs must be at least 4.5:1 (compute with a small script, record the numbers).
3. No consumer changes in this step.

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
- [x] One module holds all selected-state tokens.
- [x] Contrast pairs recorded and >= 4.5:1.
- [x] Tokens ready for consumer usage.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
