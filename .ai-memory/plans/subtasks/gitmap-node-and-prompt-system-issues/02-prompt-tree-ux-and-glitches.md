# Subtask Plan: Prompt Tree UX, Stacking Context & Glitch Elimination

- **Subtask ID**: `02-prompt-tree-ux-and-glitches`
- **Parent Task**: `gitmap-node-and-prompt-system-issues`
- **Spec Reference**: `02-spec/21-app/gitmap-node-and-prompt-system-issues/02-component-spec.md`
- **Status**: Ready for Implementation
- **Estimated Scope**: 5 phases, 8 files touched

---

## 1. Overview & Objectives

This subtask resolves 5 high-impact UX regressions, stacking context collisions, and visual glitch defects within the Project & Prompt Tree View subsystem (`src/components/instances/PromptTreeView/`):
1. **Fix Z-Index Stacking Context**: Prevent the `ImportExportMenu` dropdown from clipping beneath `DetailHeader.tsx` (`z-30`) by adding `relative z-40` to `ModalHeader.tsx` and ensuring `ImportExportMenu.tsx` anchor maintains `relative z-50`.
2. **Direct 1-Click Action Buttons & Instant Feedback**: Re-wire Backup and Restore buttons in `PromptTreeViewModal.tsx` directly to `coreActions.handleBackup()` and `coreActions.handleRestore()`. Add spinning animation to the `RefreshCw` icon during auto-sync and refresh operations. Add instant `setActionMsg` visual confirmation on Reset and Refresh button clicks.
3. **Flicker-Free Screen Stability**: Guard animated skeleton loaders in `TreePanel.tsx` and `PromptTreeViewModal.tsx` so they only display on cold start (`treeData.length === 0`), eliminating jarring flashes during background polling or manual refresh cycles. Prevent auto-selection flapping when background updates arrive.
4. **Deterministic 3-Tier Sorting & Badges**: Update `conversationGuards.ts` so `isStaleOrEmptyConversation` performs case-insensitive normalization (`status.toLowerCase().includes('run')` and `'queue'`). Implement 3-tier sorting (`Running -> Queued -> Timestamp Descending`) in `ProjectNode.tsx` and `tierClassification.ts`. Add explicit emerald pulse (`RUNNING`) and amber clock (`QUEUED`) badges across conversation nodes and subagent leaves.
5. **Preview Header Decluttering**: Remove redundant `"User Prompt"` text badge and duplicate `instanceNameDisplay` token from `DetailHeader.tsx`. Relocate full runtime metadata to `DetailsModal.tsx`.

---

## 2. File Boundaries & Responsibility Matrix

| File Path | Role | Key Changes |
| :--- | :--- | :--- |
| `src/components/instances/PromptTreeView/ModalHeader.tsx` | Header Toolbar | Add `relative z-40` to container; accept `isAutoSyncing` and `isRefreshing`; spin `RefreshCw` when active. |
| `src/components/instances/PromptTreeView/ImportExportMenu.tsx` | Export/Import Flyout | Ensure menu anchor has `relative z-50` and popover has `z-50`. |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | Main Modal Orchestrator | Bind `onBackup`/`onRestore` to direct 1-click methods; add instant `setActionMsg`; guard skeleton loader (`treeData.length === 0`); pass sync flags; prevent flapping. |
| `src/components/instances/PromptTreeView/TreePanel.tsx` | Left Tree Panel | Guard skeleton loader so it only renders when `treeData.length === 0`; ensure smooth list transitions. |
| `src/components/instances/PromptTreeView/conversationGuards.ts` | Guard Functions | Normalize status comparisons using `status.toLowerCase().includes('run')` and `'queue'` to prevent misclassifying active prompts as stale or empty. |
| `src/components/instances/PromptTreeView/tierClassification.ts` | Classification & Hierarchy | Implement 3-tier deterministic sorting (`Running -> Queued -> Timestamp Descending`) in `assembleConversationHierarchy`. |
| `src/components/instances/PromptTreeView/ProjectNode.tsx` | Project & Conversation List | Implement 3-tier deterministic sorting in `sortConversations`; render emerald pulse and amber clock badges on subagents. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | Preview Header | Remove redundant `User Prompt` text label (retain user icon); remove duplicate `instanceNameDisplay` token; maintain `z-30`. |
| `src/components/instances/PromptTreeView/DetailsModal.tsx` | Detailed Telemetry Modal | Ensure runtime environment section houses full instance and process metadata cleanly. |

---

## 3. Step-by-Step Implementation Sequence

### Phase 1: Stacking Context & Z-Index Remediation

