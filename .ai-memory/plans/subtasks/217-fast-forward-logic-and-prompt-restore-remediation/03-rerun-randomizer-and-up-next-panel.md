# Subtask 03: Rerun Randomizer & Up-Next Panel Visibility
Status: COMPLETED

## Objectives
1. In `src/components/instances/PromptTreeView/verificationTemplates.ts:848-882`:
   - [x] Expand `isNudge` check to recognize `"is it finished?"`, `"is it released?"`, `"is it completed?"`, `"finished?"`, `"released?"`, `"completed?"`, etc.
   - [x] Strip nested rerun templates so reruns never nest headers within headers.
2. In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` and `DetailHeader.tsx`:
   - [x] Ensure `effectiveOption` defaults to `'randomized'` when dispatching rerun, automatically picking and executing lively random completion checks.
3. In `src-tauri/src/commands/cmd_running_up_next.rs`:
   - [x] Expand `query_up_next` and `query_up_next_total` to `status IN ('queued', 'pending', 'backed_up')`.

## Verification Criteria
- [x] Zero builds and zero tests during code modification.
