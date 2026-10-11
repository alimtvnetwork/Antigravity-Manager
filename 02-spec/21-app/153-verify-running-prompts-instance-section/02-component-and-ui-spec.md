# Component & UI Specification: Verification of Running Prompts in Instance Section

- **Spec ID**: `153-verify-running-prompts-instance-section`
- **File**: `02-spec/21-app/153-verify-running-prompts-instance-section/02-component-and-ui-spec.md`
- **Status**: APPROVED & VERIFIED
- **Domain**: Frontend React / Zustand UI Component Architecture

---

## 1. Architectural Overview

This document specifies the exact component architecture, UI states, lifecycle hooks, and interaction models that render and manage active running prompts across Antigravity instance cards and the prompt tree modal.

The running prompt representation follows a 3-layer progressive disclosure model:
1. **Instance Card Header Badge**: High-level indicator alerting the user that an active task or prompt is currently executing within this instance.
2. **Active / Recent Projects Section**: Compact card list hoisting actively running projects to the top, showing real-time execution status pills (`[● RUNNING]`) and cumulative turn counters.
3. **Deep-Linked Prompt Tree Modal**: Full modal view (`PromptTreeViewModal`) that automatically expands the executing project, selects the newest active running conversation, and provides instant filtering via the `Running` pill.

---

## 2. Component Hierarchy & Implementation Details

### 2.1 `src/pages/instances/instancecard.tsx` (`InstanceCard.tsx`)
- **Role**: Top-level card component representing an individual Antigravity IDE instance or profile.
- **Header Prompt Indicator (Lines 123–133)**:
  Mounted directly in the instance card header, adjacent to the instance title and default badge.

```tsx
// Lines 123–133 in src/pages/instances/instancecard.tsx
{hasActiveTask && (
    <button
        type="button"
        onClick={() => openPromptTree(inst.config.id)}
        className="inline-flex items-center gap-1 px-1.5 h-5 rounded-[5px] text-[9px] font-bold bg-[#1af18d]/15 hover:bg-[#1af18d]/25 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 shrink-0 cursor-pointer transition-colors shadow-2xs"
        title="Active prompt/task running - Click to open Prompt Tree"
    >
        <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse shrink-0" />
        <span>Prompt</span>
    </button>
)}
```

- **Visual & Interaction Behavior**:
  - Only visible when `hasActiveTask` is strictly `true` (instance process is alive and owns at least one non-ghost running conversation).
  - Employs emerald glass styling (`bg-[#1af18d]/15`, `border-[#1af18d]/30`, `text-emerald-700 dark:text-[#1af18d]`).
  - Features an animated pulse dot (`animate-pulse`) with subtle glow (`shadow-[0_0_4px_rgba(26,241,141,0.9)]`).
  - Directly opens the Prompt Tree modal filtered to this specific instance via `openPromptTree(inst.config.id)`.

---

### 2.2 `src/pages/instances/instancecardprojects.tsx` (`InstanceCardProjects.tsx`)
- **Role**: Renders the "Active / Recent Projects" container within the instance card, showcasing projects mapped to this instance profile.
- **Running-First Project Sorting Logic (Lines 14–23)**:
  Prioritizes actively executing projects to guarantee immediate visibility without manual scrolling:

```tsx
// Lines 14–23 in src/pages/instances/instancecardprojects.tsx
const sortedProjects = [...instanceProjects].sort((a, b) => {
    const isARunning = Boolean(inst.is_running) && Boolean(a.is_running);
    const isBRunning = Boolean(inst.is_running) && Boolean(b.is_running);
    if (isARunning && !isBRunning) return -1;
    if (isBRunning && !isARunning) return 1;
    const aLatest = Math.max(0, ...(a.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
    const bLatest = Math.max(0, ...(b.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
    if (bLatest !== aLatest) return bLatest - aLatest;
    return a.repo_name.localeCompare(b.repo_name);
});
```

- **Running Badge Rendering (Lines 72–76)**:
  Renders an emerald status badge whenever `isProjRunning` (`Boolean(inst.is_running) && Boolean(proj.is_running)`) is `true`:

```tsx
// Lines 72–76 in src/pages/instances/instancecardprojects.tsx
{isProjRunning && (
    <span className="px-1 py-0.2 rounded-[4px] text-[9px] font-bold bg-[#1af18d]/15 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 flex items-center gap-0.5 shadow-2xs">
        <span className="w-1 h-1 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse" />
        RUNNING
    </span>
)}
```

- **Click-to-Focus Deep-Linking**:
  Clicking any project row invokes `setPromptTreeInstance`, passing `projectId: proj.project_id`. This instructs `PromptTreeViewModal` to open directly targeted at that project.

---

### 2.3 `src/pages/instances/instancecardactions.tsx` (`InstanceCardActions.tsx`)
- **Role**: Houses the two-tier primary and secondary action capsules for instance operations.
- **Slot 1: Prompt Tree Button with Cyan Pulse (Lines 144–160)**:
  Mounted as the first slot of Row 2 (Secondary Actions Capsule):

