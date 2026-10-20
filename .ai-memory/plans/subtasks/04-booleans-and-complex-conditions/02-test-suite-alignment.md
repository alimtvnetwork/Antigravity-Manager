# Subtask 02: Test Suite Alignment & Unit Verification

**Slug**: `04-booleans-and-complex-conditions`  
**Parent Plan**: `.ai-memory/plans/04-booleans-and-complex-conditions.md`  
**Status**: DONE  
**Owner**: Lead Orchestrator  

---

## 1. Overview & Scope
- Aligned `src/pages/instances/__tests__/instanceRunningPrompts.test.ts` with discrete guard clauses and `isConversationGenuinelyRunning`.
- Verified sorting logic in `sortInstanceProjects` uses `isARunning !== isBRunning ? (isARunning ? -1 : 1) : ...`.
- Verified `checkHasActiveTask` uses discrete early return guards.
- Executed `npx tsx src/pages/instances/__tests__/instanceRunningPrompts.test.ts`.
- Result: 32 passed, 0 failed.
