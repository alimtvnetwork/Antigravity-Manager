---
plan: instance-management-and-ui-ux-synchronization
subtask: "01"
title: List View Layout Enhancements, OS PID Clarity, and Switch Account Dialog Redesign
domain: frontend-react-tauri
depends_on: none
citations:
  architecture_spec: 02-spec/21-app/instance-management-and-ui-ux-synchronization/01-architecture-spec.md
target_files:
  - src/components/instances/table/InstanceRowCells.tsx
  - src/pages/instances/InstanceCard.tsx
  - src/pages/instances/SwitchAccountDialog.tsx
status: pending
---

# Subtask 01: List View Layout Enhancements, OS PID Clarity, and Switch Account Dialog Redesign

## 1. Context & Scope

This subtask executes the frontend UI/UX enhancements defined in Task-01 and Task-03 of the `instance-management-and-ui-ux-synchronization` milestone:
1. **List View Inline Paths (`IdeExeAndDataCell`)**: Transform the multi-line stacked presentation in `src/components/instances/table/InstanceRowCells.tsx` into a unified single-line horizontal layout (`{exeEnding} · {dataEnding}`) with an inline copy button.
2. **Deterministic OS Process ID (PID) Clarity**: Confirm that numeric process identifiers (e.g. `1632`) represent the native Operating System Process ID (`PID`) and explicitly format them as `PID: ${pid}` across `StatusCell` and `InstanceCard.tsx`.
3. **Switch Account Dialog Modernization**: Overhaul `src/pages/instances/SwitchAccountDialog.tsx` with Royal Violet dark-glass tokens, segmented pill capsules for quota metrics, integrated `QuotaProgressBar` components, and accessible keyboard escape dismissal.

---

## 2. Target Files & Symbols Breakdown

| Target File | Target Function / Symbol | Line Range (Approx) | Modification Action |
| :--- | :--- | :--- | :--- |
| `src/components/instances/table/InstanceRowCells.tsx` | `StatusCell` | Lines 255–275 | Replace `PID ${inst.pid}` with explicit `PID: ${inst.pid}` and descriptive tooltip. |
| `src/components/instances/table/InstanceRowCells.tsx` | `IdeExeAndDataCell` | Lines 283–326 | Replace 2-line `flex-col` with single-line `{exeEnding} · {dataEnding}` horizontal flex layout. |
| `src/pages/instances/InstanceCard.tsx` | `InstanceCard` (Status Section) | Lines 333–343 | Replace `Running (${inst.pid})` with explicit `Running · PID: ${inst.pid}` pill badge. |
| `src/pages/instances/SwitchAccountDialog.tsx` | `SwitchAccountDialog` | Lines 1–295 | Modernize with dark glass tokens, integrate `QuotaProgressBar`, segmented pill capsules, and `Escape` key listener. |

---

## 3. Concrete Modification Steps

### Step 1: List View Inline Path Formatting & PID Badge in `InstanceRowCells.tsx`

**File:** `src/components/instances/table/InstanceRowCells.tsx`

1. **Update `StatusCell` (Lines 263–267)**:
   Change:
   ```tsx
   ) : inst.is_running ? (
       <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-teal-50/90 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border border-teal-300/50 dark:border-teal-800/80 backdrop-blur-xs" title={inst.pid ? `Process id ${inst.pid}` : 'Running'}>
           <span className="w-1.5 h-1.5 rounded-full bg-teal-500 animate-pulse" />
           {inst.pid ? `PID ${inst.pid}` : 'Running'}
       </span>
   ```
   To:
   ```tsx
   ) : inst.is_running ? (
       <span 
           className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-teal-50/90 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border border-teal-300/50 dark:border-teal-800/80 backdrop-blur-xs font-mono"
           title={inst.pid ? `Operating System Process ID (PID): ${inst.pid}` : 'Running'}
       >
           <span className="w-1.5 h-1.5 rounded-full bg-teal-500 animate-pulse" />
           <span>{inst.pid ? `PID: ${inst.pid}` : 'Running'}</span>
       </span>
   ```

