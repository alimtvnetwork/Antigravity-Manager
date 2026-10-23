# Subtask 03: Rerun Randomizer Auto-Pick & Persistence
Sequence: 215
Step: 288
Status: COMPLETED
Module: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `usePromptTreePromptActions.ts`, `ConversationContextMenu.tsx`, `PromptTreeViewModal.tsx`

## Description
1. Change rerun dialog default from `is-it-done` to `randomized`.
2. Enlarge `RERUN_RANDOM_MESSAGE_SET` to at least 12 completion-check variations (16 implemented).
3. Persist user selection and count in localStorage with dual legacy key fallback.
4. Route context-menu rerun and now-running-up-next strip rerun through the same persisted randomized option.
5. Embed direct `Rerun ×{N}` button directly into the primary dispatch segmented pill capsule (`[ Send | Enqueue | Rerun | ▾ ]`).

## Verification & Outcomes
- Added direct `Rerun ×{rerunCount}` action button into Capsule B on the toolbar in `DetailHeader.tsx`.
- Updated `handleStripRerun` in `PromptTreeViewModal.tsx` from hardcoded `'is-it-done'` to `loadPersistedRerunMessageOption()`.
- Enriched `RERUN_RANDOM_MESSAGE_SET` in `usePromptTreePromptActions.ts` with 16 distinct completion, release, and verification checks.
- Enabled auto-picking and persistence for rerun message options and counts.
- Created companion verification test suite in `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`.