```tsx
// Lines 144–160 in src/pages/instances/instancecardactions.tsx
{/* Slot 1: Prompts Tree */}
<button
    type="button"
    disabled={isBusy}
    onClick={() => openPromptTree(inst.config.id)}
    className={cn(
        "flex-1 flex items-center justify-center py-1 transition-colors cursor-pointer relative rounded-l-[4px] disabled:opacity-50 disabled:cursor-not-allowed",
        hasActiveTask
            ? "bg-cyan-50 dark:bg-cyan-950/40 text-cyan-600 dark:text-cyan-300 hover:bg-cyan-100 dark:hover:bg-cyan-900/50"
            : "text-slate-600 dark:text-slate-400 hover:text-cyan-600 dark:hover:text-cyan-300 hover:bg-slate-200 dark:hover:bg-[#15334d]"
    )}
    title={hasActiveTask ? "Prompt Tree (Active Task Running)" : "Prompt Tree"}
>
    <Layers className="w-3.5 h-3.5" />
    {hasActiveTask && (
        <span className="absolute top-0.5 right-0.5 w-1.5 h-1.5 rounded-full bg-cyan-500 animate-pulse" />
    )}
</button>
```

- **Visual Signaling**:
  - Idle state: Standard slate palette (`text-slate-600 dark:text-slate-400`).
  - Active task state: Soft cyan highlight (`bg-cyan-50 dark:bg-cyan-950/40 text-cyan-600 dark:text-cyan-300`) with a pulsating cyan dot positioned at the top-right corner (`bg-cyan-500 animate-pulse`).

---

### 2.4 `src/pages/instances/instancecarddata.ts` (`instanceCardData.ts`)
- **Role**: Data-aggregation hook (`useInstanceCardData`) decoupling UI presentation from backend state.
- **Active Task & Instance Ownership Calculation (Lines 76–87)**:
  Computes `hasActiveTask` with strict multi-layer filtering:

```typescript
// Lines 76–87 in src/pages/instances/instancecarddata.ts
const hasActiveTask =
    Boolean(inst.is_running) &&
    runningTreeNodes.some((node) => {
        const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
        const hasGenuineRunningConv = (node.conversations || []).some(
            (c) =>
                Boolean(c.is_running) &&
                !isGhostConversation(c) &&
                !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
        );
        return isInstanceMatch && Boolean(node.is_running) && hasGenuineRunningConv;
    });
```

- **Invariant Enforcement**:
  1. **Process Gating**: If `Boolean(inst.is_running)` is `false`, `hasActiveTask` is immediately `false`. Terminated instances never display running badges.
  2. **Strict Ownership Isolation**: Checked via `isNodeOwnedByInstance(node, inst.config)` from `src/pages/instances/instancepageutils.ts`. Prevents cloned profiles from adopting default instance tasks or vice-versa.
  3. **Genuine Conversation Verification**:
     - `Boolean(c.is_running) === true`
     - `!isGhostConversation(c)` (excludes untitled empty conversations)
     - `!(c.prompt_word_count === 0 && !c.prompt_preview_200w?.trim())`

---

### 2.5 `src/components/instances/prompttreeview/useprompttreedata.ts` (`usePromptTreeData.ts`)
- **Role**: Manages data fetching, filtering, and automated selection for `PromptTreeViewModal`.
- **Auto-Expansion & Auto-Selection Engine (Lines 88–209)**:
  Executes `performAutoSelection` upon opening the modal to automatically locate and focus running tasks:

