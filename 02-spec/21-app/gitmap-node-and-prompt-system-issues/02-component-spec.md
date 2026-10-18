# Component Specification: Prompt Tree UX, Stacking Context & Glitch Elimination

- **Feature / Component**: Project & Prompt Tree View (`PromptTreeViewModal` and subcomponents)
- **Module Path**: `src/components/instances/PromptTreeView/`
- **Spec Document**: `02-spec/21-app/gitmap-node-and-prompt-system-issues/02-component-spec.md`
- **Subtask Plan**: `.ai-memory/plans/subtasks/gitmap-node-and-prompt-system-issues/02-prompt-tree-ux-and-glitches.md`
- **Status**: Draft / Complete

---

## 1. Executive Summary & Problem Formulation

The Project & Prompt Tree View modal (`PromptTreeViewModal.tsx`) serves as the central control plane for inspecting multi-turn conversations, live execution heartbeats, background queue sequencing, and full database persistence across Antigravity IDE instances. While the underlying backend engine (`repo_db.rs`, `backup_prompts_db.rs`) maintains strict First-In-First-Out (FIFO) ordering and durable transactional records, several frontend UI/UX regressions and visual glitches degrade usability and cause jarring interface instability:

1. **Import/Export Dropdown Z-Index Stacking Context Bug**:
   - *Problem*: In `ModalHeader.tsx`, the top-level container lacked an explicit stacking context and z-index (`z-40`), while `DetailHeader.tsx` established `relative z-30` and the queue banners mounted intermediate stacking levels. As a result, opening `ImportExportMenu.tsx` caused the flyout dropdown (`z-50`) to render beneath `DetailHeader.tsx`'s action buttons and clipping beneath the preview pane.
   - *Root Cause*: CSS Stacking Context isolation: without `relative z-40` on `ModalHeader`, child elements with `z-50` remain trapped in their parent stacking context and are occluded by sibling containers that appear later in DOM order with `z-30`.
   - *Resolution*: Establish `relative z-40` on `ModalHeader.tsx` and ensure `ImportExportMenu.tsx` maintains `relative z-50` on its trigger anchor, elevating all menu portals and flyouts above the preview pane.

2. **Prompt Tree Action Buttons Indirection & Feedback Latency**:
   - *Problem*: The `ModalHeader.tsx` Backup and Restore buttons routed through intermediate multi-step modal dialogs (`handleOpenExportModal`, `handleOpenImportModal`) rather than executing direct 1-click tree backups and restores. Furthermore, clicking `Refresh` or `Reset` provided no instantaneous visual confirmation, and the `RefreshCw` icon did not spin during auto-sync or background project refreshes.
   - *Root Cause*: `PromptTreeViewModal.tsx` bound `onBackup` and `onRestore` to scoped modal openers rather than `coreActions.handleBackup()` and `coreActions.handleRestore()`. The refresh animation was bound solely to local `isLoading`, ignoring `isAutoSyncing` and per-project refresh cycles (`refreshingProjectId`).
   - *Resolution*: Wire direct 1-click invocation in `PromptTreeViewModal.tsx` to `coreActions.handleBackup()` and `coreActions.handleRestore()`. Add spinning animations to `RefreshCw` when `isLoading || isAutoSyncing || isRefreshing`. Dispatch immediate optimistic notification banners (`setActionMsg`) upon clicking `Reset` and `Refresh`.

3. **Screen Glitch & Flicker Elimination (Cold-Start vs Hot-Cycle Invariant)**:
   - *Problem*: During periodic 15s/30s background sync cycles or manual refresh requests, the right-hand preview panel and left tree panel flashed animated skeleton loaders, wiping away the user's active view, causing layout thrashing, and inducing visual seizure-like flicker.
   - *Root Cause*: `PromptTreeViewModal.tsx` evaluated `isLoading` unconditionally (`selectedConversation ? ... : isLoading ? <Skeleton> : <EmptyState>`). When a user was viewing an empty state or transitioning between projects, background sync triggers flipped `isLoading` or `isAutoSyncing`, tearing down active DOM elements and mounting pulse shimmer blocks.
   - *Resolution*: Strictly guard skeleton loaders: render animated skeletons ONLY during initial cold starts where `treeData.length === 0`. During background syncs, keep existing UI components mounted and render a non-intrusive 2px indeterminate top progress line (`h-0.5 bg-[#1af18d]`). Prevent auto-selection flapping by locking onto the currently active conversation ID.

