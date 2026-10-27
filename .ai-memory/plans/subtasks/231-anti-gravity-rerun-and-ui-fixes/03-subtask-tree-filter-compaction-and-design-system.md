# Subtask 03: Tree View Filter Row Compaction & Design System Guideline

## 1. Metadata
- **Subtask ID**: `subtask-231-03`
- **Parent Task**: `parent-231-anti-gravity-rerun-and-ui-fixes`
- **Code**: `SUB-03`
- **Slug**: `tree-filter-compaction-design-system`
- **Status**: `PENDING`
- **Assigned Worker**: `Worker 03`

---

## 2. Objective
1. Repair clipping on the right-hand filter controls in `src/components/instances/PromptTreeView/TreePanel.tsx` (lines 465-559):
   - Compact left status capsule to icon-first with concise text (`Run`, `Queue`, `Pin`, Lucide `Image` icon replacing raw emoji).
   - Compact right capsule to sleek icon-only buttons with tooltips: `EyeOff`/`Eye` toggle for Automated prompts, and direct `Archive` button toggle for archived prompts (eliminating the bulky inline `<select>`).
   - Reduce combined width from 451px down to ~190px, completely eliminating the 123px overflow deficit at default 340px width and at minimum 280px width.
2. Author the canonical Tree View Design System Specification in:
   - `02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md`
   - Reference in `02-spec/24-app-ui-design-system/01-index.md`
   - Covering: 4-tier origin classification, compact segmented pill capsules, responsive width constraints, hover-over physics, `.btn-tactile` micro-interactions, and interactive truncation callouts.

---

## 3. Files to Touch
- `src/components/instances/PromptTreeView/TreePanel.tsx`
- `02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md`
- `02-spec/24-app-ui-design-system/01-index.md`

---

## 4. Acceptance Criteria
- [ ] Tree view filter row fits seamlessly within 280px without horizontal overflow, truncation, or clipping.
- [ ] All buttons have tooltips and tactile hover/active states.
- [ ] Design system spec in coding guidelines provides an authoritative, reproducible blueprint for tree views.
