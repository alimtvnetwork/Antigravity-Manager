# Subtask Plan: 01-header-and-tree-compactness

- **Subtask ID:** `01-header-and-tree-compactness.md`
- **Parent Task:** `225-prompt-tree-view-issues`
- **Master Audit Ledger:** `02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md`
- **Parent Spec:** `02-spec/21-app/225-prompt-tree-view-issues/01-architecture-spec.md`
- **Component Spec:** `02-spec/21-app/225-prompt-tree-view-issues/02-component-spec.md`
- **Status:** `READY_FOR_EXECUTION`
- **Estimated Lines Changed:** `~180-240 lines` across 6 files
- **Owner:** Spec Author 01 / Worker 01

---

## 1. Subtask Objective

Execute the visual compaction and ergonomic overhaul of the Prompt Tree View navigation system. Specifically:
1. Minimizing the top modal header container padding and vertical footprint to reclaim screen space.
2. Converting secondary text actions ("Import / Export", "Hide tree view") into sleek Lucide icon buttons (`Upload`, `Download`, `PanelLeftClose`) with rich hover tooltips.
3. Relabeling the yellow action button next to Refresh explicitly as "Reset" and implementing high-contrast styling (`text-zinc-950 bg-amber-400 hover:bg-amber-300 font-semibold shadow-sm active:translate-y-px`).
4. Tightening tree node vertical padding and line heights across conversation items and prompt turn subtrees to increase visible node density.
5. Eliminating hover jitter and border displacement on conversation rows (zero-shift hover styling).
6. Consolidating toolbar and header controls into contiguous segmented dark-glass pill capsules (`rounded-full`, shared border, `divide-x`).
7. Enforcing a strict 30-second synchronization caching guard in `usePromptTreeData.ts`, eliminating the legacy 2-second polling accelerator and hover-triggered re-fetches.

---

## 2. Target Files & Touch Points

| File Relative Path | Responsibility & Touch Point | Est. Diff Lines |
| :--- | :--- | :---: |
| `src/components/instances/PromptTreeView/ModalHeader.tsx` | Reduce padding to `py-1.5 px-3`, embed high-contrast Reset button (`bg-amber-400 text-zinc-950`), iconize tree toggle (`PanelLeftClose`), wrap controls in segmented capsule. | `+35 / -40` |
| `src/components/instances/PromptTreeView/ImportExportMenu.tsx` | Convert text trigger to compact icon button with tooltip; refine dropdown menu items with high-contrast text and clean borders. | `+20 / -25` |
| `src/components/instances/PromptTreeView/TreePanel.tsx` | Tighten search and filter container padding (`py-1 px-2`), compact 4-tier origin capsule, convert tree collapse button to icon, refine project node container. | `+30 / -35` |
| `src/components/instances/PromptTreeView/ConversationNode.tsx` | Reduce vertical padding to `py-0.5 px-1.5`, enforce stable border styling (`border border-transparent`), eliminate layout shifts on hover, tighten prompt turn subtree spacing. | `+25 / -30` |
| `src/components/instances/PromptTreeView/ProjectNode.tsx` | Compact project accordion header (`py-1 px-2`), tighten spacing between project header and child conversation list. | `+20 / -20` |
| `src/components/instances/PromptTreeView/usePromptTreeData.ts` | Remove legacy 2-second dynamic polling acceleration; enforce strict adherence to configured `syncInterval` (default 30s); ensure zero IPC calls on hover. | `+15 / -25` |

---

## 3. Step-by-Step Implementation Instructions

### Step 3.1: Modal Header Container & Title Compaction (`ModalHeader.tsx`)
1. In `src/components/instances/PromptTreeView/ModalHeader.tsx`:
   - Replace container outer classes with compact metrics:
     ```tsx
     className="relative z-40 flex items-center justify-between border-b border-slate-200/80 dark:border-purple-900/40 px-3 py-1.5 bg-slate-50/80 dark:bg-[#0c0817]/95 backdrop-blur-xl shrink-0"
     ```
   - Retain compact icon badge (`h-6 w-6 rounded-[5px]`).
   - Title text: `text-xs font-bold text-slate-900 dark:text-white`.
   - Badges: `v{appVersion}` (`text-[10px] font-mono px-1.5 py-0.5`), sequence badge `#{sequenceNumber}` (`text-[10px] font-bold px-1.5 py-0.5`).
   - Instance name: `max-w-[140px] truncate text-xs font-semibold`.