4. **Prompt Ordering & Badge Hierarchy**:
   - *Problem*: In `conversationGuards.ts`, `isStaleOrEmptyConversation` utilized exact case comparison (`status === 'RUNNING' || status === 'QUEUED'`), causing conversations reporting lowercase `'running'` or `'queued'` from different backend channels to be misclassified as stale or empty and grouped away. In `ProjectNode.tsx` and `tierClassification.ts`, sorting only differentiated running versus non-running, leaving queued items unsorted or buried under idle timestamps. Additionally, subagent tasks lacked explicit queued badges.
   - *Root Cause*: Binary sorting (`is_running ? -1 : 1`) lacking 3-tier precedence. Exact case matching without string normalization (`status.toLowerCase().includes(...)`).
   - *Resolution*: Implement a robust 3-tier deterministic sorting algorithm (`Running -> Queued -> Timestamp Descending`) across `ProjectNode.tsx` and `tierClassification.ts`. Normalize status comparisons using `status.toLowerCase().includes('run')` and `'queue'`. Render explicit emerald pulsating indicators (`#1af18d`) for running tasks and amber clock badges (`Clock`, `text-amber-500`) for queued tasks across primary and subagent nodes.

5. **Preview Header Decluttering & Metadata Relocation**:
   - *Problem*: `DetailHeader.tsx` displayed redundant visual tokens: a large `"User Prompt"` badge when origin was obvious from the user icon, plus an intrusive `instanceNameDisplay` token that duplicated information already present in `ModalHeader.tsx` (`#sequenceNumber instanceName`).
   - *Root Cause*: Clutter accumulated from incremental badge additions without consolidation.
   - *Resolution*: Remove the redundant text badge for `User Prompt` (retaining the clean sky user icon) and eliminate the duplicate `instanceNameDisplay` text token from `DetailHeader.tsx`. Relocate exhaustive runtime telemetry, executable paths, and PID metadata into the dedicated `DetailsModal.tsx`.

---

## 2. Architectural Invariants & UI/UX Standards

### Invariant 1: Stacking Context Hierarchy
The Prompt Tree View modal establishes a layered z-index ladder ensuring menus and popovers never clip behind child or sibling panels:

```
+--------------------------------------------------------------+
| Modal Wrapper Backdrop (z-[99999])                           |
|   +--------------------------------------------------------+ |
|   | Modal Container (relative z-10)                        | |
|   |   +--------------------------------------------------+ | |
|   |   | ModalHeader (relative z-40)                      | | |
|   |   |   +-- ImportExportMenu Flyout (z-50)             | | |
|   |   +--------------------------------------------------+ | |
|   |   | Progress Bar & Notification Banners (z-20)       | | |
|   |   +--------------------------------------------------+ | |
|   |   | Main Split Pane (z-10)                           | | |
|   |   |   +-- TreePanel (relative z-10)                  | | |
|   |   |   +-- Detail Pane & DetailHeader (relative z-30) | | |
|   |   +--------------------------------------------------+ | |
|   +--------------------------------------------------------+ |
+--------------------------------------------------------------+
```

1. **`ModalHeader.tsx` MUST be `relative z-40`**: Any child flyout (such as `ImportExportMenu.tsx` with `z-50`) will naturally float above `DetailHeader.tsx` (`z-30`) and all queue banners (`z-20`).
2. **`DetailHeader.tsx` MUST NOT exceed `z-30`**: Its internal flyouts (Copy options, Rerun menu) must remain strictly constrained to `z-30` so they never occlude header controls.

### Invariant 2: SWR Zero-Flicker & Cold-Start Guard
- **Cold Start (`treeData.length === 0`)**: When the modal first mounts without cached data, animated skeleton shimmers (`animate-pulse`) indicate initial fetch progress.
- **Hot Cycle (`treeData.length > 0`)**: During auto-sync, background polling, or manual refresh, the UI MUST retain existing data. Skeletons are strictly forbidden. Progress is indicated exclusively by the top indeterminate line (`h-0.5 bg-[#1af18d] animate-pulse`) and spinning button icons (`RefreshCw className="animate-spin"`).

