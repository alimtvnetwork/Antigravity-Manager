---
plan: 127-accounts-ui-supabase-sync-instance-compact-and-release
subtask: "02"
title: Instance Table Compacting, 4-per-Row Card Grid, and 5-6px Button Radius Standard
domain: frontend-react
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/127-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md#33-instancetable-compacting--horizontal-scrollbar-elimination
  screenshots:
    - assets/screenshots/106-02-instance-card-buttons-and-path.png
    - assets/screenshots/106-03-instance-header-rotate-next-best-button.png
    - assets/screenshots/106-04-instance-card-stack-compact-grid.png
target_files:
  - src/components/instances/InstanceTable.tsx
  - src/pages/Instances.tsx
status: pending
---

# 02 — Instance Table Compacting, 4-per-Row Card Grid, and 5-6px Button Radius Standard

## 1. Context & Motivation

In the Instances view of Antigravity Manager, user workflows are disrupted by multiple layout and ergonomics issues:
1. **Unwanted Horizontal Scrollbar in Instance Table**: On standard 1280px+ desktop displays, `InstanceTable.tsx` requires horizontal scrolling because separate columns for Profile Name, Bound Email, and long data paths take up too much width.
2. **Prompts Button Floating Outside Action Capsule**: The "Prompts" action is placed separately or wastes table space rather than being consolidated into the segmented action capsule.
3. **Card Mode Takes Excessive Screen Space**: In card view (`Instances.tsx`), cards render in 1–2 columns (`xl:grid-cols-2`), failing to utilize modern wide screens. The user explicitly requires 4 items per row (`xl:grid-cols-4`).
4. **Haphazard Action Buttons in Card Mode**: Card buttons wrap awkwardly in a single uneven row. They need a clean, structured 2-row layout with distinct functional groupings.
5. **Bulbous Pill Buttons**: Header and card buttons frequently use rounded pill styles (`rounded-full`, `rounded-2xl`). The user standard mandates strict 5–6px rounded corners (`rounded-[5px]`).
6. **"Rotate to Next Best" Header Button Flaws**: The button has disproportionate padding and lacks contextual feedback about which instance will actually be rotated upon click.

---

## 2. Target Files & Key Symbols

| Target File | Symbols / Locations | Scope of Modification |
|---|---|---|
| `src/components/instances/InstanceTable.tsx` | Table headers `<thead>` (~L110-120)<br>`formatShortPath` (~L45-52)<br>Merged Profile & Email cell (~L190-250)<br>Path cell (~L338-366)<br>Segmented Action Capsule (~L368-450) | Compact columns to eliminate horizontal scroll at 1280px+. Merge Profile & Email into single cell. Constrain path to `max-w-[140px]`. Embed Prompts button inside capsule. Modernize emerald to cyan/teal. |
| `src/pages/Instances.tsx` | "Rotate to Next Best" button (~L860-880)<br>Card Grid Container (~L1007-1012)<br>Card Action Buttons (~L1495-1660)<br>All interactive button classes | Enforce 4 cards per row (`xl:grid-cols-4`). Restructure card action buttons into 2 rows (5 primary, 6 secondary). Standardize all button radii to `rounded-[5px]`. Balance "Rotate to Next Best" padding to `px-3 py-1.5` and dynamic tooltip. |

---

## 3. Concrete Implementation Steps

### 3.1 InstanceTable Compacting (`src/components/instances/InstanceTable.tsx`)

1. **Eliminate Horizontal Scroll at 1280px+**:
   - Reduce cell padding across table rows and headers from `px-3 py-2.5` to `px-2 py-1.5`.
   - Tighten action capsule button padding to `px-1.5 py-1`.
   - Update table header widths:
     * `#`: `w-8 px-1 text-center`
     * `Profile & Account`: `min-w-[140px] max-w-[180px]`
     * `Model & Quota`: `min-w-[160px] max-w-[200px]`
     * `Status & PID`: `min-w-[90px] max-w-[120px]`
     * `File / Data Path`: `min-w-[120px] max-w-[160px]`
     * `Actions`: `min-w-[180px] text-right`
