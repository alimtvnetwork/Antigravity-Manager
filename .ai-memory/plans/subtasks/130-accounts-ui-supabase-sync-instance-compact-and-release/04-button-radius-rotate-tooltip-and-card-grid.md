# Subtask 04: Button Radius Normalization, "Rotate to Next Best" Tooltip, and 4-Column Card Grid Architecture

- **Parent Task**: `130-accounts-ui-supabase-sync-instance-compact-and-release`
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
4. **Structured 2-Row Card Action Toolbar**: Cleanly partitioning card actions into Row 1 (5 primary lifecycle buttons) and Row 2 (6 utility/danger buttons), eliminating awkward button drops.

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
   const rotateTooltip = useMemo(() => {
       const targetInst = instances.find(i => i.config.id === activeInstanceId)
           || instances.find(i => i.config.is_default || i.config.id === 'default')
           || instances[0];
       const name = targetInst?.config.name || 'Default';
       const seq = targetInst?.config.seq_num || 1;
       const email = targetInst?.config.bound_email ? ` (${targetInst.config.bound_email})` : '';
       return t(
           'instances.rotate_next_best_tooltip',
           `Rotate active instance (#${seq} ${name}${email}) to the highest health candidate account based on available quota, cooldown window, and distributed lease availability.`
       );
   }, [instances, activeInstanceId, t]);
   ```
2. Standardize the button container styling:
   ```tsx
   <button
       type="button"
       disabled={isRotatingAccount || isActionPending}
       onClick={handleRotateNextBest}
       className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-[5px] bg-blue-600 hover:bg-blue-500 text-white shadow-xs cursor-pointer transition-colors active:scale-95 shrink-0 disabled:opacity-50"
       title={rotateTooltip}
   >
       <RotateCcw className={cn("w-3.5 h-3.5", isRotatingAccount && "animate-spin")} />
       <span>{t('instances.rotate_next_best', 'Rotate to Next Best')}</span>
   </button>
   ```

### Step 3: Standardize 4-Column Card Grid Breakpoints
In `src/pages/Instances.tsx`:
1. Locate the instance card rendering container.
2. Standardize the grid classes:
   ```tsx
   <div className={cn(
       cardDensity === 'compact'
           ? "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-2"
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
1. Split the action buttons into two dedicated flex rows with uniform gap and strict `rounded-[5px]`:
   - **Row 1: Primary Lifecycle (5 buttons)**:
     ```tsx
     <div className="grid grid-cols-5 gap-1 pt-2 border-t border-slate-100 dark:border-slate-800">
         {/* 1. Launch / Stop Button */}
         <button onClick={() => toggleLaunch(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             {inst.is_running ? <Square className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5" />}
         </button>
         {/* 2. Switch Account Button */}
         <button onClick={() => openSwitchModal(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <ArrowRightLeft className="w-3.5 h-3.5" />
         </button>
         {/* 3. Fast Forward Button */}
         <button onClick={() => fastForward(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <FastForward className="w-3.5 h-3.5" />
         </button>
         {/* 4. Audit Trail Button */}
         <button onClick={() => openAuditTrail(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <History className="w-3.5 h-3.5" />
         </button>
         {/* 5. Sync PID & Quota Button */}
         <button onClick={() => syncInstance(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <RotateCw className="w-3.5 h-3.5" />
         </button>
     </div>
     ```
   - **Row 2: Utilities, Prompts & Danger (6 buttons)**:
     ```tsx
     <div className="grid grid-cols-6 gap-1 pt-1">
         {/* 1. Prompts Button */}
         <button onClick={() => openPromptTree(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <Layers className="w-3.5 h-3.5" />
         </button>
         {/* 2. Settings & Sync Button */}
         <button onClick={() => openSettings(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <SlidersHorizontal className="w-3.5 h-3.5" />
         </button>
         {/* 3. Clone Profile Button */}
         <button onClick={() => cloneProfile(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <Copy className="w-3.5 h-3.5" />
         </button>
         {/* 4. Clone Executable Button */}
         <button onClick={() => cloneExecutable(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <Cpu className="w-3.5 h-3.5" />
         </button>
         {/* 5. Wipe Credentials Button */}
         <button onClick={() => wipeCredentials(inst.config.id)} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <RotateCcw className="w-3.5 h-3.5" />
         </button>
         {/* 6. Delete Profile Button (Hidden/Disabled for Default) */}
         <button onClick={() => deleteProfile(inst.config.id)} disabled={isDefault} className="flex items-center justify-center p-1.5 rounded-[5px] ...">
             <Trash2 className="w-3.5 h-3.5" />
         </button>
     </div>
     ```

---

## 3. Verification & Validation Steps

1. **Card Grid Verification**:
   - Open Instances page in Card View on a monitor >= 1280px wide.
   - Confirm that cards form a clean 4-column layout (`xl:grid-cols-4`).
2. **Two-Row Action Layout Verification**:
   - Inspect multiple cards across different zoom levels and translations.
   - Verify that Row 1 strictly renders 5 primary buttons and Row 2 renders 6 utility buttons with zero irregular line drops.
3. **Button Corner Radius Verification**:
   - Inspect CSS of buttons in Cards, Table, and Toolbars.
   - Verify `border-radius: 5px` (`rounded-[5px]`) is universally applied.
4. **"Rotate to Next Best" Tooltip Test**:
   - Hover over "Rotate to Next Best".
   - Confirm tooltip dynamically indicates: `Rotate active instance (#<seq> <name>) ...`.
   - Set another instance active and re-hover; confirm tooltip updates to reflect the new active instance name.

---

## 4. Acceptance Criteria

- [ ] All action buttons across Cards, Tables, and Toolbars strictly use `rounded-[5px]`.
- [ ] Bulbous `rounded-full` buttons are completely removed from interactive actions.
- [ ] Card mode standardizes on `xl:grid-cols-4` for desktop viewports.
- [ ] Card actions are cleanly organized into Row 1 (5 buttons) and Row 2 (6 buttons).
- [ ] "Rotate to Next Best" button renders with `px-3 py-1.5 text-xs font-semibold rounded-[5px]` and displays a dynamic candidate target tooltip.
- [ ] `npm run build` succeeds cleanly.
