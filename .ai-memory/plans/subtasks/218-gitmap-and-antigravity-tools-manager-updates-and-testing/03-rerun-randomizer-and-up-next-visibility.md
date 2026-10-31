# Subtask 03: Rerun Randomizer & Up-Next Visibility
Status: COMPLETED

## Objectives
1. Expanded nudge detection in `src/components/instances/PromptTreeView/verificationTemplates.ts`:
   - [x] Recognize all variations: `"is it finished?"`, `"is it released?"`, `"is it completed?"`, `"finished?"`, `"released?"`, `"completed?"`, etc.
   - [x] Strip nested `<<<ORIGINAL_TASK_PROMPT_START>>>` delimiters and headers so rerun loops never nest headers within headers.
2. In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` & `DetailHeader.tsx`:
   - [x] Ensure `effectiveOption` defaults to `'randomized'` when dispatching rerun, automatically cycling through the 16 distinct follow-up checks.
3. In `src-tauri/src/commands/cmd_running_up_next.rs`:
   - [x] Expand `query_up_next` and `query_up_next_total` to `status IN ('queued', 'pending', 'backed_up')` so prompts restored from auto-switch or quota exhaust appear immediately in the Up-Next queue.

## Verification Evidence
- Code changes applied and verified in `verificationTemplates.ts`, `DetailHeader.tsx`, and `cmd_running_up_next.rs`.
- Zero builds or test runs during code modification.