2. **Merged Profile Name & Bound Email Cell**:
   - In row render, combine Profile Name and Bound Email into a vertically stacked cell:
     * Top Line: Profile name (`font-semibold text-xs text-slate-800 dark:text-slate-100 truncate max-w-[130px]`) accompanied by `DEFAULT` or `ACTIVE` badge (`text-[9px] px-1 py-0.2 rounded-[5px]`).
     * Bottom Line: Status dot (`w-1.5 h-1.5 rounded-full`) + masked/revealed email (`text-[10px] font-mono text-slate-500 dark:text-slate-400 truncate max-w-[130px]`).
     * Quick toggle for mask/unmask on email click.
3. **Ending Path Truncation via `formatShortPath`**:
   - Enforce `formatShortPath(fullPath)` returning `...\<parent>\<leaf>`:
     ```tsx
     <div className="flex items-center justify-between gap-1 text-[11px] font-mono text-slate-500 dark:text-slate-400 max-w-[140px]">
         <div className="flex items-center gap-1 min-w-0 truncate" title={fullPath}>
             <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
             <span className="truncate">{shortPath}</span>
         </div>
         <button
             type="button"
             onClick={() => handleCopyPath(inst.config.id, fullPath)}
             className="p-1 rounded-[5px] text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 transition-colors"
             title={isCopied ? "Copied!" : "Copy full path"}
         >
             {isCopied ? <Check className="w-3 h-3 text-teal-500" /> : <Copy className="w-3 h-3" />}
         </button>
     </div>
     ```
4. **Segmented Action Capsule with Prompts Button**:
   - Consolidate all row actions into the segmented capsule:
     * Slot 1: `Launch` / `Stop`
     * Slot 2: `Switch Account` (`RotateCcw`)
     * Slot 3: `Fast Forward` (`Zap`)
     * Slot 4: `Prompts & Conversations` (`Layers`)
     * Slot 5: `Audit Trail` (`History`, if `onAudit` provided)
     * Slot 6: `Sync PID & Quota` (`RotateCw`, if `onSync` provided)
   - Ensure the outer capsule has `rounded-[5px]`, `p-0.5`, `divide-x divide-slate-200 dark:divide-[#15334d]`.
5. **Modernize Emerald Accents to Cyan/Teal**:
   - Replace `text-emerald-500` and `bg-emerald-500` with `text-cyan-500` / `text-teal-500` and `bg-teal-500` / `bg-cyan-400`.

---

### 3.2 Instances.tsx Card Mode & Grid Compacting (`src/pages/Instances.tsx`)

1. **Enforce 4 Cards per Row Grid**:
   - In `src/pages/Instances.tsx`, update the card container grid to:
     ```tsx
     <div className={cn(
         cardDensity === 'compact'
             ? "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-2"
             : "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3"
     )}>
     ```
   - At desktop resolutions (`>= 1280px` / `xl:`), render exactly 4 cards per row.
2. **Two-Row Card Action Buttons**:
   - Inside each instance card, structure the action toolbar into 2 distinct rows:
     * **Row 1 (Primary Lifecycle - 5 buttons)**:
       1. `Launch` / `Stop` (`Play` / `Square`) with `rounded-[5px]`
       2. `Switch Account` (`ArrowRightLeft`) with `rounded-[5px]`
       3. `Fast Forward` (`FastForward`) with `rounded-[5px]`
       4. `Audit Trail` (`History`) with `rounded-[5px]`
       5. `Sync PID & Quota` (`RotateCw`) with `rounded-[5px]`
     * **Row 2 (Utilities & Danger - 6 buttons)**:
       1. `Prompts` (`Layers`) with active task pulse indicator and `rounded-[5px]`
       2. `Settings & Sync` (`SlidersHorizontal`) with `rounded-[5px]`
       3. `Clone Profile` (`Copy`) with `rounded-[5px]`
       4. `Clone Binary / Executable` (`Cpu`) with `rounded-[5px]`
       5. `Wipe Credentials` (`RotateCcw`) with `rounded-[5px]`
       6. `Delete Profile` (`Trash2` / invisible placeholder for default instance) with `rounded-[5px]`
   - Each button formatted with `px-2 py-1 text-xs rounded-[5px]`.

