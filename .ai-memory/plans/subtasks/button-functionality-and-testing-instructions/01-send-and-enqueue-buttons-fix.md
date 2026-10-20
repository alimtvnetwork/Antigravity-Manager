# Subtask 01: Send & Enqueue Buttons Repair

**Subtask ID**: `01-send-and-enqueue-buttons-fix`  
**Parent Plan**: `.ai-memory/plans/button-functionality-and-testing-instructions.md`  
**Status**: QUEUED  

## Intent
Fix Send and Enqueue buttons in `usePromptTreePromptActions.ts` and `DetailHeader.tsx` so that prompts (with and without images) are reliably dispatched to active IDE sessions or enqueued into the prompt queue without failing.

## Files
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/services/instanceService/lifecycle.ts`
