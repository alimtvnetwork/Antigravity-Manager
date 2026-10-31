# Issue 65: Anti-gravity Rerun Target ID, Sequential Enqueue, and Tree View Compaction

- **Issue ID:** #65
- **Sequence ID:** 231
- **Status:** CLOSED (RESOLVED)
- **Component:** Tree View, Prompt Queue, Rerun Handler, GitMap CLI
- **Resolved Version:** v4.272.0

---

## 1. Problem Description

1. **Anti-gravity rerun failed to enqueue onto target ID:** Reruns dropped the authoritative conversation UUID, falling back to `'current'` and failing to enqueue properly.
2. **Sequential enqueue not working after first time:** Enqueuing a prompt once worked, but enqueuing a second time (or running multi-round reruns) resulted in the subsequent items being dropped from the database or merged into historical transcript turns.
3. **Unwanted refresh bar on enqueue:** Clicking "Enqueue" displayed a top 2px blue indeterminate progress bar and spinning refresh icon.
4. **Right-hand tree view filter clipped:** Row 2 filters (`[ 🚫 Hide Auto ]`, `[ 📦 Archived (0) ]`) required 451px, causing the right edge to be cut off in compact sidebars.
5. **GitMap Skill compliance break:** Raw ambient `python -c` scripts were executed in PowerShell instead of native compiled Go commands (`gitmap agent task init`).

---

## 2. Solution Summary

1. **Exempt Queued Prompts from In-Flight Dedup (`prompts_crud.rs`):**
   Added `&& prompt.status != "queued"` to `save_or_requeue_prompt` so new queue entries are never skipped when an identical prompt is dispatched or in-flight.
2. **Decouple Queued Turn Ingestion (`tree_node_conv.rs`):**
   Gated `existing_turn` lookup to active running prompts only (`if is_ap_run && !is_queued`). Queued prompts always append a new `AgmPromptTurnNode` with `status: "QUEUED"` and `is_queued: true`.
3. **Synchronize Frontend Dispatch Props (`PromptTreeViewModal.tsx`, `PromptInstructionView.tsx`):**
   Wired `conversationTargetMode`, `newConversationTitle`, `confirmationSuffix`, `onRerun`, `onEnqueue`, and `onResend`. Added 0ms optimistic turn injection into `executeDirectEnqueue`.
4. **Suppress Enqueue Refresh Bar (`usePromptTreePromptActions.ts`, `DetailHeader.tsx`):**
   Replaced `loadTree(true, true)` with `loadTree(false, true)` for silent background SWR revalidation. Removed artificial 60ms delay in multi-round rerun dispatch.
5. **Tree Filter Row 2 Compaction (`TreePanel.tsx`):**
   Compacted status capsule to icon-first labels (`Run`, `Queue`, `Pin`, Lucide `Image`) and scope capsule to icon buttons (`EyeOff`/`Eye`, Lucide `Archive` with count badge). Total row width: 194px ($\le 195\text{ px}$), with $+70\text{ px}$ safety margin at $280\text{ px}$.
6. **Author Canonical Tree View Design System Spec:**
   Created `02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md` detailing token architecture, hover-over physics, and 4-tier origin badges.
7. **GitMap Agent Task Orchestration:**
   Replaced ambient Python scripts with native `gitmap agent task init --name <slug> --budget 400` (<16ms).

---

## 3. Verification

- Verified via `scripts/test-anti-gravity-rerun-and-ui-fixes-e2e.ps1`.
- Total checks: 28 | Passed: 28 | Failed: 0 (100% Green).