### Invariant 3: 3-Tier Deterministic Sorting
Every list of conversations (root conversations, subagents, and project items) MUST adhere to deterministic 3-tier ordering:
$$\text{Rank}(C) = \begin{cases} 0 & \text{if } C \text{ is Running} \\ 1 & \text{if } C \text{ is Queued} \\ 2 & \text{if } C \text{ is Idle / Finished} \end{cases}$$

When $\text{Rank}(A) = \text{Rank}(B)$, break ties deterministically by $\text{last\_modified}$ descending:
$$\text{Time}(A) > \text{Time}(B) \implies A \text{ precedes } B$$

### Invariant 4: 4-Tier Origin Classification Hierarchy
Adhering to project guidelines in `AGENTS.md`:
1. `USER_PROMPT`: Sky badge (`bg-sky-500/15 text-sky-700 dark:text-cyan-300`), User icon (`User`).
2. `SUBAGENT_INSTRUCTION`: Purple badge (`bg-purple-500/20 text-purple-900 dark:text-amber-200`), Bot icon (`Bot`), rendered with tree branch glyphs (`↳`, `└──`).
3. `SYSTEM_MESSAGE`: Zinc badge (`bg-zinc-500/15 text-zinc-700 dark:text-zinc-300`), Terminal icon (`Terminal`).
4. `TOOL_OUTPUT`: Amber badge (`bg-amber-500/15 text-amber-700 dark:text-amber-300`), Wrench icon (`Wrench`).

---

## 3. Component Specifications & Interfaces

### 3.1 `ModalHeader.tsx`
- **File**: `src/components/instances/PromptTreeView/ModalHeader.tsx`
- **Role**: Top-level modal toolbar housing instance identity, segmented capsule controls, sync rate selector, and window controls.

#### Interface Specification:
```typescript
export interface ModalHeaderProps {
    instanceName: string;
    sequenceNumber?: number;
    executablePath?: string;
    isFullscreen: boolean;
    setIsFullscreen: (v: boolean | ((prev: boolean) => boolean)) => void;
    isLoading: boolean;
    isAutoSyncing?: boolean;
    isRefreshing?: boolean;
    syncInterval: SyncInterval;
    setSyncInterval: (v: SyncInterval) => void;
    restoreFileInputRef: React.RefObject<HTMLInputElement | null>;
    onClose: () => void;
    onBackup: () => void;
    onRestore: () => void;
    onExportSqlite: () => void;
    onImportSqlite: () => void;
    onCopyAllTreePrompts: () => Promise<void> | void;
    onFileRestore: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onRefreshAll: () => void;
    onResetTree: () => void;
    isTreeCollapsed?: boolean;
    onToggleTree?: () => void;
}
```

#### DOM & Styling Requirements:
1. Outer wrapper container:
   ```tsx
   <div className="relative z-40 flex items-center justify-between border-b border-slate-200/80 dark:border-[#40285e] px-4 py-2.5 bg-slate-50/80 dark:bg-[#090611] shrink-0">
   ```
2. Refresh button spinning indicator:
   ```tsx
   const isSpinning = isLoading || Boolean(props.isAutoSyncing) || Boolean(props.isRefreshing);
   <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", isSpinning && "animate-spin")} />
   ```
3. Seamless Segmented Capsule: Ensure the `ImportExportMenu`, `Reset`, `Refresh`, `Sync`, `Show/Hide Tree`, `Fullscreen`, and `Close` buttons remain wrapped in a single contiguous `rounded-full` capsule with `divide-x` borders and transparent inline dropdown styling.

---

### 3.2 `ImportExportMenu.tsx`
- **File**: `src/components/instances/PromptTreeView/ImportExportMenu.tsx`
- **Role**: Unified flyout menu consolidating tree-level operations (SQLite export/import, JSON backup/restore, status downloads, and copy-all).

#### Interface Specification:
```typescript
export interface ImportExportMenuProps {
    onExportSqlite: () => void;
    onExportJson: () => void;
    onImportSqlite: () => void;
    onImportJson: () => void;
    onCopyAllTree: () => Promise<void> | void;
    isLoading: boolean;
}
```

#### Stacking & Interaction Requirements:
1. Anchor Container:
   ```tsx
   <div className="relative z-50 flex items-center" ref={rootRef}>
   ```
2. Flyout Menu Popover:
   ```tsx
   {isOpen && (
       <div className="absolute right-0 top-full mt-1.5 z-50 w-64 rounded-xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-1.5 animate-in fade-in zoom-in-95 duration-100 backdrop-blur-md space-y-1">
   ```
