# Subtask 04: Button Radius Normalization, "Rotate to Next Best" Tooltip, and 4-Column Card Grid Architecture

- **Parent Task**: `134-accounts-ui-supabase-instance-compact`
- **Subtask ID**: `04-button-radius-rotate-tooltip-and-card-grid`
- **Target Files**:
  - `src/pages/Instances.tsx` (Card grid breakpoints, 2-row action toolbar, "Rotate to Next Best" button & tooltip, button radius)
  - `src/components/instances/InstanceTable.tsx` (Table button radii alignment)
  - `src/components/accounts/AccountTable.tsx` & `AccountRow.tsx` (Toolbar and action button radii alignment)
- **Status**: Ready for Implementation
- **Dependencies**: Subtask 03 (Table compaction)

---

## 1. Context & Objectives

A cohesive, professional UI requires strict design token discipline. Previously, the application suffered from:
1. **Inconsistent Button Radii**: A mixture of `rounded-full`, `rounded-2xl`, and `rounded-lg` buttons across pages, giving an inconsistent, toy-like appearance rather than a sleek VS Code developer environment.
2. **Ambiguous Rotation Action**: The prominent "Rotate to Next Best" button lacked context, leaving users unsure which instance was being rotated or why.
3. **Suboptimal Card View Layout**: The instance cards wrapped loosely at 2 or 3 columns on large screens with uneven, messy action button wrapping.

This subtask achieves:
1. **Strict 5–6px Button Radius Standard (`rounded-[5px]`)**: Enforcing uniform corner geometry across all buttons, segmented capsules, toolbar items, and modal actions.
2. **Dynamic "Rotate to Next Best" Candidate Tooltip**: Contextually inspecting the active or default instance and rendering a comprehensive tooltip explaining target and rotation criteria.
3. **4 Cards per Row Grid (`xl:grid-cols-4`)**: Standardizing the card container to render exactly 4 cards on desktop screens (`>= 1280px`).
4. **Structured 2-Row Card Action Toolbar**: Cleanly partitioning card actions into Row 1 (4 lifecycle slots) and Row 2 (5 secondary slots), eliminating awkward button drops.

---

## 2. Step-by-Step Implementation Instructions

### Step 1: Enforce Strict 5–6px Border Radius Standard (`rounded-[5px]`)
1. Audit and replace bulbous button classes across `src/pages/Instances.tsx`, `InstanceTable.tsx`, and `AccountTable.tsx`:
   - Replace any instance of `rounded-full`, `rounded-2xl`, or `rounded-xl` on interactive action buttons with `rounded-[5px]`.
   - Update DaisyUI buttons to include `rounded-[5px]` (e.g. `className="btn btn-primary btn-sm rounded-[5px] ..."`).
   - Ensure segmented capsule wrappers use `rounded-[5px]` with edge children using `rounded-l-[5px]` and `rounded-r-[5px]`.
2. Preserve `rounded-full` ONLY for circular notification badges, status ping dots, and progress bar fills where circular geometries are semantically appropriate.

### Step 2: Implement "Rotate to Next Best" Dynamic Tooltip & Styling
In `src/pages/Instances.tsx`:
1. Calculate the dynamic tooltip based on the currently active instance or default instance:
   ```typescript
   const activeInstance = instances.find(i => i.config.id === (activeInstanceId || 'default')) 
       || instances.find(i => i.config.is_default) 
       || instances[0];
   const inUseAccountIds = instances.map(i => i.config.bound_account_id).filter(Boolean) as string[];
   const rankedCandidates = rankSmartCandidates(accounts, inUseAccountIds, activeInstance?.config.bound_account_id);
   const nextBestCandidate = rankedCandidates[0]?.account || null;
   const rotateTooltip = `Target: ${activeInstance?.config.name || 'Current'} → Next Best: ${
       nextBestCandidate
           ? `${nextBestCandidate.email} (${nextBestCandidate.quota?.subscription_tier || 'PRO'} · 4H: ${nextBestCandidate.quota?.models?.[0]?.percentage ?? 100}%)`
           : 'No idle candidate available'
   }`;
   ```
2. Standardize the button container styling:
   ```tsx
   <button
       type="button"
       onClick={handleRotateNextBest}
       className="flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold rounded-[5px] bg-blue-600 hover:bg-blue-500 text-white shadow-xs cursor-pointer transition-colors active:scale-95 shrink-0"
       title={rotateTooltip}
   >
       <RotateCcw className="w-3.5 h-3.5" />
       <span>{t('instances.rotate_next_best', 'Rotate to Next Best')}</span>
       {nextBestCandidate && (
           <span className="px-1.5 py-0.5 rounded-[4px] bg-blue-500/30 text-[10px] font-mono font-normal max-w-[110px] truncate">
               {nextBestCandidate.email.split('@')[0]}
           </span>
       )}
   </button>
   ```

### Step 3: Standardize 4-Column Card Grid Breakpoints
In `src/pages/Instances.tsx`:
1. Locate the instance card rendering container.
2. Standardize the grid classes:
   ```tsx
   <div className={cn(
       cardDensity === 'compact'
           ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2"
           : "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3"
   )}>
       {filteredInstances.map((instance, index) => (
           <InstanceCard key={instance.config.id} ... />
       ))}
   </div>
   ```
3. At the standard desktop breakpoint `>= 1280px` (`xl:`), ensure the grid renders exactly 4 columns.

