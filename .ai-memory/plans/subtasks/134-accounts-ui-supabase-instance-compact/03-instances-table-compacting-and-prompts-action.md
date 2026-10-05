# Subtask 03: Instances Table Column Compacting, Path Truncation, and Prompts Action Integration

- **Parent Task**: `134-accounts-ui-supabase-instance-compact`
- **Subtask ID**: `03-instances-table-compacting-and-prompts-action`
- **Target Files**:
  - `src/components/instances/InstanceTable.tsx` (Table column compaction, merged Profile/Email cell, path truncation, Prompts button integration)
  - `src/pages/Instances.tsx` (Instances page table container integration, action handler wiring)
- **Status**: Ready for Implementation
- **Dependencies**: Subtasks 01 & 02 (Accounts styling and backend sync)

---

## 1. Context & Objectives

In standard 1280px and 1366px desktop display resolutions, the Instances list/table view previously forced a horizontal scrollbar. The rightmost Actions capsule was pushed off-screen, forcing users to scroll horizontally to perform basic instance lifecycle operations (Launch, Stop, Switch, Rotate). Additionally, the directory path column consumed excessive horizontal space with raw absolute paths, and the Prompts button was separated from the main action capsule.

This subtask achieves:
1. **Zero Horizontal Scrollbar**: Strict compaction of all table columns and padding, guaranteeing `scrollWidth <= clientWidth` on viewports `>= 1280px`.
2. **Consolidated Profile & Account Cell**: Merging Profile Name and Bound Email into a single stacked cell with tier badges, mask/unmask toggling, and status indicators.
3. **Smart Path Truncation (`formatShortPath`)**: Truncating long Windows/POSIX paths to `...\<parent>\<leaf>` (`max-w-[140px]`) with click-to-copy feedback and full-path hover tooltips.
4. **Integrated Capsule Prompts Button**: Merging the "Prompts & History" button (`Layers` icon) directly inside the segmented action capsule and More Dropdown portal with compact responsive padding.
5. **Cyan/Teal Accent Modernization**: Replacing emerald green accents (spinners and pulse dots) with cohesive VS Code cyan/teal design tokens.

---

## 2. Step-by-Step Implementation Instructions

### Step 1: Consolidate Profile Name & Bound Email Column
In `src/components/instances/InstanceTable.tsx`:
1. In `<thead>`, merge the previously separate or oversized headers into a clean `Profile & Account` column header:
   ```tsx
   <th className="px-2 py-1.5 min-w-[140px] max-w-[180px]">Profile & Account</th>
   ```
2. In `<tbody>`, replace individual cells with a unified stacked cell:
   - **Top Row**: Profile name (`font-semibold text-slate-800 dark:text-slate-100 text-xs truncate max-w-[130px]`) accompanied by `DEFAULT` or `ACTIVE` badge (`rounded-[5px] text-[9px] font-bold`) and subscription tier pill (`ULTRA` / `PRO`).
   - **Bottom Row**: Account status indicator dot (teal for active, rose for disabled, amber for proxy disabled) + `Mail` icon (`w-3 h-3 text-slate-400`) + masked or revealed email (`text-xs font-mono truncate max-w-[170px] cursor-pointer hover:underline`).

### Step 2: Implement Path Truncation (`formatShortPath`) & 1-Click Copy
In `src/components/instances/InstanceTable.tsx`:
1. Ensure the `formatShortPath` helper extracts only the last two path segments:
   ```typescript
   export function formatShortPath(fullPath: string): string {
       if (!fullPath) return '';
       const isWindows = fullPath.includes('\\') || /^[a-zA-Z]:/.test(fullPath);
       const sep = isWindows ? '\\' : '/';
       const parts = fullPath.split(/[\\/]/).filter(Boolean);
       if (parts.length <= 2) return fullPath;
       return `...${sep}${parts.slice(-2).join(sep)}`;
   }
   ```
2. Render the `File / Data Path` cell constrained to `min-w-[120px] max-w-[140px]`:
   - Enforce `truncate` on the short path text.
   - Attach the full absolute path to the wrapper `title` attribute for native browser tooltip rendering.
   - Include a compact copy button triggering `handleCopyPath(inst.config.id, fullPath)`:
     - Default: `<Copy className="w-3 h-3" />`
     - Copied (1.5s active): `<Check className="w-3 h-3 text-teal-500" />` and tooltip `"Copied!"`.