2. **Update `IdeExeAndDataCell` (Lines 289–326)**:
   Change the multi-line stacked column:
   ```tsx
   export function IdeExeAndDataCell({ inst, isCopied, onCopy }: IdeExeAndDataCellProps) {
       const fullPath = resolveDataDir(inst);
       const exeFull = inst.config.executable_path || '';
       const exeName = exeFull
           ? (exeFull.split(/[/\\]/).filter(Boolean).pop() || exeFull)
           : (inst.config.is_default || inst.config.id === 'default' ? 'Antigravity.exe' : 'Default IDE');
       const dataEnding = formatPathEnding(fullPath);

       return (
           <td
               className="px-2 py-1 whitespace-nowrap min-w-[160px] max-w-[220px]"
               title={`IDE: ${exeFull || exeName}\nData: ${fullPath}`}
           >
               <div className="flex items-center justify-between gap-1 text-[11px] font-mono">
                   <div className="flex flex-col min-w-0 leading-tight">
                       <div className="flex items-center gap-1 min-w-0 text-slate-700 dark:text-slate-200 font-semibold" title={exeFull || exeName}>
                           <Terminal className="w-3 h-3 text-slate-400 shrink-0" />
                           <span className="truncate">{exeName}</span>
                       </div>
                       <div className="flex items-center gap-1 min-w-0 text-[10px] text-slate-500 dark:text-slate-400" title={fullPath}>
                           <Folder className="w-3 h-3 text-slate-400 shrink-0" />
                           <span className="truncate">{dataEnding || '—'}</span>
                       </div>
                   </div>
                   <button ... />
               </div>
           </td>
       );
   }
   ```
   To single-line horizontal layout:
   ```tsx
   export function IdeExeAndDataCell({ inst, isCopied, onCopy }: IdeExeAndDataCellProps) {
       const fullPath = resolveDataDir(inst);
       const exeFull = inst.config.executable_path || '';
       const exeEnding = exeFull 
           ? formatPathEnding(exeFull)
           : (inst.config.is_default || inst.config.id === 'default' ? 'Antigravity.exe' : 'Default IDE');
       const dataEnding = formatPathEnding(fullPath);

       return (
           <td
               className="px-2 py-1 whitespace-nowrap min-w-[200px] max-w-[280px]"
               title={`IDE: ${exeFull || exeEnding}\nData: ${fullPath}`}
           >
               <div className="flex items-center justify-between gap-1 text-[11px] font-mono w-full">
                   <div className="flex items-center gap-1.5 min-w-0 flex-1 truncate">
                       <Terminal className="w-3 h-3 text-slate-400 shrink-0" />
                       <span className="truncate text-slate-700 dark:text-slate-200 font-medium" title={exeFull || exeEnding}>
                           {exeEnding}
                       </span>
                       <span className="text-slate-400 dark:text-slate-500 font-sans select-none">·</span>
                       <Folder className="w-3 h-3 text-slate-400 shrink-0" />
                       <span className="truncate text-slate-500 dark:text-slate-400" title={fullPath}>
                           {dataEnding || '—'}
                       </span>
                   </div>
                   <button
                       type="button"
                       onClick={onCopy}
                       data-ui-name="copy-path-btn"
                       data-ui-id={`copy-path-${inst.config.id}`}
                       className="p-1 rounded-[5px] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#40285e] transition-colors cursor-pointer shrink-0 ml-1"
                       title={isCopied ? 'Copied!' : 'Copy full data path'}
                   >
                       {isCopied ? <Check className="w-3 h-3 text-teal-500" /> : <Copy className="w-3 h-3" />}
                   </button>
               </div>
           </td>
       );
   }
   ```

---

### Step 2: Explicit PID Labeling in `InstanceCard.tsx`

**File:** `src/pages/instances/InstanceCard.tsx`

1. **Update Process Status Section (Lines 333–343)**:
   Change:
   ```tsx
   {inst.is_running ? (
       <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[10px] font-semibold bg-[#1af18d]/10 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 shadow-2xs">
           <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse" />
           Running {inst.pid ? `(${inst.pid})` : ''}
       </span>
   ) : (
   ```
   To explicit `PID:` notation:
   ```tsx
   {inst.is_running ? (
       <span 
           className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[10px] font-semibold bg-[#1af18d]/10 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 shadow-2xs font-mono"
           title={inst.pid ? `Operating System Process ID (PID): ${inst.pid}` : 'Running'}
       >
           <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse" />
           <span>Running</span>
           {inst.pid && (
               <>
                   <span className="text-emerald-500/50 dark:text-[#1af18d]/50 font-sans">·</span>
                   <span>PID: {inst.pid}</span>
               </>
           )}
       </span>
   ) : (
   ```