3. Accessibility: Close on Escape key or pointer-down outside root element.

---

### 3.3 `PromptTreeViewModal.tsx`
- **File**: `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- **Role**: Main container orchestrating state hooks, data loading, hotkey navigation, and layout rendering.

#### Implementation Requirements:
1. **Direct 1-Click Backup & Restore**:
   - Change `ModalHeader` prop bindings:
     ```tsx
     <ModalHeader
         ...
         onBackup={coreActions.handleBackup}
         onRestore={coreActions.handleRestore}
         onRefreshAll={() => {
             setActionMsg('Refreshing prompt tree...');
             void loadTree(false, true);
         }}
         onResetTree={() => {
             setActionMsg('Resetting prompt tree cache...');
             void invoke('reset_prompt_tree_cache', { instanceId: instanceId || null })
                 .catch((err) => {
                     useErrorStore.getState().captureException(
                         err instanceof Error ? err : new Error(String(err)),
                         { source: 'PromptTreeViewModal.onResetTree', triggerAction: 'reset_prompt_tree_cache' }
                     );
                 })
                 .finally(() => {
                     void loadTree(false, true);
                 });
         }}
     />
     ```
2. **Skeleton Loading Guard**:
   - In Right Panel (lines ~816):
     ```tsx
     {selectedConversation ? (
         <div className="flex-1 flex flex-col px-5 py-3.5 overflow-y-auto space-y-1.5">
             ...
         </div>
     ) : isLoading && treeData.length === 0 ? (
         <div className="flex-1 flex flex-col px-6 py-5 space-y-4 animate-pulse overflow-hidden bg-white dark:bg-[#171025]">
             {/* Skeleton Shimmer */}
         </div>
     ) : (
         <DetailEmptyState
             selectedProject={selectedProject}
             isResending={isResending}
             onResend={promptActions.handleResendPrompt}
         />
     )}
     ```
3. **Flapping Prevention**:
   - When background sync triggers `setTreeData`, preserve `selectedConversation` using stable ID matching (`c.conversation_id === selectedConversation.conversation_id`).
   - Guard deep-link selection so it only runs if `selectedConversation` is null or if `initialSelectedProjectId` changed.

---

### 3.4 `TreePanel.tsx`
- **File**: `src/components/instances/PromptTreeView/TreePanel.tsx`
- **Role**: Left-hand navigation panel rendering filters, search inputs, archived toggles, and hierarchical project lists.

#### Requirements:
1. **Cold Start Skeleton Guard**:
   - Ensure line ~448 checks:
     ```tsx
     isEffectiveLoading && treeData.length === 0 ? (
         /* Animated Skeleton Shimmer */
     ) : (
         /* Active / Archived Projects */
     )
     ```
2. **Prop Synchronization**:
   - Forward `isRefreshing={Boolean(refreshingProjectId)}` to `ModalHeader` if needed.

---

### 3.5 `conversationGuards.ts`
- **File**: `src/components/instances/PromptTreeView/conversationGuards.ts`
- **Role**: Domain predicate functions identifying ghost, empty, and stale conversations.

#### Exact Logic Implementation:
```typescript
import type { AgmConversationNode } from './types';

export function isGhostConversation(conv: AgmConversationNode): boolean {
    const status = (conv.status || '').toLowerCase();
    if (
        Boolean(conv.is_running) ||
        Boolean(conv.is_queued) ||
        status.includes('run') ||
        status.includes('queue')
    ) {
        return false;
    }

    const title = (conv.title || '').trim().toLowerCase();
    const isUntitled =
        !title ||
        title === 'untitled' ||
        title.startsWith('untitled') ||
        title === 'new conversation' ||
        title.startsWith('new conversation') ||
        title === 'conversation' ||
        title.startsWith('conversation-') ||
        title.startsWith('chat-') ||
        title === 'chat' ||
        title === (conv.short_id || '').toLowerCase();

    const isEmptyPrompt =
        conv.prompt_word_count === 0 ||
        !conv.prompt_preview_200w ||
        conv.prompt_preview_200w.trim().length === 0;

    return isUntitled && isEmptyPrompt;
}

