# Subtask 218-03: Rerun Randomizer UX & Persistent Execution Engine
Task ID: `Imp-717`
Sequence: 218
Step: 303
Status: IN_PROGRESS

## Scope
- Set default `rerunMessageOption` to `'randomized'` across DetailHeader, context menus, and modals.
- Expand actionable variation set to 25+ distinct nudges in `RERUN_RANDOM_MESSAGE_SET` and add extended options to `RERUN_MESSAGE_OPTIONS`.
- Ensure clicking Rerun automatically picks a random nudge from the bunch ("Is it finished?", "Is it released?", "Is it completed?", "Please verify and finish the remaining items.", "Did all tests pass and is everything green?", etc.).
- Fix original prompt extraction in `DetailHeader.tsx`, `PromptTreeViewModal.tsx`, and `verificationTemplates.ts` so original user prompt is losslessly extracted without recursive header nesting or accidental clearing.
- Persist user rerun preference in `localStorage` under `agm:prompt-tree:rerun-option` and `agm:prompt-tree:rerun-count`.
- Update unit tests in `rerunActions.test.ts`.

## Files Touched
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- `src/components/instances/PromptTreeView/verificationTemplates.ts`
- `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`
