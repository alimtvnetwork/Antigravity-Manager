# Subtask 02: Immediate Send vs Enqueue & Image Forwarding

**Parent Plan**: `.ai-memory/plans/ui-and-image-copy-functionality-fixes.md`  
**Target Files**:  
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`  
- `src/components/instances/PromptTreeView/DetailHeader.tsx`  
- `src-tauri/src/modules/repo_db/prompts_crud.rs`  

---

## 1. Objectives
1. Decouple "Send" from "Enqueue" in `usePromptTreePromptActions.ts` and `DetailHeader.tsx`.
   - "Send" (`handleResendPrompt` / `handleSendPromptNow`) must invoke `sendPromptNow` immediately targeting the running instance session without enqueuing into `active_prompts`.
   - "Enqueue" (`handleEnqueuePrompt`) explicitly appends the prompt to `active_prompts` with status `queued`.
2. Ensure image forwarding in Send, Run, and Rerun:
   - When rerunning or sending a prompt that contains attached images (`media_paths`), extract and preserve all image file paths or markdown references (`![image](<path>)`) so that the dispatched prompt payload includes the images.
   - Prevent image references from being stripped during prompt sanitization or dispatch formatting.