### Step 3.2: Icon Button Conversion for Import / Export (`ImportExportMenu.tsx`)
1. In `src/components/instances/PromptTreeView/ImportExportMenu.tsx`:
   - Replace any lingering textual labels on the trigger button.
   - Use Lucide `Database` icon with a tiny indicator or split into `Upload` / `Download` icons inside the contiguous capsule.
   - Set accessible attributes:
     ```tsx
     title="Import / Export tree transcripts (JSON & SQLite)"
     aria-label="Import / Export"
     className="flex items-center justify-center w-7 h-7 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 rounded-l-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
     ```
   - Ensure the dropdown popup (`createPortal`) uses dark-glass styling (`bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-xl p-1.5 rounded-xl`).

### Step 3.3: High-Contrast Reset & Refresh Buttons (`ModalHeader.tsx`)
1. In `src/components/instances/PromptTreeView/ModalHeader.tsx`:
   - Update Reset button styling to ensure maximum contrast and 3D tactile feedback:
     ```tsx
     <button
         type="button"
         onClick={onResetTree}
         disabled={isLoading || isResetting}
         className="flex items-center justify-center gap-1 h-7 px-2.5 text-[11px] font-semibold text-zinc-950 bg-amber-400 hover:bg-amber-300 shadow-[0_2px_0_0_#b45309] active:translate-y-px active:shadow-none transition-all duration-150 ease-out cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
         title="Reset tree from disk transcripts (wipes SQLite cache and rescans)"
         aria-label="Reset"
     >
         <RotateCcw className={cn("w-3.5 h-3.5 text-zinc-950", isResetting && "animate-spin")} />
         <span>Reset</span>
     </button>
     ```
   - Update Refresh button styling:
     ```tsx
     <button
         type="button"
         onClick={onRefreshAll}
         disabled={isLoading}
         className="flex items-center justify-center gap-1 h-7 px-2.5 text-[11px] font-semibold text-white bg-sky-600 hover:bg-sky-500 shadow-[0_2px_0_0_#0284c7] active:translate-y-px active:shadow-none transition-all duration-150 ease-out cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
         title="Refresh the tree now. Automatic refresh follows the Sync interval only."
         aria-label="Refresh"
     >
         <RefreshCw className={cn("w-3.5 h-3.5 text-white", (isLoading || isAutoSyncing || isRefreshing) && "animate-spin")} />
         <span>Refresh</span>
     </button>
     ```

### Step 3.4: Hide Tree View Icon Conversion (`ModalHeader.tsx` & `TreePanel.tsx`)
1. In `src/components/instances/PromptTreeView/ModalHeader.tsx`:
   - Render the tree toggle within the segmented capsule as an icon button:
     ```tsx
     {onToggleTree && (
         <button
             type="button"
             onClick={onToggleTree}
             className="flex items-center justify-center w-7 h-7 text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-colors cursor-pointer"
             title={isTreeCollapsed ? "Show tree panel" : "Hide tree panel"}
             aria-label={isTreeCollapsed ? "Show tree panel" : "Hide tree panel"}
         >
             {isTreeCollapsed ? (
                 <PanelLeftOpen className="w-3.5 h-3.5 text-sky-400" />
             ) : (
                 <PanelLeftClose className="w-3.5 h-3.5 text-slate-400" />
             )}
         </button>
     )}
     ```
2. In `src/components/instances/PromptTreeView/TreePanel.tsx`:
   - Keep the collapse icon button in the project scope bar compact (`p-1 rounded-md text-slate-400 hover:text-slate-200`).