export function isStaleOrEmptyConversation(conv: AgmConversationNode): boolean {
    if (isGhostConversation(conv)) {
        return true;
    }

    const status = (conv.status || '').toLowerCase();
    // Running OR queued conversations must NEVER be treated as stale or empty
    if (
        Boolean(conv.is_running) ||
        Boolean(conv.is_queued) ||
        status.includes('run') ||
        status.includes('queue')
    ) {
        return false;
    }

    const isEmptyPrompt =
        conv.prompt_word_count === 0 ||
        !conv.prompt_preview_200w ||
        conv.prompt_preview_200w.trim().length === 0;

    // 0-word empty untitled conversations must NEVER be treated as running or active
    if (isEmptyPrompt) {
        return true;
    }

    const title = (conv.title || '').trim().toLowerCase();
    const isUntitled =
        !title ||
        title === 'untitled' ||
        title.startsWith('untitled') ||
        title === 'new conversation' ||
        title.startsWith('new conversation') ||
        title === 'conversation' ||
        title.startsWith('conversation-') ||
        title.startsWith('chat-') ||
        title === 'chat' ||
        title === (conv.short_id || '').toLowerCase();

    return isUntitled;
}

export const isUntitledOrEmpty = isStaleOrEmptyConversation;
```

---

### 3.6 `tierClassification.ts` & `ProjectNode.tsx`
- **Files**:
  - `src/components/instances/PromptTreeView/tierClassification.ts`
  - `src/components/instances/PromptTreeView/ProjectNode.tsx`
- **Role**: 4-tier origin classification, hierarchical conversation tree assembly, and deterministic sorting.

#### 3-Tier Deterministic Sorting Contract:
In `ProjectNode.tsx`:
```typescript
export function sortConversations(
    convs: AgmConversationNode[],
    activeFilter: ProjectNodeProps['activeFilter']
): AgmConversationNode[] {
    let sorted = [...convs];
    if (activeFilter === 'running') {
        sorted = sorted.filter((c) => {
            const status = (c.status || '').toLowerCase();
            return Boolean(c.is_running) || status.includes('run');
        });
    } else if (activeFilter === 'queued') {
        sorted = sorted.filter((c) => {
            const status = (c.status || '').toLowerCase();
            return (Boolean(c.is_queued) || status.includes('queue')) && !c.is_running;
        });
    }

    return sorted.sort((a, b) => {
        const aStatus = (a.status || '').toLowerCase();
        const bStatus = (b.status || '').toLowerCase();

        const aRunning = Boolean(a.is_running) || aStatus.includes('run');
        const bRunning = Boolean(b.is_running) || bStatus.includes('run');

        // Tier 1: Running
        if (aRunning !== bRunning) {
            return aRunning ? -1 : 1;
        }

        const aQueued = (Boolean(a.is_queued) || aStatus.includes('queue')) && !aRunning;
        const bQueued = (Boolean(b.is_queued) || bStatus.includes('queue')) && !bRunning;

        // Tier 2: Queued
        if (aQueued !== bQueued) {
            return aQueued ? -1 : 1;
        }

        // Tier 3: Timestamp Descending
        const aTime = new Date(a.last_modified).getTime() || 0;
        const bTime = new Date(b.last_modified).getTime() || 0;
        return bTime - aTime;
    });
}
```

In `tierClassification.ts` (`assembleConversationHierarchy`):
```typescript
    // Reverse rootNodes so the newest user prompt appears at top,
    // and sort running then queued roots to the top
    return rootNodes.reverse().sort((a, b) => {
        const aStatus = (a.primaryNode.status || '').toLowerCase();
        const bStatus = (b.primaryNode.status || '').toLowerCase();

        const aRunning =
            Boolean(a.primaryNode.is_running) ||
            aStatus.includes('run') ||
            a.subagents.some((s) => Boolean(s.primaryNode.is_running) || (s.primaryNode.status || '').toLowerCase().includes('run'));
        const bRunning =
            Boolean(b.primaryNode.is_running) ||
            bStatus.includes('run') ||
            b.subagents.some((s) => Boolean(s.primaryNode.is_running) || (s.primaryNode.status || '').toLowerCase().includes('run'));

        if (aRunning !== bRunning) return aRunning ? -1 : 1;

        const aQueued =
            (Boolean(a.primaryNode.is_queued) || aStatus.includes('queue') ||
             a.subagents.some((s) => Boolean(s.primaryNode.is_queued) || (s.primaryNode.status || '').toLowerCase().includes('queue'))) && !aRunning;
        const bQueued =
            (Boolean(b.primaryNode.is_queued) || bStatus.includes('queue') ||
             b.subagents.some((s) => Boolean(s.primaryNode.is_queued) || (s.primaryNode.status || '').toLowerCase().includes('queue'))) && !bRunning;

        if (aQueued !== bQueued) return aQueued ? -1 : 1;

        const aTime = new Date(a.primaryNode.last_modified).getTime() || 0;
        const bTime = new Date(b.primaryNode.last_modified).getTime() || 0;
        return bTime - aTime;
    });