### Step 3: Compact Column Padding & Dimensions
Standardize table column widths and cell padding across `InstanceTable.tsx`:
- Table wrapper: `w-full overflow-hidden rounded-xl border border-slate-200/80 dark:border-slate-800/80 bg-white dark:bg-[#0c2438] shadow-xs`.
- Cell vertical/horizontal padding: reduce from `px-3 py-2.5` to `px-2 py-1.5`.
- Column dimensions:
  - Column 1 (`#`): `w-8 px-1 py-1.5 text-center`
  - Column 2 (`Profile & Account`): `min-w-[140px] max-w-[180px] px-2 py-1.5`
  - Column 3 (`Model & Weekly Quota`): `min-w-[160px] max-w-[200px] px-2 py-1.5`
  - Column 4 (`Status & PID`): `min-w-[90px] max-w-[120px] px-2 py-1.5`
  - Column 5 (`File / Data Path`): `min-w-[120px] max-w-[140px] px-2 py-1.5`
  - Column 6 (`Actions`): `min-w-[130px] px-2 py-1.5 text-right`

### Step 4: Integrate Prompts Action Button into Segmented Capsule & More Dropdown
In `src/components/instances/InstanceTable.tsx`:
1. In the primary segmented capsule:
   - Outer container: `inline-flex items-center rounded-[5px] overflow-hidden bg-slate-100 dark:bg-[#071a27] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs`.
   - Primary 1: Launch / Stop / Restart split button (`px-2 py-1`).
   - Primary 2: Switch Account (`px-2 py-1 text-sky-600 dark:text-sky-400`).
   - Primary 3: Fast Forward (`px-2 py-1 text-amber-600 dark:text-amber-400`).
   - Primary 4: More Dropdown (`px-2 py-1 text-slate-600 dark:text-slate-300 rounded-r-[5px]`).
2. In the More Dropdown Portal:
   - Slot 1: `Prompts & History` with `Layers` icon (`text-cyan-600 dark:text-cyan-400`) calling `onOpenPromptTree(inst.config.id)`.
   - Slot 2: `Audit Trail` with `History` icon (`text-amber-500`).
   - Slot 3: `Sync PID & Quota` with `Cpu` icon (`text-teal-500`).
   - Slot 4: `Restart Instance` (if running).
   - Slot 5: `Settings & Sync` with `SlidersHorizontal` icon.
   - Slot 6: `Clone Profile` with `Copy` icon.
   - Slot 7: `Delete Profile` with `Trash2` icon.

### Step 5: Modernize Accents to Cyan/Teal
1. Replace all `text-emerald-500` launch spinner classes with `text-teal-500` or `text-cyan-400`.
2. Update active instance running badge to `bg-teal-50 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border border-teal-300/50 dark:border-teal-800/80`.
3. Set running pulse dot to `bg-teal-500 animate-pulse`.

---

## 3. Verification & Validation Steps

1. **Viewport Resolution Test**:
   - Open Antigravity Manager at 1280px browser/window width.
   - Navigate to Instances page in List/Table view.
   - Verify that no horizontal scrollbar appears on the table container (`scrollWidth <= clientWidth`).
   - Verify that all columns, including Actions, are visible simultaneously.
2. **Path Truncation & Copy Test**:
   - Locate an instance with a long data path (`AppData/Roaming/...`).
   - Confirm short path displays `...\<parent>\<leaf>`.
   - Click copy button and confirm clipboard receives the complete absolute path.
3. **Prompts Action Test**:
   - Click the "More Actions" dropdown on an instance row.
   - Click "Prompts & History" and verify the Prompt Tree modal opens for that instance.
4. **Pre-flight Check**:
   - Run `npm run build` to verify clean compilation.

---

## 4. Done When Checklist

- [ ] `Profile & Account` column rendered as a single stacked cell.
- [ ] Directory paths truncated to `...\<parent>\<leaf>` with copy button and full hover title.
- [ ] Cell padding reduced to `px-2 py-1.5`.
- [ ] Actions column fits neatly within 130px with segmented capsule and More dropdown portal.
- [ ] "Prompts & History" button functional in the actions menu.
- [ ] Zero horizontal scrollbar on 1280px desktop resolution.
- [ ] `npm run build` passes with zero TypeScript or Vite errors.
