---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "007"
title: Render the three selected-state options for user review
domain: frontend-design
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#selected-state-design-options-t05-user-picks-one-subtask-008-applies-it
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: none (design request)
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/05-selected-state-highlight-option-choice.md
target_files:
  - src/components/common/SelectedStateOptions.tsx (new, dev-only preview, not routed in production)
  - 02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md — section 'Selected-state design options' (read; edit only if tokens change)
status: pending
---

# 007 — Render the three selected-state options for user review

## 1. Context
The user wants two or three bright, high-contrast options with a yellow line and dark colors, then chooses. No option is applied in this step.

## 2. Target files and symbols
- src/components/common/SelectedStateOptions.tsx (new, dev-only preview, not routed in production)
- 02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md — section 'Selected-state design options' (read; edit only if tokens change)

## 3. Steps
1. Build a preview component showing one sample instance row and one sample account row in options A, B, and C, light and dark themes side by side (no real emails; use 'user@example.test').
2. Compute and display contrast ratios in the preview for selected text versus background; all must be at least 4.5:1.
3. Expose it only under the existing dev/debug console area (no new route or setting).
4. Stop and report to the user with a screenshot request; do not continue to subtask 008 until ambiguity 05 is resolved.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.
- No change to real lists in this step.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
npm run build
```

## 7. Done When
- [ ] Preview shows A, B, C in both themes.
- [ ] Contrast figures shown and >= 4.5:1.
- [ ] User has been asked to choose (ambiguity 05 open).

## 8. Ambiguities and interim defaults
- Ambiguity 05: no default; subtask 008 stays blocked.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