#### Goal:
Ensure the `ImportExportMenu` dropdown floats above `DetailHeader` and all banners without clipping.

#### Detailed Steps:
1. Edit `src/components/instances/PromptTreeView/ModalHeader.tsx`:
   - Locate the root `<div>` element on line 72.
   - Replace `className="flex items-center justify-between border-b ..."` with `className="relative z-40 flex items-center justify-between border-b ..."`.
   - Update `ModalHeaderProps` to include optional `isAutoSyncing?: boolean;` and `isRefreshing?: boolean;`.
2. Edit `src/components/instances/PromptTreeView/ImportExportMenu.tsx`:
   - Locate the root `<div>` element on line 75.
   - Ensure the anchor wrapper has `className="relative z-50 flex items-center" ref={rootRef}`.
   - Verify the popover container retains `className="absolute right-0 top-full mt-1.5 z-50 ..."` so it stays elevated above sibling elements.

---

### Phase 2: Action Button Direct Wiring & Instant Visual Feedback

#### Goal:
Provide instant 1-click backup/restore and immediate visual feedback when triggering tree actions.

#### Detailed Steps:
1. Edit `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`:
   - Locate `<ModalHeader ... />` invocation (around lines 580-612).
   - Change `onBackup`:
     - *From*: `onBackup={() => coreActions.handleOpenExportModal({ target_type: 'full_tree' }, 'json')}`
     - *To*: `onBackup={coreActions.handleBackup}`
   - Change `onRestore`:
     - *From*: `onRestore={coreActions.handleOpenImportModal}`
     - *To*: `onRestore={coreActions.handleRestore}`
   - Change `onRefreshAll`:
     - *From*: `onRefreshAll={() => loadTree(false, true)}`
     - *To*:
       ```tsx
       onRefreshAll={() => {
           setActionMsg('Refreshing prompt tree...');
           void loadTree(false, true);
       }}
       ```
   - Change `onResetTree`:
     - Add `setActionMsg('Resetting prompt tree cache...');` before invoking `reset_prompt_tree_cache`.
   - Pass `isAutoSyncing={isAutoSyncing}` and `isRefreshing={Boolean(refreshingProjectId)}` to `ModalHeader`.
2. Edit `src/components/instances/PromptTreeView/ModalHeader.tsx`:
   - Update the Refresh button icon on lines ~137:
     ```tsx
     const isSpinning = isLoading || Boolean(props.isAutoSyncing) || Boolean(props.isRefreshing);
     <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", isSpinning && "animate-spin")} />
     ```

---

### Phase 3: Screen Glitch & Flicker Elimination

#### Goal:
Guarantee zero-flicker UI updates during background sync cycles and eliminate auto-selection flapping.

#### Detailed Steps:
1. Edit `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`:
   - Locate the right-hand panel conditional rendering around line 816:
     - *Before*:
       ```tsx
       ) : isLoading ? (
           <div className="flex-1 flex flex-col px-6 py-5 space-y-4 animate-pulse ...">
       ```
     - *After*:
       ```tsx
       ) : isLoading && treeData.length === 0 ? (
           <div className="flex-1 flex flex-col px-6 py-5 space-y-4 animate-pulse ...">
       ```
   - Auto-selection flapping prevention:
     - In `usePromptTreeData.ts` (or the selection effects in `PromptTreeViewModal.tsx`), ensure that when `treeData` reloads on background sync, if `selectedConversation` already exists, retain it by finding matching ID `c.conversation_id === selectedConversation.conversation_id`. Do NOT re-trigger `selectConversation` with a default conversation if the currently selected one is still present in `treeData`.
2. Edit `src/components/instances/PromptTreeView/TreePanel.tsx`:
   - Verify line ~448:
     ```tsx
     ) : isEffectiveLoading && treeData.length === 0 ? (
     ```
   - Ensure that when `treeData.length > 0`, the tree panel always renders the existing project list without flashing skeleton rows.

---

### Phase 4: Deterministic 3-Tier Ordering & Badges

#### Goal:
Ensure active conversations consistently sort to top (`Running -> Queued -> Timestamp Descending`) and display high-contrast status badges.

#### Detailed Steps:
1. Edit `src/components/instances/PromptTreeView/conversationGuards.ts`:
   - In `isGhostConversation(conv)`:
     ```typescript
     const status = (conv.status || '').toLowerCase();
     if (
         Boolean(conv.is_running) ||
         Boolean(conv.is_queued) ||
         status.includes('run') ||
         status.includes('queue')
     ) {
         return false;
     }
     ```
   - In `isStaleOrEmptyConversation(conv)`:
     ```typescript
     const status = (conv.status || '').toLowerCase();
     if (
         Boolean(conv.is_running) ||
         Boolean(conv.is_queued) ||
         status.includes('run') ||
         status.includes('queue')
     ) {
         return false;
     }
     ```
