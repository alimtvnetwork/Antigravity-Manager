# Subtask 03: Instances Table Column Compacting, Path Truncation, and Prompts Action Capsule Integration

- **Parent Task**: `130-accounts-ui-supabase-sync-instance-compact-and-release`
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
4. **Integrated Capsule Prompts Button**: Merging the "Prompts & Conversations" button (`Layers` icon) directly inside the segmented action capsule with compact responsive padding.
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
   function formatShortPath(fullPath: string): string {
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
  - Column 6 (`Actions`): `min-w-[180px] px-2 py-1.5 text-right`

### Step 4: Integrate Prompts Action Button into Segmented Capsule
In `src/components/instances/InstanceTable.tsx`:
1. Remove any loose external prompt buttons.
2. In the Actions cell, render the unified segmented capsule:
   - Outer container: `inline-flex items-center rounded-[5px] overflow-hidden bg-slate-100 dark:bg-[#071a27] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs`.
   - Action buttons:
     1. `Launch` / `Stop` (`px-1.5 py-1 rounded-l-[5px]`)
     2. `Switch Account` (`px-1.5 py-1 text-sky-600 dark:text-sky-400`)
     3. `Fast Forward` (`px-1.5 py-1 text-amber-600 dark:text-amber-400`)
     4. `Prompts` (`px-1.5 py-1 text-slate-600 dark:text-slate-300 hover:text-cyan-600 dark:hover:text-cyan-300`, icon: `<Layers className="w-3 h-3" />`, title: `"Prompts & Conversations"`, `onClick={() => onOpenPromptTree(inst.config.id)}`)
     5. `Audit Trail` (`px-1.5 py-1 text-slate-500 hover:text-amber-500`)
     6. `Sync PID & Quota` (`px-1.5 py-1 text-teal-600 dark:text-teal-400`)
     7. `Settings & Sync` (`px-1.5 py-1 text-slate-600 dark:text-slate-300`)
     8. `Clone Profile` (`px-1.5 py-1 text-indigo-600 dark:text-indigo-400`)
     9. `Delete Profile` (`px-1.5 py-1 text-rose-600 dark:text-rose-400 rounded-r-[5px]`)

### Step 5: Modernize Accents to Cyan/Teal
1. Replace all `text-emerald-500` launch spinner classes with `text-teal-500` or `text-cyan-400`.
2. Update active instance running badge to `bg-teal-50 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border border-teal-300/50 dark:border-teal-800/80`.
3. Set running pulse dot to `bg-teal-500 animate-pulse`.

---

## 3. Verification & Validation Steps

1. **Viewport Resolution Test**:
   - Open Antigravity Manager at 1280px browser/window width.
   - Navigate to Instances page in List/Table view.
   - Verify that no horizontal scrollbar appears on the table container.
   - Verify that all columns, including Actions, are visible simultaneously.
2. **Path Truncation & Copy Test**:
   - Locate an instance with a long file path (e.g. `C:\Users\Administrator\AppData\Roaming\Antigravity\profiles\instance_default`).
   - Confirm the display reads `...\profiles\instance_default`.
   - Hover to verify the browser tooltip displays the entire un-truncated path.
   - Click the Copy icon; verify the checkmark appears and the clipboard contains the full path.
3. **Prompts Button Test**:
   - Click the `Layers` icon inside the segmented capsule.
   - Verify `PromptTreeViewModal` opens cleanly for that specific instance.
4. **Pre-flight Check**:
   - Run `npm run build` to confirm zero TypeScript compilation or linting regressions.

---

## 4. Acceptance Criteria

- [ ] Table wrapper exhibits `scrollWidth <= clientWidth` on 1280px viewport width (0 horizontal scroll).
- [ ] Profile Name and Bound Email are merged into a clean stacked cell with status dots and badges.
- [ ] Data directory paths are truncated via `formatShortPath` with 1-click clipboard copy.
- [ ] Prompts button is seamlessly integrated inside the segmented action capsule with `Layers` icon.
- [ ] Segmented capsule container and edge buttons enforce strict `rounded-[5px]`.
- [ ] Spinners and running accents utilize the modernized VS Code teal/cyan palette.
