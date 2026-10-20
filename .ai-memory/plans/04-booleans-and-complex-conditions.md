# Master Plan: Boolean Logic Modernization & Minor Bump Release

**Slug**: `04-booleans-and-complex-conditions`  
**Status**: COMPLETED  
**Lead Orchestrator**: Antigravity Lead Agent  
**Target Release**: `v4.244.0`  

---

## 1. User Request (Verbatim)

```text
Try to have better versions of the if  else please read and apply
@[01-prompts/15-cg-execute/04-booleans-and-complex-conditions.md] 

and release with a minor bump please

if (!inst.is_running || !proj.is_running) return false;
```

---

## 2. Subtask Breakdown

| Subtask ID | Focus Area | Owned Files | Status |
|---|---|---|---|
| **Task-01** | Boolean Refactoring in `InstanceCardProjects.tsx` & `instanceCardData.ts` | `src/pages/instances/InstanceCardProjects.tsx`<br>`src/pages/instances/instanceCardData.ts`<br>`src/pages/instances/instancePageUtils.ts`<br>`src/pages/instances/useInstancePage.ts` | COMPLETED |
| **Task-02** | Test Suite Alignment in `instanceRunningPrompts.test.ts` | `src/pages/instances/__tests__/instanceRunningPrompts.test.ts` | COMPLETED |
| **Task-03** | Minor Version Bump & Release Ceremony | Version manifests, `README.md`, `README_EN.md`, changelogs | COMPLETED |