```typescript
// Excerpt from lines 88–209 in src/components/instances/prompttreeview/useprompttreedata.ts
const performAutoSelection = useCallback(
    (data: AgmProjectTreeNode[], currentArchivedIds: string[], currentPinnedIds: string[]) => {
        if (data.length === 0) {
            setSelectedProject(null);
            setSelectedConversation(null);
            return;
        }

        // Direct deep-link focus if initialSelectedProjectId is provided
        if (initialSelectedProjectId) {
            const target = data.find((p) => p.project_id === initialSelectedProjectId);
            if (target) {
                setExpandedProjects({ [target.project_id]: true });
                setSelectedProject(target);
                if (target.conversations && target.conversations.length > 0) {
                    const runningConv = target.conversations.find(
                        (c) => Boolean(c.is_running) && isValidConversation(c)
                    );
                    if (runningConv) {
                        selectConversation(runningConv, target);
                        setExpandedStaleGroups(
                            isStaleOrEmptyConversation(runningConv) ? { [target.project_id]: true } : {}
                        );
                        return;
                    }
                    // Fallback to latest valid conversation
                    const candidateConvs = target.conversations.filter(isValidConversation);
                    if (candidateConvs.length > 0) {
                        const winningConv = sortByLastModifiedDesc(candidateConvs)[0];
                        selectConversation(winningConv, target);
                        setExpandedStaleGroups(
                            isStaleOrEmptyConversation(winningConv) ? { [target.project_id]: true } : {}
                        );
                    }
                }
                return;
            }
        }

        // Target non-archived projects in prioritized order
        const nonArchived = data.filter((p) => !currentArchivedIds.includes(p.project_id));
        const targetPool = nonArchived.length > 0 ? nonArchived : data;

        const prioritized = [...targetPool].sort((a, b) => {
            const aPinned = currentPinnedIds.includes(a.project_id);
            const bPinned = currentPinnedIds.includes(b.project_id);
            if (aPinned !== bPinned) return aPinned ? -1 : 1;

            const aRunning = Boolean(a.is_running) || a.conversations.some((c) => Boolean(c.is_running));
            const bRunning = Boolean(b.is_running) || b.conversations.some((c) => Boolean(c.is_running));
            if (aRunning !== bRunning) return aRunning ? -1 : 1;

            const aLatest = Math.max(0, ...a.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
            const bLatest = Math.max(0, ...b.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
            if (bLatest !== aLatest) return bLatest - aLatest;

            return a.repo_name.localeCompare(b.repo_name);
        });

        // 1. Look for actively running conversation across non-archived projects
        const runningCandidates: RunningCand[] = [];
        for (const proj of prioritized) {
            for (const conv of proj.conversations) {
                if (Boolean(conv.is_running) && isValidConversation(conv)) {
                    runningCandidates.push({
                        conv,
                        proj,
                        timestamp: new Date(conv.last_modified).getTime() || 0,
                    });
                }
            }
        }

        if (runningCandidates.length > 0) {
            // Pick newest running conversation
            runningCandidates.sort((a, b) => b.timestamp - a.timestamp);
            const winning = runningCandidates[0];
            setExpandedProjects({ [winning.proj.project_id]: true });
            selectConversation(winning.conv, winning.proj);
            setExpandedStaleGroups(
                isStaleOrEmptyConversation(winning.conv) ? { [winning.proj.project_id]: true } : {}
            );
            return;
        }

        // 2. Fallback to running project or newest project conversation
        // ...
    },
    [selectConversation, initialSelectedProjectId, setSelectedProject, setSelectedConversation, setExpandedProjects, setExpandedStaleGroups]
);
```

- **Key Capabilities**:
  1. **Direct Deep-Link Resolution**: If `initialSelectedProjectId` is provided (e.g. from an instance project card click), that project is instantly expanded and focused.
  2. **Active Conversation Detection**: Across all projects, collects any conversation where `Boolean(conv.is_running) && isValidConversation(conv)` is true.
  3. **Timestamp Tie-Breaker**: Sorts running candidates by `b.timestamp - a.timestamp`, ensuring the user lands directly on the newest active prompt.
  4. **Auto-Expansion**: Automatically sets `setExpandedProjects({ [winning.proj.project_id]: true })`, ensuring the user sees the active node in the tree immediately.

---

## 3. UI Token & Styling Reference

| Element | Location | Background / Border | Animation & Badges |
| :--- | :--- | :--- | :--- |
| **Instance Header Prompt Pill** | `InstanceCard.tsx` (top bar) | `bg-[#1af18d]/15`, `border-[#1af18d]/30` | Pulsing emerald dot (`bg-[#1af18d] animate-pulse`) |
| **Project Row Running Badge** | `InstanceCardProjects.tsx` (project list) | `bg-[#1af18d]/15`, `border-[#1af18d]/30` | Pulsing emerald dot (`bg-[#1af18d] animate-pulse`), `text-[9px] font-bold` |
| **Action Capsule Slot 1** | `InstanceCardActions.tsx` (row 2) | `bg-cyan-50 dark:bg-cyan-950/40` | Cyan pulse indicator (`bg-cyan-500 animate-pulse` top-right) |
| **Tree Conversation Node** | `ConversationNode.tsx` (prompt tree) | Dark-glass row highlight | Pulsing green dot + 4-tier origin badges (`USER_PROMPT`, etc.) |
| **Tree Filter Pill** | `TreePanel.tsx` (filter bar) | Segmented pill capsule | Filter toggle (`Running` active count) |

---

## 4. Automated Testing & Verification Gate

The component behaviors specified above are verified by the automated frontend test suite:
- **Test File**: `src/pages/instances/__tests__/instancerunningprompts.test.ts`
- **Execution Script**: `scripts/run-frontend-tests.mjs`
- **Verification Command**:
  ```bash
  node scripts/run-frontend-tests.mjs instanceRunningPrompts
  ```
- **Test Invariants Checked**:
  1. `isNodeOwnedByInstance` correctly associates projects to default & cloned instances, preventing cross-profile leakage.
  2. Sorting logic in `InstanceCardProjects` places running projects at the top.
  3. `hasActiveTask` returns `true` only when an active conversation is genuine and the instance process is alive.
  4. Prompt status identification marks running conversations and filters out ghost/empty conversations.
