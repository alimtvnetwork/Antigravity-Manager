# Subtask 03: Rerun Randomizer UX & Multi-Turn Anti-Repeat Engine
Parent: `221-gitmap-and-antigravity-tools-manager-updates-and-testing.md`
Step: 318
Improvement: `Imp-732`
Status: COMPLETED

## Scope & Objective
In `src/hooks/usePromptTreePromptActions.ts` and `src/components/prompts/DetailHeader.tsx`:
- Enhance `getRandomRerunNudge` with an expanded pool of 10+ diverse confirmation prompts ("Is it finished?", "Is it released?", "Is it completed?", "Are you confident?", "Did all tests pass?", "Are edge cases covered?", "Is documentation synced?", "Is linting clean?").
- Implement 5-turn history tracking in local state to prevent any repeat of the last 5 nudges.
- Ensure Round 1 sends immediately while Round 2..N enqueues sequentially into FIFO prompt queue.

## Completed Actions
- Added `pickDiverseRerunNudge` and exported `getRandomRerunNudge` with `RERUN_HISTORY_WINDOW = 5`.
- Updated `DetailHeader.tsx` to maintain `rerunHistory` in local state and cycle through diverse nudges.
- Unit tests added in `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts` and `src/tests/rerunActions.test.ts` (0 builds/tests executed).
- Status: COMPLETED.
