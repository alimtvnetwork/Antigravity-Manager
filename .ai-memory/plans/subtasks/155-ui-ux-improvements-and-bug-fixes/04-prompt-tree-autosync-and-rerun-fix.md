# Subtask 04: Prompt Tree Auto-Sync & Rerun Logic Fixes

## Objective
Enable responsive auto-syncing during active running prompts, fix the Rerun action, and strictly preserve the functioning Send & Enqueue buttons.

## Target Files
- `src/components/instances/PromptTreeView/usePromptTreeData.ts`
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/verificationTemplates.ts`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`

## Technical Details
1. **Auto-Sync**: Dynamically poll at 2–3s intervals when `is_running: true` or active task is present, passing `force: true` to bypass the 120s SQLite cache TTL.
2. **Rerun Logic**: Fix `buildRerunPrompt` to retain the selected nudge message and resolve previous conversation text. Avoid self-cancelling preemption loop in multi-count reruns.
3. **Send & Enqueue Invariant**: Keep the segmented pill layout `[ Send (Preempt) | Enqueue (FIFO) | v ]`, `N` hotkey, and dual-MIME rich image priming intact.