### 3.3 Strict 5–6px Border Radius Standard
- Audit and standardize all button classes across `InstanceTable.tsx` and `Instances.tsx`:
  * Replace any `rounded-full`, `rounded-2xl`, or `rounded-xl` on interactive buttons with `rounded-[5px]` (or `rounded-md`).
  * Ensure segmented capsules, dropdowns, and search bars adhere to 5–6px or 8px max radii.

### 3.4 "Rotate to Next Best" Header Button Polish
- In `src/pages/Instances.tsx` (~L860-880):
  * **Padding**: Apply balanced `px-3 py-1.5 text-xs font-semibold rounded-[5px]`.
  * **Dynamic Hover Tooltip**:
    ```tsx
    const activeInstance = instances.find(i => i.config.id === (activeInstanceId || 'default'))
        || instances.find(i => i.config.is_default)
        || instances[0];
    const rotateTooltip = activeInstance
        ? `Rotates active instance (#${activeInstance.config.seq_num ?? 1} ${activeInstance.config.name}) to the highest health candidate`
        : 'Rotates active instance to the highest health candidate';
    ```
  * Ensure tooltip updates dynamically when active instance selection changes.

---

## 4. Constraints

- **Strict Frontend Boundary**: Do NOT touch any backend Rust files (`supabase_sync.rs`, `auto_switcher.rs`, `config.rs`).
- **Total Ban on Git Commands**: Never execute `git add`, `git commit`, `git checkout`, `git push`, or any other git commands.
- **Preserve Action Handlers**: Retain all handlers for `onLaunch`, `onStop`, `onSwitch`, `onFastForward`, `onAudit`, `onSync`, `onSettings`, `onClone`, `onDelete`, `onOpenPromptTree`, `onSetActive`, and `onSetDefault`.

---

## 5. Out of Scope

- QuotaProgressBar and AccountTable styling (covered in Subtask 01).
- Supabase repo-secrets auto-discovery and CLI integration (covered by Subagent 2).

---

## 6. Verification Steps

1. Run pre-flight build check:
   ```bash
   npm run build
   ```
2. Test Instance Table:
   - Verify table at 1280px viewport width: zero horizontal scrollbar (`overflow-x` not active).
   - Check that Profile Name and Bound Email are consolidated into a single cell.
   - Check that data path is truncated using `formatShortPath` with `max-w-[140px]`.
   - Check that Prompts button (`<Layers />`) is located inside the segmented capsule.
3. Test Instance Cards:
   - Verify card container has `xl:grid-cols-4`.
   - Inspect card buttons: Row 1 has 5 buttons, Row 2 has 6 buttons.
   - Inspect all button border radii: exactly 5–6px (`rounded-[5px]`).
4. Test "Rotate to Next Best":
   - Inspect padding: `px-3 py-1.5`.
   - Hover over button: tooltip displays active profile name and target rotation explanation.

---

## 7. Done When

- [ ] `InstanceTable.tsx` displays zero horizontal scroll at 1280px+ desktop widths.
- [ ] Profile Name and Bound Email are merged into a single compact cell.
- [ ] File/Data path is truncated to `...\<parent>\<leaf>` with `max-w-[140px]` constraint.
- [ ] Prompts button is integrated into the table's segmented action capsule.
- [ ] Emerald green accents in `InstanceTable.tsx` modernized to cyan/teal.
- [ ] `Instances.tsx` card view displays 4 cards per row (`xl:grid-cols-4`).
- [ ] Card action buttons structured in 2 clean rows (5 buttons in Row 1, 6 in Row 2).
- [ ] All action buttons strictly adhere to 5–6px border radius (`rounded-[5px]`).
- [ ] "Rotate to Next Best" header button uses `px-3 py-1.5 rounded-[5px]` with dynamic target hover tooltip.
- [ ] `npm run build` passes with zero errors.
