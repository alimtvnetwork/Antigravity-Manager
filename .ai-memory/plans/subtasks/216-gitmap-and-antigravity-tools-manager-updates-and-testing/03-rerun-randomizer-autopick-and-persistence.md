# Subtask 03: Rerun Randomizer Auto-Pick & Persistence
Sequence: 216
Step: 293
Status: COMPLETED
Module: `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, `usePromptTreePromptActions.ts`, `DetailHeader.tsx`

## Description
Ensure that clicking rerun automatically picks and defaults to the randomized option (`randomized`), provides discrete choices ("Is it finished?", "Is it released?", "Is it done?", "Is it published?"), and draws from 18+ varied completion/release prompts with persistent user selection.

## Implementation Details
1. Verify `RERUN_DEFAULT_MESSAGE_OPTION` is pinned to `'randomized'`.
2. Verify `RERUN_MESSAGE_OPTIONS` includes `is-it-finished`, `is-it-released`, `is-it-done`, and `is-it-published`.
3. Verify `RERUN_RANDOM_MESSAGE_SET` contains 18 diverse check prompts.
4. Verify localStorage persistence via `savePersistedRerunMessageOption` and `loadPersistedRerunMessageOption`.

## Done When
- Rerun button defaults to randomized auto-pick on click, with all message options fully supported and persisted.
