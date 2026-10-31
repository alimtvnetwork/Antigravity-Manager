# Subtask 2: Action Buttons Consolidation, Compact Capsules, and In-Tree Breakdown

- **Subtask ID**: 242-02
- **File**: `02-spec/21-app/242-prompt-tree-and-ui-bug-fixes/02-subtask-action-buttons-dropdowns-and-compact-ui.md`
- **Category**: Frontend UI / Prompt Tree Action Controls
- **Status**: Ready for Execution

---

## 1. Objectives & Scope
1. Replace bulky `rounded-full` pills in `DetailHeader.tsx` with cohesive, compact segmented capsules (`h-7`, `rounded-md`, subtle dividers, dark glass styling).
2. Group controls into 3 compact capsules:
   - Capsule 1: `[ Copy | ▾ ]` (format export menu: clean text, markdown, JSON, rich media).
   - Capsule 2: `[ Send ] [ Enqueue ] [ Rerun ×N ] [ ▾ ]` (direct action triggers + unified dispatch popover).
   - Capsule 3: `[ ✨ Next Breakdown {count} | ▾ ]` (1-click trigger + interactive dropdown).
3. Implement rich Next Breakdown dropdown popover in `DetailHeader.tsx`:
   - Category selector: `Code`, `UI`, `System`, `Ideas`.
   - Step count selector: `10`, `20`, `50`, `100`.
   - Automated actions: `Detail Out Tasks`, `Auto-Enqueue Next Batch`, `Plan Improvements`, `Check Backlog`.
   - In-Tree Navigation: "View in Prompt Tree" toggle button.
4. Extend `previewTab` type in `types.ts` and `usePromptTreeState.ts` to include `'improvements'`.
5. Add 3rd tab in `DetailTabs.tsx`: `[ Input ] [ Output ] [ ✨ Breakdowns (100) ]`.
6. Implement `PromptTreeBreakdownView.tsx` and render inside `PromptTreeViewModal.tsx` when `previewTab === 'improvements'`.

---

## 2. Target Files
- `src/components/instances/PromptTreeView/types.ts`
- `src/components/instances/PromptTreeView/usePromptTreeState.ts`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/DetailTabs.tsx`
- `src/components/instances/PromptTreeView/PromptTreeBreakdownView.tsx` (new)
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- `src/pages/instances/InstancesHeader.tsx`