2. Edit `src/components/instances/PromptTreeView/ProjectNode.tsx`:
   - Update `sortConversations` (lines ~52-70):
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
             if (aRunning !== bRunning) return aRunning ? -1 : 1;

             const aQueued = (Boolean(a.is_queued) || aStatus.includes('queue')) && !aRunning;
             const bQueued = (Boolean(b.is_queued) || bStatus.includes('queue')) && !bRunning;
             if (aQueued !== bQueued) return aQueued ? -1 : 1;

             const aTime = new Date(a.last_modified).getTime() || 0;
             const bTime = new Date(b.last_modified).getTime() || 0;
             return bTime - aTime;
         });
     }
     ```
   - In subagent rendering (lines ~144-215):
     - Extract both `isRunning` and `isQueued`:
       ```typescript
       const { isRunning: isSubRunning, isQueued: isSubQueued } = getConversationStatus(subNode.primaryNode);
       ```
     - Add explicit amber clock badge for queued subagents:
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
3. Edit `src/components/instances/PromptTreeView/tierClassification.ts`:
   - In `assembleConversationHierarchy`:
     - Sort root nodes using 3-tier precedence (`Running -> Queued -> Timestamp Descending`):
       ```typescript
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

---

### Phase 5: Preview Header Decluttering & Metadata Relocation

#### Goal:
Clean up `DetailHeader.tsx` by removing redundant visual noise and centralizing telemetry in `DetailsModal.tsx`.

#### Detailed Steps:
1. Edit `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - Locate Indicator 1 on lines ~377-403:
     - Remove the redundant `"User Prompt"` text string when `tierInfo.tier === PromptOriginTier.UserPrompt` and `activeTurnNumber === null`.
     - Retain the clean sky `User` icon with title tooltip. Subagent, System, and Tool Output labels remain distinct.
     ```tsx
     <span>
         {isTurnSelected
             ? `Turn T${activeTurnNumber}`
             : tierInfo.tier === PromptOriginTier.UserPrompt
               ? ''
               : tierInfo.tier === PromptOriginTier.SubagentInstruction
                 ? 'Subagent'
                 : tierInfo.tier === PromptOriginTier.SystemMessage
                   ? 'System'
                   : 'Tool Output'}
     </span>
     ```
   - Locate lines ~483-490 rendering `{instanceNameDisplay}`:
     - Remove the duplicate `{instanceNameDisplay}` span token (already rendered in `ModalHeader.tsx`).
2. Verify `src/components/instances/PromptTreeView/DetailsModal.tsx`:
   - Confirm lines 80-134 cleanly display Target IDE, Instance Name, Executable Path, Process PID, and Liveness Status with copy buttons.

---

## 4. Code Diff Blueprints

### Diff 1: `src/components/instances/PromptTreeView/ModalHeader.tsx`
```diff
@@ -72,1 +72,1 @@
-        <div className="flex items-center justify-between border-b border-slate-200/80 dark:border-[#40285e] px-4 py-2.5 bg-slate-50/80 dark:bg-[#090611] shrink-0">
+        <div className="relative z-40 flex items-center justify-between border-b border-slate-200/80 dark:border-[#40285e] px-4 py-2.5 bg-slate-50/80 dark:bg-[#090611] shrink-0">

@@ -137,1 +137,2 @@
-                        <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", isLoading && "animate-spin")} />
+                        <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", (isLoading || props.isAutoSyncing || props.isRefreshing) && "animate-spin")} />
```

### Diff 2: `src/components/instances/PromptTreeView/ImportExportMenu.tsx`
```diff
@@ -75,1 +75,1 @@
-        <div className="relative flex items-center" ref={rootRef}>
+        <div className="relative z-50 flex items-center" ref={rootRef}>
```

### Diff 3: `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
```diff
@@ -593,4 +593,18 @@
-                    onBackup={() => coreActions.handleOpenExportModal({ target_type: 'full_tree' }, 'json')}
-                    onRestore={coreActions.handleOpenImportModal}
+                    onBackup={coreActions.handleBackup}
+                    onRestore={coreActions.handleRestore}
                     onExportSqlite={() => coreActions.handleOpenExportModal({ target_type: 'full_tree' }, 'sqlite')}
                     onImportSqlite={coreActions.handleOpenImportModal}
                     onCopyAllTreePrompts={coreActions.handleCopyAllTreePrompts}
                     onFileRestore={coreActions.handleFileRestore}
-                    onRefreshAll={() => loadTree(false, true)}
-                    onResetTree={() => {
+                    onRefreshAll={() => {
+                        setActionMsg('Refreshing prompt tree...');
+                        void loadTree(false, true);
+                    }}
+                    onResetTree={() => {
+                        setActionMsg('Resetting prompt tree cache...');
                         void invoke('reset_prompt_tree_cache', { instanceId: instanceId || null })
@@ -816,1 +830,1 @@
-                            ) : isLoading ? (
+                            ) : isLoading && treeData.length === 0 ? (
```