```

#### Badges Specification in `ProjectNode.tsx` (Subagent & Conversation Nodes):
1. **Running Badge (Emerald Pulse)**:
   ```tsx
   {isSubRunning && (
       <span className={cn(
           "flex items-center gap-1 px-1.5 py-0.2 rounded-md text-[8.5px] font-bold font-mono border animate-pulse",
           isSubSelected
               ? "bg-emerald-400 text-emerald-950 border-emerald-300"
               : "bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border-emerald-500/40"
       )}>
           <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
           RUNNING
       </span>
   )}
   ```
2. **Queued Badge (Amber Clock)**:
   ```tsx
   {isSubQueued && !isSubRunning && (
       <span className={cn(
           "flex items-center gap-1 px-1.5 py-0.2 rounded-md text-[8.5px] font-bold font-mono border",
           isSubSelected
               ? "bg-amber-300 text-amber-950 border-amber-200"
               : "bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/40"
       )}>
           <Clock className="w-2.5 h-2.5 text-amber-500" />
           <span>QUEUED</span>
       </span>
   )}
   ```

---

### 3.7 `DetailHeader.tsx` & `DetailsModal.tsx`
- **Files**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/DetailsModal.tsx`
- **Role**: Conversation preview header and extended telemetry modal.

#### Decluttering Specification:
1. **Remove redundant `"User Prompt"` text badge**:
   - When `tierInfo.tier === PromptOriginTier.UserPrompt` and `!isTurnSelected`, render only the clean sky user icon (`<User className="w-3 h-3 text-sky-500" />`) with tooltip, or omit the redundant literal text `"User Prompt"`.
   - Preserve turn markers: `isTurnSelected ? `Turn T${activeTurnNumber}` : ...`.
2. **Remove redundant `instanceNameDisplay` token**:
   - Eliminate lines ~484-490 rendering `{instanceNameDisplay}` in `DetailHeader.tsx`.
3. **Relocate Runtime Metadata to `DetailsModal.tsx`**:
   - `DetailsModal.tsx` already features a comprehensive "Runtime Environment" telemetry grid.
   - Ensure `instanceNameDisplay`, `instanceSeqNum`, `instanceExeName`, `executablePath`, and `instancePid` are cleanly formatted inside `DetailsModal.tsx` with copy actions.

---

## 4. Verification Contracts & Quality Criteria

### 4.1 Visual Verification Checklist
1. **Dropdown Stacking**: Open `Import / Export` menu in `ModalHeader`. Verify the popup appears fully visible on top of `DetailHeader` and does not get clipped or hidden beneath `DetailHeader`.
2. **Action Buttons**: Click `Backup`. Verify file save dialog immediately initiates without showing modal prompt. Click `Restore`. Verify native file picker opens directly. Click `Refresh` and `Reset`. Verify notification banner displays immediately.
3. **Icon Spin**: During manual refresh or auto-sync, verify `RefreshCw` spins smoothly.
4. **Flicker-Free SWR**: While viewing a conversation, trigger background sync. Verify preview pane remains completely stable with zero skeleton flickering.
5. **3-Tier Deterministic Order**: Verify running tasks appear at top with emerald pulse badge, queued tasks appear immediately below running tasks with amber clock badge, and idle tasks sort by timestamp descending.
6. **Preview Header Cleanliness**: Verify `DetailHeader.tsx` is uncluttered, showing title, double-click rename, status badge, and action capsule without duplicate instance names or redundant text tags.

### 4.2 Error Handling & Pre-flight Gate Invariants
- All async invoke calls must be mapped to `useErrorStore.getState().trackWarning` or `captureException`.
- Zero raw search tools (`rg`, `grep`, `findstr`, etc.) and zero unmocked git commands during verification.
- Full TypeScript type-checking (`npm run build`) and Rust pre-flight checks (`cargo fmt -- --check`, `cargo clippy --all-targets --all-features`) must pass.