### Step 4: Partition Card Action Toolbar into Structured 2 Rows
Inside the instance card action section in `src/pages/Instances.tsx`:
1. Split the action buttons into two dedicated grid rows with uniform gap and strict `rounded-[5px]`:
   - **Row 1: Primary Lifecycle (4 slots)**:
     ```tsx
     <div className="grid grid-cols-4 gap-1 w-full">
         {/* Slot 1: Launch / Stop / Restart split button */}
         {inst.is_running ? (
             <div className="flex items-center rounded-[5px] border border-rose-200 dark:border-rose-900/50 bg-rose-50 dark:bg-rose-950/40 overflow-hidden divide-x divide-rose-200 dark:divide-rose-900/50">
                 <button onClick={() => handleStop(inst.config.id)} className="flex-1 p-1.5 text-rose-600 dark:text-rose-400 hover:bg-rose-100" title="Stop Instance">
                     <Square className="w-3.5 h-3.5 fill-current" />
                 </button>
                 <button onClick={() => handleRestart(inst.config.id)} className="flex-1 p-1.5 text-amber-600 dark:text-amber-400 hover:bg-amber-100" title="Restart Instance">
                     <RotateCcw className="w-3.5 h-3.5" />
                 </button>
             </div>
         ) : (
             <button onClick={() => handleLaunch(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] bg-teal-50 dark:bg-teal-950/40 text-teal-600 dark:text-teal-400 border border-teal-200 dark:border-teal-900/50 hover:bg-teal-100" title="Launch Instance">
                 <Play className="w-3.5 h-3.5 fill-current" />
             </button>
         )}
         {/* Slot 2: Switch Account */}
         <button onClick={() => setSwitchTargetInstance(inst)} className="flex items-center justify-center p-1.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-sky-600 dark:text-sky-400 border border-slate-300 dark:border-slate-700 hover:bg-slate-200" title="Switch Account">
             <ArrowRightLeft className="w-3.5 h-3.5" />
         </button>
         {/* Slot 3: Fast Forward */}
         <button onClick={() => handleFastForward(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-amber-600 dark:text-amber-400 border border-slate-300 dark:border-slate-700 hover:bg-slate-200" title="Fast Forward to Best Candidate">
             <FastForward className="w-3.5 h-3.5" />
         </button>
         {/* Slot 4: Sync PID & Quota */}
         <button onClick={() => handleSync(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-teal-600 dark:text-teal-400 border border-slate-300 dark:border-slate-700 hover:bg-slate-200" title="Sync PID and Quota">
             <Cpu className="w-3.5 h-3.5 text-teal-500" />
         </button>
     </div>
     ```
   - **Row 2: Secondary Utilities (5 slots)**:
     ```tsx
     <div className="grid grid-cols-5 gap-1 w-full relative">
         {/* Slot 1: Prompts */}
         <button onClick={() => setPromptTreeInstance({ id: inst.config.id, name: inst.config.name })} className="flex items-center justify-center p-1.5 rounded-[5px] border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700" title="Prompt Tree">
             <Layers className="w-3.5 h-3.5" />
         </button>
         {/* Slot 2: Settings */}
         <button onClick={() => { setSettingsModalTarget(inst); setIsSettingsModalOpen(true); }} className="flex items-center justify-center p-1.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 border border-slate-300 dark:border-slate-700 hover:bg-slate-200" title="Settings & Sync">
             <SlidersHorizontal className="w-3.5 h-3.5" />
         </button>
         {/* Slot 3: Clone Profile */}
         <button onClick={() => handleClone(inst.config.id, inst.config.name)} className="flex items-center justify-center p-1.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 border border-slate-300 dark:border-slate-700 hover:bg-slate-200" title="Clone Profile">
             <Copy className="w-3.5 h-3.5" />
         </button>
         {/* Slot 4: Audit Trail */}
         <button onClick={() => openAudit(inst.config.id, inst.config.name)} className="flex items-center justify-center p-1.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 border border-slate-300 dark:border-slate-700 hover:bg-slate-200" title="Audit Trail">
             <History className="w-3.5 h-3.5 text-amber-500" />
         </button>
         {/* Slot 5: More Actions Popover */}
         <button onClick={() => toggleCardMoreMenu(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 border border-slate-300 dark:border-slate-700 hover:bg-slate-200" title="More Actions">
             <MoreHorizontal className="w-3.5 h-3.5" />
         </button>
     </div>
     ```

---

## 3. Verification & Validation Steps

1. **Button Radius Audit**:
   - Inspect all buttons in Instances and Accounts pages.
   - Verify that all buttons use `rounded-[5px]` (or `rounded-md`) and zero `rounded-full` or `rounded-2xl` classes remain on interactive controls.
2. **"Rotate to Next Best" Tooltip Test**:
   - Hover over "Rotate to Next Best" in the Instances header.
   - Verify the tooltip text displays target profile name and next candidate email, tier, and 4H quota percentage.
3. **Card Grid Breakpoint Test**:
   - Resize window to `>= 1280px` width.
   - Confirm exactly 4 cards are rendered per row (`xl:grid-cols-4`).
4. **Card Action Row Test**:
   - Verify that card actions cleanly separate into Row 1 (4 buttons) and Row 2 (5 buttons) without wrapping.

---

## 4. Done When Checklist

- [ ] All interactive action buttons standardize on `rounded-[5px]`.
- [ ] Bulbous rounded pill buttons (`rounded-full`, `rounded-2xl`) removed from toolbars.
- [ ] "Rotate to Next Best" displays dynamic candidate tooltip with target instance and quota information.
- [ ] Instance cards display 4 columns at desktop resolution (`xl:grid-cols-4`).
- [ ] Card action toolbar partitions into 2 clean, structured rows.
- [ ] `npm run build` succeeds without errors.
