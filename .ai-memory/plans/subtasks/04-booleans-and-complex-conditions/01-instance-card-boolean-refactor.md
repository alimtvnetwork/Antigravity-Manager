# Subtask 01: Instance Card Boolean Logic Refactoring & Guard Clause Extraction

**Slug**: `04-booleans-and-complex-conditions`  
**Parent Plan**: `.ai-memory/plans/04-booleans-and-complex-conditions.md`  
**Status**: DONE  
**Owner**: Lead Orchestrator  

---

## 1. Overview & Scope
- Eliminated compound negative guard `if (!inst.is_running || !proj.is_running) return false;` in `src/pages/instances/InstanceCardProjects.tsx`.
- Replaced with discrete affirmative guards:
  ```typescript
  const isInstanceRunning = Boolean(inst.is_running);
  if (!isInstanceRunning) {
      return false;
  }
  const isProjectRunning = Boolean(proj.is_running);
  if (!isProjectRunning) {
      return false;
  }
  ```
- Extracted `isConversationGenuinelyRunning` in `src/pages/instances/instancePageUtils.ts` and reused it across `InstanceCardProjects.tsx`, `instanceCardData.ts`, and `useInstancePage.ts`.
- Decomposed sorting mixed polarity in `InstanceCardProjects.tsx`:
  ```typescript
  if (isARunning !== isBRunning) {
      return isARunning ? -1 : 1;
  }
  ```
- Decomposed complex `SwitchAccountDialog.tsx` Gemini/Claude/GPT checks and Escape key handling.
