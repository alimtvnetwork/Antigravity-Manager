# Component Spec: 07-verify-running-prompts-instance-section

## 1. Overview
This specification details the frontend component hierarchy and UI behavior for displaying running prompts within instance cards and the prompt tree modal.

## 2. Component Structure & Interactions

### 2.1 `InstanceCardProjects.tsx`
- **Location**: Mounted within `InstanceCard.tsx` in the "Active / Recent Projects" section.
- **Data Dependency**: Receives `projectTreeNodes` from `useInstancePage.ts`, filtered by `isNodeOwnedByInstance(node, inst.config)`.
- **Project Hoisting**: Projects with `Boolean(inst.is_running) && Boolean(proj.is_running)` are sorted to the top.
- **Visual Presentation**:
  - Displays project name, total turn count, and if running, a green pill with a pulsing dot:
    `<span className="px-1 py-0.2 rounded-[4px] text-[9px] font-bold bg-[#1af18d]/15 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 flex items-center gap-0.5 shadow-2xs">`
  - Clicking any project opens `PromptTreeViewModal` with `projectId: proj.project_id`.

### 2.2 `InstanceCard.tsx`
- **Header Prompt Pill**: When `hasActiveTask` is true (live instance process with genuine running conversation), renders a pulsing green `Prompt` pill in the header title bar.
- **Action Capsule**: Slot 1 (`Layers` icon) tints cyan with a pulsing notification badge.

### 2.3 `PromptTreeViewModal.tsx` & `TreePanel.tsx`
- **Filter Capsule**: Provides interactive pills: `Running`, `Queued`, `Pinned`.
- **"Running" Pill**: Toggles filtering to show only projects and conversations that are actively executing (`Boolean(c.is_running)`).
- **Auto-Selection**: On modal open, `performAutoSelection` in `usePromptTreeData.ts` automatically expands the project and selects the newest active running conversation.

### 2.4 `ProjectNode.tsx` & `ConversationNode.tsx`
- **ProjectNode**: Displays `{runningCount} RUNNING` pill with pinging radar indicator (`relative flex h-2 w-2`).
- **ConversationNode**: Displays `RUNNING` badge with pulsing dot (`bg-[#1af18d] animate-pulse`) alongside 4-tier prompt badges (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, etc.).
- **Ghost Filtration**: Ghost conversations (`Untitled`, 0 words, empty preview) are hidden from the tree.

## 3. Automated Test Verification
Verified by standalone test runner:
- Command: `node scripts/run-frontend-tests.mjs instanceRunningPrompts`
- Target: `src/pages/instances/__tests__/instanceRunningPrompts.test.ts`
- Result: 20 passing assertions across 4 test groups.