### Step 3.5: Tree Panel Spacing & Layout Compaction (`TreePanel.tsx`)
1. In `src/components/instances/PromptTreeView/TreePanel.tsx`:
   - Reduce filter section outer padding:
     ```tsx
     <div className="relative z-10 bg-slate-50/95 dark:bg-[#0c0817]/95 backdrop-blur-md px-2 py-1 border-b border-slate-200 dark:border-[#40285e] shrink-0 space-y-1">
     ```
   - Project scope selector row: compact `px-2 py-1 rounded-lg border border-slate-200 dark:border-[#40285e]`.
   - Ensure the inline `<select>` element uses borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none text-xs font-medium`).
   - Category filter pill bar: `rounded-lg border border-slate-200 dark:border-[#40285e] p-0.5 divide-x`.
   - Status filter pill bar: `rounded-lg border border-slate-200 dark:border-[#40285e] p-0.5 divide-x`.
   - Tree items scroll container:
     ```tsx
     <div className="flex-1 overflow-y-auto p-1 space-y-0.5">
     ```

### Step 3.6: Conversation Node Spacing & Zero-Jitter Hover Protection (`ConversationNode.tsx`)
1. In `src/components/instances/PromptTreeView/ConversationNode.tsx`:
   - Standardize base container styling to have a fixed, transparent border so hover state NEVER shifts layout:
     ```tsx
     className={cn(
         'group flex items-center justify-between rounded-md px-1.5 py-0.5 text-xs cursor-pointer transition-colors duration-150 relative outline-none focus:outline-none border border-transparent',
         isSelected && selectedTurnNumber === null
             ? 'bg-gradient-to-r from-violet-600/30 to-purple-600/20 text-white font-medium border-violet-500/30'
             : isSelected
               ? 'bg-purple-500/15 text-purple-300 font-medium border-purple-500/20'
               : 'text-slate-600 dark:text-zinc-300 hover:bg-slate-100/80 dark:hover:bg-[#181126]/70'
     )}
     ```
   - Notice: No green top/side border on hover. Hover strictly alters background tint (`hover:bg-slate-100/80 dark:hover:bg-[#181126]/70`).
   - Prompt turn rows:
     ```tsx
     className={cn(
         "group/turn flex items-center justify-between rounded-md px-1.5 py-0.5 text-xs cursor-pointer transition-colors outline-none border-l-2",
         isTurnActive
             ? "bg-sky-500/10 border-sky-500 text-sky-400 font-medium"
             : "border-transparent hover:bg-slate-100 dark:hover:bg-[#1a1228]"
     )}
     ```
   - Indented subtree container: reduce margin and vertical spacing:
     ```tsx
     <div className="border-l-2 border-slate-300/60 dark:border-purple-900/50 ml-4 pl-2 space-y-0.5 mt-0.5">
     ```

### Step 3.7: Strict 30-Second Caching Guard (`usePromptTreeData.ts`)
1. In `src/components/instances/PromptTreeView/usePromptTreeData.ts`:
   - Audit the timer effect. Remove any condition that drops `intervalMs` below `syncInterval` when prompts are running:
     ```tsx
     useEffect(() => {
         if (!isOpen || syncInterval === 'off') return;

         let intervalMs = 30000;
         if (syncInterval === '15s') intervalMs = 15000;
         else if (syncInterval === '30s') intervalMs = 30000;
         else if (syncInterval === '1m') intervalMs = 60000;
         else if (syncInterval === '2m') intervalMs = 120000;

         const timer = setInterval(() => {
             loadTree(false, false);
         }, intervalMs);

         return () => clearInterval(timer);
     }, [isOpen, syncInterval, instanceId, loadTree]);
     ```
   - Verify that no `onMouseEnter`, `onMouseOver`, or hover callbacks invoke `loadTree`, `invalidatePromptTreeCache`, or IPC commands.

---

## 4. Acceptance Criteria & Verification Checklist

- [ ] **AC-01: Header Height Reduction:** Header container measures `< 42px` in total vertical height with `py-1.5 px-3` padding.
- [ ] **AC-02: Icon Buttons:** "Import / Export" is an icon trigger with a descriptive tooltip; "Hide tree view" is rendered as `PanelLeftClose` / `PanelLeftOpen` with a tooltip.
- [ ] **AC-03: High-Contrast Reset Button:** The button next to Refresh is explicitly labeled "Reset" with text `text-zinc-950` on background `bg-amber-400` / `hover:bg-amber-300` and `shadow-[0_2px_0_0_#b45309]`.
- [ ] **AC-04: Blue Refresh Button:** The Refresh button is labeled "Refresh" with text `text-white` on `bg-sky-600` / `hover:bg-sky-500`.
- [ ] **AC-05: Contiguous Segmented Capsule:** All top-right header controls reside in a single unbroken pill capsule with `rounded-full` and `divide-x`.
- [ ] **AC-06: Compact Tree Spacing:** Conversation rows use `py-0.5 px-1.5`; prompt turn rows use `py-0.5 px-1.5` and `space-y-0.5`.
- [ ] **AC-07: Zero Hover Jitter:** Hovering over conversation or turn rows produces zero pixel shifts or green line artifacts.
- [ ] **AC-08: 30s Polling Rate:** The tree polling strictly obeys the configured interval (default 30 seconds); running prompts do not accelerate the timer.
- [ ] **AC-09: Code Cleanliness:** `npm run build` passes with zero TypeScript diagnostics.
