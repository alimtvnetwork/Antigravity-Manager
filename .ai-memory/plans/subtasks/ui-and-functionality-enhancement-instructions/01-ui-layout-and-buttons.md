---
plan: ui-and-functionality-enhancement-instructions
subtask: "01"
title: UI Layout Modernization, Button Border-Radius Normalization, Tree Node Right-Click Context Menus, and Image Copy Pipeline
domain: frontend-ui
depends_on: []
citations:
  architecture_spec: 02-spec/21-app/ui-and-functionality-enhancement-instructions/01-architecture-spec.md
  coding_guidelines: 02-spec/02-coding-guidelines/
  master_plan: .ai-memory/plans/ui-and-functionality-enhancement-instructions.md
target_files:
  - src/components/instances/PromptTreeView/DetailTabs.tsx
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/TreePanel.tsx
  - src/components/instances/PromptTreeView/ProjectNode.tsx
  - src/components/instances/PromptTreeView/ConversationNode.tsx
  - src/components/instances/PromptTreeView/PromptInstructionView.tsx
  - src/components/instances/PromptTreeView/markdown.tsx
  - src/components/instances/PromptTreeView/DetailsModal.tsx
  - src/components/instances/PromptTreeView/usePromptTreeState.ts
status: pending
---

# Subtask 01: UI Layout Modernization, Button Border-Radius Normalization, Tree Node Right-Click Context Menus, and Image Copy Pipeline

## 1. Context & Motivation

