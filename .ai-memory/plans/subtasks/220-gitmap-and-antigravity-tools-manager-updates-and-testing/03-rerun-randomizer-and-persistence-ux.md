# Subtask 03: Rerun Randomizer UX & Persistent Execution Engine
Status: COMPLETED
Sequence: 220
Step: 313
Improvement: `Imp-727`
Owner: Worker 02

## Objectives
- Implement automated randomizer variation selection when clicking Rerun in Prompt Tree View.
- Provide automatic selection and preview of randomizer nudge templates (e.g. style variations, prompt re-framing, dynamic prefixes).
- Expand rerun execution flow to support sequential queueing with multiple variations without requiring repeated manual clicks.
- Enhance `PromptTreeViewModal.tsx` and `usePromptTreePromptActions.ts` with transparent segmented styling and instant execution feedback.

## Disjoint File Boundaries
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`

## Invariants
- Zero builds / zero tests during implementation.
- Transparent borderless dropdown styling inside segmented capsules.