### Diff 4: `src/components/instances/PromptTreeView/conversationGuards.ts`
```diff
@@ -3,3 +3,4 @@
 export function isGhostConversation(conv: AgmConversationNode): boolean {
-    if (Boolean(conv.is_running) || Boolean(conv.is_queued) || conv.status === 'QUEUED' || conv.status === 'RUNNING') {
+    const status = (conv.status || '').toLowerCase();
+    if (Boolean(conv.is_running) || Boolean(conv.is_queued) || status.includes('run') || status.includes('queue')) {
         return false;
@@ -36,3 +37,4 @@
     // Running OR queued conversations must NEVER be treated as stale or empty
-    if (Boolean(conv.is_running) || Boolean(conv.is_queued) || conv.status === 'QUEUED' || conv.status === 'RUNNING') {
+    const status = (conv.status || '').toLowerCase();
+    if (Boolean(conv.is_running) || Boolean(conv.is_queued) || status.includes('run') || status.includes('queue')) {
         return false;
```

### Diff 5: `src/components/instances/PromptTreeView/DetailHeader.tsx`
```diff
@@ -400,1 +400,7 @@
-                                <span>{isTurnSelected ? `Turn T${activeTurnNumber}` : (tierInfo.tier === PromptOriginTier.UserPrompt ? 'User Prompt' : tierInfo.tier === PromptOriginTier.SubagentInstruction ? 'Subagent' : tierInfo.tier === PromptOriginTier.SystemMessage ? 'System' : 'Tool Output')}</span>
+                                {(isTurnSelected || tierInfo.tier !== PromptOriginTier.UserPrompt) && (
+                                    <span>
+                                        {isTurnSelected
+                                            ? `Turn T${activeTurnNumber}`
+                                            : tierInfo.tier === PromptOriginTier.SubagentInstruction ? 'Subagent' : tierInfo.tier === PromptOriginTier.SystemMessage ? 'System' : 'Tool Output'}
+                                    </span>
+                                )}
@@ -483,8 +489,0 @@
-                    {/* Subtle Instance Context */}
-                    <span
-                        className="text-[10px] font-mono text-slate-400 dark:text-slate-500 truncate cursor-help hover:text-slate-200 transition-colors"
-                        title={`Instance #${instanceSeqNum} · ${instanceExeName} (${instanceNameDisplay})`}
-                    >
-                        {instanceNameDisplay}
-                    </span>
```

---

## 5. Verification Checklist & Acceptance Gates

- [ ] **Stacking Context**: Open `Import / Export` dropdown in `ModalHeader`. Verify dropdown appears on top of `DetailHeader` (`z-30`) with zero clipping.
- [ ] **Direct Backup**: Clicking `Backup` triggers immediate file download/save dialog without intermediary modal.
- [ ] **Direct Restore**: Clicking `Restore` triggers native file picker or file input dialog directly.
- [ ] **Refresh & Reset Feedback**: Notification banners appear instantly ("Refreshing prompt tree...", "Resetting prompt tree cache...").
- [ ] **Spin Animation**: `RefreshCw` icon spins during initial loading, manual refresh, and auto-sync.
- [ ] **Zero-Flicker SWR**: Auto-sync updates do not cause skeleton shimmers when `treeData.length > 0`.
- [ ] **Selection Stability**: Active conversation selection does not jump or flap during background polling.
- [ ] **3-Tier Order**: Running items sort first (emerald pulse), Queued items sort second (amber clock), finished/idle items sort by timestamp descending.
- [ ] **Header Cleanliness**: No redundant `"User Prompt"` text string on default turn; no duplicate instance name in `DetailHeader`.
- [ ] **Compiler Gates**: `npm run build` succeeds with zero TypeScript errors; `cargo clippy --all-targets --all-features` and `cargo fmt -- --check` pass.