---

### Step 3: Modernize `SwitchAccountDialog.tsx`

**File:** `src/pages/instances/SwitchAccountDialog.tsx`

1. **Imports & Setup**:
   - Import `useEffect` from `'react'`.
   - Import `QuotaProgressBar` from `'../../components/accounts/QuotaProgressBar'`.
2. **Keyboard Accessibility (`Escape` key)**:
   - Add a `useEffect` inside `SwitchAccountDialog`:
     ```tsx
     useEffect(() => {
         const handleKeyDown = (e: KeyboardEvent) => {
             if (e.key === 'Escape' && !isAnySwitching) {
                 close();
             }
         };
         window.addEventListener('keydown', handleKeyDown);
         return () => window.removeEventListener('keydown', handleKeyDown);
     }, [isAnySwitching]);
     ```
3. **Dark Glass Container Tokens**:
   - Update container styling to adhere to Royal Violet design tokens:
     ```tsx
     <div
         className="backdrop-blur-md bg-white/95 dark:bg-[var(--ui-surface-1,#120d1d)]/95 border border-slate-200 dark:border-[var(--ui-border-subtle,#40285e)] shadow-2xl rounded-2xl p-6 w-full max-w-lg max-h-[85vh] flex flex-col"
         onClick={(e) => e.stopPropagation()}
     >
     ```
4. **Segmented Pill Capsules for Quota Metrics & `QuotaProgressBar` Integration**:
   - Replace the legacy badges and custom progress bar with:
     ```tsx
     {/* Segmented Pill Capsule for Quota Scores */}
     <div className="flex flex-wrap items-center gap-2 text-[11px] mt-2">
         <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[var(--ui-surface-2,#1f1433)] border border-slate-200/80 dark:border-[var(--ui-border-subtle,#40285e)] overflow-hidden shadow-2xs divide-x divide-slate-200 dark:divide-[#40285e]">
             <span className="px-2 py-0.5 text-slate-700 dark:text-slate-300 font-mono text-[10px]">
                 <span className="text-slate-400 uppercase font-semibold mr-1">4h:</span>
                 {formatQuotaPercent(fourH)}
             </span>
             <span className="px-2 py-0.5 text-slate-700 dark:text-slate-300 font-mono text-[10px]">
                 <span className="text-slate-400 uppercase font-semibold mr-1">Wk:</span>
                 {formatQuotaPercent(weekly)}
             </span>
             <span className="px-2 py-0.5 text-teal-800 dark:text-teal-300 font-mono font-bold text-[10px] bg-teal-50/50 dark:bg-teal-950/30">
                 <span className="text-teal-600/70 dark:text-teal-400/70 uppercase font-bold mr-1">Tot:</span>
                 {formatQuotaPercent(total)}
             </span>
         </div>

         {/* Standard QuotaProgressBar Component */}
         <div className="w-24 shrink-0">
             <QuotaProgressBar
                 percentage={barPct}
                 compact={true}
                 showCheckpoints={false}
                 className="w-full"
             />
         </div>
     </div>
     ```

---

## 4. Verification Commands & Acceptance Criteria

### Verification Commands
```bash
# 1. Type check and build frontend
npm run build

# 2. Run ESLint across modified frontend components
npm run lint
```

### Acceptance Criteria
- [x] **Inline Path Consistency**: In `InstanceTable.tsx`, `IdeExeAndDataCell` renders the executable end-segment and data directory end-segment on a single line separated by `·`. No 2-line column vertical expansion.
- [x] **Process ID (PID) Clarity**: Running instances in `StatusCell` render `PID: <number>`. `InstanceCard.tsx` renders `Running · PID: <number>`. No ambiguous raw integers without `PID:` labels.
- [x] **Switch Account Dialog**:
  - Royal Violet dark-glass surface tokens (`var(--ui-surface-1)` / `var(--ui-border-subtle)`).
  - Quotas displayed inside contiguous segmented capsules (`4h` / `Wk` / `Tot`).
  - Quota visual indicator powered by standardized `QuotaProgressBar` component.
  - Pressing `Escape` closes the modal immediately when idle.