In the `PromptTreeView` component suite, several ergonomic and visual defects hinder user productivity:
1. **DetailTabs Asymmetry**: The `[Input]` and `[Output]` tabs are anchored on the far left next to view mode buttons (`Preview`, `Raw`, `Edit`, `Details`), creating left-edge visual clutter while separating payload selection from word-count stats and `[Expand Full]`.
2. **Excessive Button Rounding**: Buttons and segmented action capsules throughout the prompt tree utilize `rounded-full`, creating a bubbly, uncoordinated mobile appearance rather than a sleek desktop developer tool.
3. **Tree Node Right-Click Dead Zones**: Right-clicking on tree nodes (projects, conversations, prompt turns) is either ignored or bound only to inner text spans, failing when users right-click whitespace or icons.
4. **Missing Image Preview & Copying**: Markdown images and attached media paths in `PromptInstructionView.tsx` lack thumbnail strips, click-to-preview lightbox modals, and contextual right-click copy actions.
5. **Ambiguous Copy Options**: The copy toolbar in `DetailHeader.tsx` lacks unambiguous choices between copying rich HTML with images versus clean sanitized plain text without images.
6. **Missing Runtime Telemetry in Details Modal**: The Details button was removed from the header, and `DetailsModal.tsx` does not show the running IDE name, instance name, executable path, or OS process PID.
7. **Lack of Enqueued Prompts Visibility**: When multiple prompts are dispatched to a conversation, users cannot see how many prompts are enqueued in the background queue from the tree or header.
8. **Ambiguity Between Session vs Turn Selection**: Clicking a whole conversation session versus clicking a single prompt turn renders nearly indistinguishable detail headers, confusing users about which scope is active.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/instances/PromptTreeView/DetailTabs.tsx` | `DetailTabs`, `DetailTabsProps` | 1. Move `[Input]` and `[Output]` tabs to the right-hand side adjacent to `[Expand Full]`.<br/>2. Replace `rounded-full` with `rounded-lg` wrapper and `rounded-md` buttons. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeader`, `DetailHeaderProps` | 1. Replace `rounded-full` capsules with `rounded-lg`.<br/>2. Implement dual copy buttons: "Copy with images" vs "Copy without images (as text)".<br/>3. Restore prominent Details button.<br/>4. Render enqueued prompts notification banner.<br/>5. Differentiate conversation session view vs single turn view in badges and title. |
| `src/components/instances/PromptTreeView/TreePanel.tsx` | `TreePanel`, `TreePanelProps` | 1. Normalize filter chips and buttons from `rounded-full` to `rounded-md`.<br/>2. Forward full-row right-click handlers down to project and conversation nodes. |
| `src/components/instances/PromptTreeView/ProjectNode.tsx` | `ProjectNode`, `ProjectNodeProps` | 1. Attach `onContextMenu` across entire project row container `div`.<br/>2. Normalize status pills and buttons to `rounded-md`. |
| `src/components/instances/PromptTreeView/ConversationNode.tsx` | `ConversationNode`, `ConversationNodeProps` | 1. Attach `onContextMenu` across whole conversation row `div`.<br/>2. Display enqueued count badge (`X QUEUED`).<br/>3. Normalize badges from `rounded-full` to `rounded-md`.<br/>4. Provide visual differentiation for session selection vs turn selection. |
| `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | `PromptInstructionView`, `PromptInstructionViewProps` | 1. Integrate image thumbnail gallery for attached media and markdown images.<br/>2. Enable lightbox click preview and right-click image menu.<br/>3. Normalize button rounding to `rounded-md`. |
| `src/components/instances/PromptTreeView/markdown.tsx` | `RichMarkdownRenderer`, `parseInlineMarkdown` | 1. Support interactive image wrappers with click to preview and right-click context menu.<br/>2. Convert local filesystem image URIs using Tauri `convertFileSrc` when available. |
| `src/components/instances/PromptTreeView/DetailsModal.tsx` | `DetailsModal`, `DetailsModalProps` | 1. Display Target IDE, Instance Name, Executable Path, and Process PID.<br/>2. Add 1-click copy buttons for executable path and PID.<br/>3. Normalize modal buttons to `rounded-md`. |
| `src/components/instances/PromptTreeView/usePromptTreeState.ts` | `usePromptTreeState`, `PromptTreeState` | 1. Add `selectedTurnNumber: number | null` state and setter.<br/>2. Add `isDetailsModalOpen: boolean` and setter. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Re-anchor Input/Output Tabs and Normalize Rounding in `DetailTabs.tsx`

1. In `src/components/instances/PromptTreeView/DetailTabs.tsx`:
   - Keep View Mode controls (`Preview`, `Raw`, `Edit`, `Details`) in the left-hand container.
   - Replace outer capsule `rounded-full` with `rounded-lg`:
     ```tsx
     <div className="inline-flex items-center rounded-lg bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
     ```
   - Update button corner radii: first button `rounded-l-md`, middle buttons `rounded-none`, last button `rounded-r-md`.
   - Move the `Input` and `Output` tabs to the right container `flex items-center gap-2 ml-auto shrink-0`:
     ```tsx
     {/* Right: Stats + Input/Output Tabs + Expand Full Toggle */}
     <div className="flex items-center gap-2 ml-auto shrink-0">
         {previewTab === 'instruction' && (
             <span className="text-xs font-semibold text-slate-500 dark:text-slate-400 font-mono">
                 {activeWordCount} words · {formatByteSize(activeByteCount)}
             </span>
         )}

         {setPreviewTab && (
             <div className="inline-flex items-center rounded-lg bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
                 <button
                     type="button"
                     onClick={() => setPreviewTab('instruction')}
                     className={cn(
                         "flex items-center gap-1.5 px-3 py-1 rounded-l-md text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                         previewTab === 'instruction'
                             ? "bg-blue-600 text-white shadow-2xs"
                             : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white"
                     )}
                     title="View Input Instruction"
                 >
                     <MessageSquare className="w-3.5 h-3.5" />
                     <span>Input</span>
                 </button>

                 <button
                     type="button"
                     onClick={() => setPreviewTab('results')}
                     className={cn(
                         "flex items-center gap-1.5 px-3 py-1 rounded-r-md text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                         previewTab === 'results'
                             ? "bg-blue-600 text-white shadow-2xs"
                             : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white"
                     )}
                     title="View Output Results"
                 >
                     {selectedConversation?.is_running ? (
                         <span className="relative flex h-2 w-2">
                             <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                             <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d]"></span>
                         </span>
                     ) : (
                         <Terminal className="w-3.5 h-3.5 text-cyan-500" />
                     )}
                     <span>Output</span>
                     {(selectedConversation?.execution_results || selectedConversation?.latest_response) && (
                         <span className="w-1.5 h-1.5 rounded-full bg-cyan-400" />
                     )}
                 </button>
             </div>
         )}

         {previewTab === 'instruction' && (isTruncated || activePromptText.trim().endsWith('...') || activePromptText.trim().endsWith('…') || showAllWords) && (
             <button
                 type="button"
                 onClick={() => setShowAllWords(!showAllWords)}
                 className="flex items-center gap-1 px-2.5 py-1 rounded-md text-[10px] font-semibold bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 border border-cyan-500/20 hover:bg-cyan-500/20 transition-all cursor-pointer"
                 title={showAllWords ? 'Collapse full prompt text' : 'Expand full prompt text'}
             >
                 {showAllWords ? (
                     <>
                         <ChevronUp className="w-3 h-3" />
                         <span>Collapse</span>
                     </>
                 ) : (
                     <>
                         <ChevronDown className="w-3 h-3" />
                         <span>Expand Full</span>
                     </>
                 )}
             </button>
         )}
     </div>
     ```

---

### 3.2 Step 2: Implement Dual Copy Buttons and Details Trigger in `DetailHeader.tsx`

1. In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - Replace the single ambiguous "Copy All" button with a segmented dual copy choice:
     - Button 1: "Copy with images" (executes `handleCopyAllWithRichImages`).
     - Button 2: "Copy without images" (executes `handleCopyCleanText`).
     - Chevron: opens flyout menu for export options (JSON, YAML, SQLite).
   - Normalized capsule styling: `rounded-lg` container with `rounded-l-md` and `rounded-none` segments.
   - Restore the Details button beside title or in toolbar:
     ```tsx
     <button
         type="button"
         onClick={() => setIsDetailsModalOpen?.(true)}
         className="flex items-center gap-1 px-2.5 py-1 rounded-md bg-slate-100 dark:bg-[#120a1f] border border-slate-200 dark:border-purple-500/30 text-slate-700 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-[#40285e] text-xs font-semibold transition-colors cursor-pointer"
         title="View instance environment, IDE, and process PID details"
     >
         <FileText className="w-3.5 h-3.5 text-blue-500" />
         <span>Details</span>
     </button>
     ```
   - Render the enqueued prompts notification banner if enqueued turns exist:
     ```tsx
     {enqueuedCount > 0 && (
         <div className="flex items-center gap-2 px-2.5 py-1 rounded-md bg-amber-500/10 border border-amber-500/30 text-amber-800 dark:text-amber-200 text-xs font-medium mt-1">
             <Clock className="w-3.5 h-3.5 text-amber-500 shrink-0" />
             <span>{enqueuedCount} prompt(s) enqueued in FIFO queue for this conversation.</span>
         </div>
     )}
     ```

---

### 3.3 Step 3: Container-Level Right-Click Context Menus in `ProjectNode.tsx` and `ConversationNode.tsx`

1. In `src/components/instances/PromptTreeView/ProjectNode.tsx`:
   - Ensure the outer project container `div` attaches `onContextMenu`:
     ```tsx
     onContextMenu={(e) => {
         e.preventDefault();
         e.stopPropagation();
         setProjectMenuPos({ x: e.clientX, y: e.clientY });
     }}
     ```
   - Render `ProjectContextMenu` through portal with proper coordinate boundary clamping.
2. In `src/components/instances/PromptTreeView/ConversationNode.tsx`:
   - Attach `onContextMenu` directly to the top-level row container:
     ```tsx
     <div
         tabIndex={0}
         onClick={handleRowClick}
         onDoubleClick={handleRowDoubleClick}
         onContextMenu={(e) => {
             e.preventDefault();
             e.stopPropagation();
             setContextMenuPos({ x: e.clientX, y: e.clientY });
         }}
         className={cn(
             "group relative flex items-center justify-between rounded-md px-2 py-1.5 text-xs cursor-pointer transition-colors outline-none",
             ...
         )}
     >
     ```
   - Render `ConversationContextMenu` for the active conversation or turn.
   - Calculate and display enqueued prompts count badge:
     ```tsx
     const enqueuedCount = conv.prompts?.filter((p) => p.is_queued || p.status === 'QUEUED').length || (conv.is_queued ? 1 : 0);
     ```
     Render amber badge if `enqueuedCount > 0`.

---

### 3.4 Step 4: Enrich `DetailsModal.tsx` with Target IDE, Executable Path, and Process PID

1. In `src/components/instances/PromptTreeView/DetailsModal.tsx`:
   - Extend `DetailsModalProps` to receive `instanceExeName?: string`, `instanceNameDisplay?: string`, `executablePath?: string`, and `instancePid?: number`.
   - Add a dedicated **Runtime Environment** card at the top:
     ```tsx
     <div className="space-y-2">
         <div className="text-[11px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
             Runtime Environment
         </div>
         <div className="bg-slate-50 dark:bg-[#090611] rounded-lg p-3 border border-slate-200/80 dark:border-[#40285e]/80 space-y-2">
             <div className="grid grid-cols-2 gap-2">
                 <div>
                     <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Target IDE</span>
                     <span className="font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-1.5 mt-0.5">
                         <Terminal className="w-3.5 h-3.5 text-purple-400" />
                         {instanceExeName || 'Antigravity IDE'}
                     </span>
                 </div>
                 <div>
                     <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Instance Name</span>
                     <span className="font-semibold text-slate-800 dark:text-slate-200 block mt-0.5">
                         {instanceNameDisplay || instanceId || 'default'}
                     </span>
                 </div>
             </div>
             <div className="pt-2 border-t border-slate-200/50 dark:border-[#40285e]/50">
                 <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Executable Path</span>
                 <div className="flex items-center justify-between gap-2 mt-0.5">
                     <span className="font-mono text-[11px] text-slate-700 dark:text-slate-300 break-all select-all">
                         {executablePath || instanceExeName || 'N/A'}
                     </span>
                     {executablePath && (
                         <button
                             type="button"
                             onClick={() => handleCopyField('exe_path', executablePath)}
                             className="px-2 py-0.5 rounded-md border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] text-[10px] font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] shrink-0 cursor-pointer"
                         >
                             {copiedField === 'exe_path' ? 'Copied!' : 'Copy'}
                         </button>
                     )}
                 </div>
             </div>
             <div className="grid grid-cols-2 gap-2 pt-2 border-t border-slate-200/50 dark:border-[#40285e]/50">
                 <div>
                     <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Process PID</span>
                     <span className="font-mono font-bold text-slate-800 dark:text-slate-200">
                         {instancePid ? `PID: ${instancePid}` : 'Not Running / Detached'}
                     </span>
                 </div>
                 <div>
                     <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Liveness Status</span>
                     <span className={cn('font-semibold text-xs', instancePid ? 'text-emerald-500' : 'text-slate-500')}>
                         {instancePid ? 'Active (Live)' : 'Idle'}
                     </span>
                 </div>
             </div>
         </div>
     </div>
     ```

---

### 3.5 Step 5: Implement Image Thumbnail Gallery and Lightbox in `PromptInstructionView.tsx`

1. In `src/components/instances/PromptTreeView/PromptInstructionView.tsx`:
   - Extract images from `activePromptText` and `selectedConversation.prompts` using `extractAssets()`.
   - Render a media thumbnail strip if images are present:
     ```tsx
     {extractedImages.length > 0 && (
         <div className="flex items-center gap-2 p-2 rounded-lg bg-slate-100/60 dark:bg-[#120a1f]/60 border border-slate-200 dark:border-[#40285e] overflow-x-auto">
             {extractedImages.map((img, idx) => (
                 <div
                     key={`thumb-${idx}`}
                     onClick={() => setPreviewImageSrc(img.src)}
                     onContextMenu={(e) => {
                         e.preventDefault();
                         e.stopPropagation();
                         setImageContextMenu({ x: e.clientX, y: e.clientY, src: img.src });
                     }}
                     className="relative group shrink-0 cursor-pointer"
                 >
                     <img
                         src={img.src}
                         alt={img.alt || `Attachment ${idx + 1}`}
                         className="w-14 h-14 object-cover rounded-md border border-slate-200 dark:border-purple-500/30 group-hover:border-purple-400 transition-all hover:scale-105"
                     />
                     <span className="absolute bottom-0 right-0 px-1 text-[8px] bg-black/70 text-white rounded-tl font-mono">
                         {idx + 1}
                     </span>
                 </div>
             ))}
         </div>
     )}
     ```
   - Connect `PromptImagePreviewModal` for full lightbox zoom/pan.
   - Connect `PromptImageContextMenu` for right-click copy and save operations.

---

### 3.6 Step 6: Visual Differentiation Between Conversation Session vs Prompt Turn

1. In `src/components/instances/PromptTreeView/usePromptTreeState.ts`:
   - Add `selectedTurnNumber: number | null` and `setSelectedTurnNumber`.
2. In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - If `selectedTurnNumber === null`:
     - Badge: `[Session C001 · {stepCount} Turns]` in cyan/blue font.
     - Breadcrumb: `Full Conversation Session Inspection`.
   - If `selectedTurnNumber !== null`:
     - Badge: `[Turn T{selectedTurnNumber} · C001]` in purple/emerald font.
     - Breadcrumb: `↳ Prompt Turn {selectedTurnNumber} of {totalTurns}` with navigation buttons (`< Prev Turn`, `Next Turn >`).
3. In `src/components/instances/PromptTreeView/ConversationNode.tsx`:
   - If `selectedTurnNumber === null` and `isSelected`: highlight the parent card with prominent blue/purple border.
   - If a turn is selected: keep parent card subtly styled while highlighting the active turn with `border-l-2 border-sky-500 bg-sky-500/15`.

---

## 4. Testing & Verification Checklist

- [ ] **Tab Placement**: Verify `[Input]` and `[Output]` tabs reside on the right-hand side beside `[Expand Full]`.
- [ ] **Border Radius**: Search all JSX classes in `PromptTreeView` to ensure `rounded-full` is eliminated from buttons and capsules, replaced by `rounded-md` and `rounded-lg`.
- [ ] **Right-Click**: Test right-clicking on empty spaces in project rows, conversation rows, and prompt turn rows. Confirm appropriate context menus appear without browser defaults.
- [ ] **Image Copy & Lightbox**: Test prompts containing images. Confirm thumbnails render, clicking opens lightbox preview, and right-clicking allows "Copy Image".
- [ ] **Dual Copy**: Test "Copy with images" (rich HTML + binary PNG) and "Copy without images" (pure markdown).
- [ ] **Details Modal**: Open Details modal and verify Target IDE, Instance Name, Executable Path, and Process PID are accurately rendered.
- [ ] **Enqueued Counter**: Enqueue multiple prompts and verify the amber badge appears in both `ConversationNode` and `DetailHeader`.
- [ ] **Session vs Turn View**: Switch between conversation card and sub-turn nodes; confirm title, badge, and styling clearly reflect the active scope.
