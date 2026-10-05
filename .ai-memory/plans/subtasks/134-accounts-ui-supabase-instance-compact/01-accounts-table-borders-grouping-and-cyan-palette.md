# Subtask: 01-accounts-table-borders-grouping-and-cyan-palette

- **Task ID**: `134-accounts-ui-supabase-instance-compact`
- **Subtask Number**: `01`
- **Status**: `[QUEUED]`
- **Spec Reference**: [02-spec/21-app/134-accounts-ui-supabase-instance-compact/01-architecture-spec.md](../../../../02-spec/21-app/134-accounts-ui-supabase-instance-compact/01-architecture-spec.md)
- **Parent Plan**: [.ai-memory/plans/134-accounts-ui-supabase-instance-compact.md](../../134-accounts-ui-supabase-instance-compact.md)

---

## 1. Objective

Modernize the Accounts view and audit badges by:
1. Replacing harsh neon green (`#1af18d`, `emerald-500`, `lime-400`) progress bar tracks, neon glowing checkpoint bubbles, and audit badges with a professional VS Code cyan/teal theme palette (`from-teal-500 via-cyan-500 to-[#38bdf8]`).
2. Restoring clean, explicit row borders across all table rows (`border-b border-slate-200/90 dark:border-slate-800/90`).
3. Implementing distinct visual grouping for the middle quota section (`4H Model Quota` and `Weekly Quota` columns) so quota metrics stand out clearly as a cohesive visual envelope with subtle background tinting (`bg-slate-50/50 dark:bg-slate-900/40`) and outer group border demarcations (`border-x border-slate-300 dark:border-slate-700`).
4. Modernizing audit badges in `src/pages/Audit.tsx` from oversaturated green/emerald to refined VS Code cyan/teal and slate styling with standardized 5–6px border radii (`rounded-[5px]`).

---

## 2. Visual Ground Truth References

- **Theme Catalogue & Neon Green Defect**: `![Theme Catalogue and Quota Progress](assets/screenshots/134-accounts-ui-03-theme-catalogue-quota-bars.png)`
  *Defect:* In dark themes (Tokyo Night Slate, One Dark Pro, Antigravity Dark), the 100% quota progress bar emits an aggressive neon green gradient with glowing neon dot indicators that clash with the dark slate IDE aesthetics.
- **Header Toggle & Toolbar Spacing**: `![Show All Quotas Header Toggle](assets/screenshots/134-accounts-ui-04-show-all-quotas-toggle.png)`
  *Defect:* The header toolbar controls (`Focus`, `+`, `Refresh`, `Show All Quotas`) lack crisp vertical separation from the table, and the table rows below have faint, washed-out divider lines.
- **Accounts Table Red Box Deficiencies**: `![Accounts Table Red Boxes](assets/screenshots/134-accounts-ui-05-accounts-table-red-boxes.png)`
  *Defect:* User highlighted three specific red-box areas:
  - Box 1: Saturated green progress bar in the top-left preview and 4H quota column.
  - Box 2: Missing structural middle section grouping around 4H Model Quota and Weekly Quota columns.
  - Box 3: Weekly quota progress bar styling and column boundaries.

---

## 3. Target Files & Surgical Code Modifications

### 3.1 `src/components/accounts/QuotaProgressBar.tsx`
- **Track Gradient Overhaul (`getTrackGradient`)**:
  Replace all saturated neon green hexes and Tailwind emerald/lime classes with the VS Code professional cyan/teal/sky palette:
  ```tsx
  const getTrackGradient = (pct: number) => {
      if (pct >= 75) return 'bg-gradient-to-r from-teal-500 via-cyan-500 to-[#38bdf8]';
      if (pct >= 50) return 'bg-gradient-to-r from-teal-600 via-cyan-500 to-sky-400';
      if (pct >= 25) return 'bg-gradient-to-r from-amber-400 via-amber-500 to-orange-500';
      return 'bg-gradient-to-r from-orange-500 via-rose-500 to-rose-600';
  };
  ```
- **Percentage & Time Color Classes**:
  ```tsx
  const getPercentColorClass = (pct: number) => {
      if (pct >= 50) return 'text-teal-700 dark:text-cyan-400';
      if (pct >= 25) return 'text-amber-700 dark:text-amber-400';
      return 'text-rose-600 dark:text-rose-400';
  };
  ```
- **Milestone Checkpoint Bubble Styling (`getNodeStyle`)**:
  Eliminate neon green glowing bubbles (`shadow-[0_0_8px_rgba(26,241,141,0.6)]`), replacing Checkpoint 0 (100%) and Checkpoint 1 (75%) with soft cyan/teal:
  ```tsx
  const getNodeStyle = (idx: number, isFilled: boolean) => {
      if (!isFilled) {
          return "bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none";
      }
      switch (idx) {
          case 0: // 100% Checkpoint: VS Code soft cyan
              return "bg-cyan-400 dark:bg-cyan-500 border-[1.5px] border-cyan-300 dark:border-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.4)]";
          case 1: // 75% Checkpoint: Calm teal
              return "bg-teal-500 border-[1.5px] border-teal-400 shadow-none";
          case 2: // 50% Checkpoint: Warm amber
              return "bg-amber-400 dark:bg-amber-500 border-[1.5px] border-amber-300 dark:border-amber-400 shadow-none";
          case 3: // 25% Checkpoint: Orange
              return "bg-orange-500 border-[1.5px] border-orange-400 shadow-none";
          default:
              return "bg-rose-500 border-[1.5px] border-rose-400 shadow-none";
      }
  };
  ```

### 3.2 `src/components/common/WaterDrainProgressBar.tsx`
- Ensure track gradients and node bubble colors match `QuotaProgressBar.tsx` exactly, eliminating any remaining emerald/lime/neon green classes.

### 3.3 `src/components/accounts/AccountTable.tsx`
- **Table Container & Row Borders**:
  - The outer table container must maintain crisp borders:
    `<div className="overflow-x-auto rounded-xl border border-slate-200/90 dark:border-slate-800/90 bg-white dark:bg-[#071724] shadow-xs">`
  - The `<tbody>` must have explicit row dividing lines:
    `<tbody className="divide-y divide-slate-200/90 dark:divide-slate-800/90">`
- **Middle Section Quota Header Grouping (`thead tr`)**:
  - Group Column 4 (`4H Quota`) and Column 5 (`Weekly Quota`) under a distinct visual envelope:
    ```tsx
    {/* Column 4: 4H Quota with Gemini/Claude Toggle */}
    <th className="px-2.5 py-1 text-left rtl:text-right text-xs font-medium uppercase tracking-wider min-w-[220px] whitespace-nowrap bg-slate-100/60 dark:bg-[#091b2c]/80 border-l-2 border-slate-300 dark:border-cyan-900/60">
        ...
    </th>

    {/* Column 5: Weekly Quota */}
    <th className="px-2.5 py-1 text-left rtl:text-right text-xs font-medium uppercase tracking-wider min-w-[220px] whitespace-nowrap bg-slate-100/60 dark:bg-[#091b2c]/80 border-r-2 border-slate-300 dark:border-cyan-900/60">
        ...
    </th>
    ```
- **Sortable Account Row Cells (`SortableAccountRow` & `AccountRowContent`)**:
  - Apply matching middle section grouping background and borders to the body cells:
    ```tsx
    {/* 4H Model Quota Cell */}
    <td className="px-2.5 py-1 align-middle min-w-[210px] bg-slate-50/50 dark:bg-[#081b2b]/50 border-l-2 border-slate-300/80 dark:border-cyan-900/50 border-b border-slate-200/90 dark:border-slate-800/90">
        ...
    </td>

    {/* Weekly Quota Cell */}
    <td className="px-2.5 py-1 align-middle min-w-[210px] bg-slate-50/50 dark:bg-[#081b2b]/50 border-r-2 border-slate-300/80 dark:border-cyan-900/50 border-b border-slate-200/90 dark:border-slate-800/90">
        ...
    </td>
    ```

### 3.4 `src/components/accounts/AccountRow.tsx`
- Ensure standalone `AccountRow` component mirrors the same row borders and middle section grouping for parity when loaded outside DnD contexts.
- Ensure all status badges (`CURRENT`, `DISABLED`, `PROXY_DISABLED`, `FORBIDDEN`, `VALIDATION_BLOCKED`) strictly use `rounded-[5px]`.

### 3.5 `src/pages/Audit.tsx`
- **Replace Harsh Green Badges with VS Code Cyan/Teal Palette**:
  - Line 112 (Action Code 1 / Add):
    Replace `bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300`
    With: `bg-teal-50 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border-teal-200/60 dark:border-teal-800/60 rounded-[5px]`
  - Line 199 (Action Taken - Positive):
    Replace `bg-emerald-100 text-emerald-800 dark:bg-emerald-950/70 dark:text-emerald-300`
    With: `bg-cyan-50 text-cyan-800 dark:bg-cyan-950/70 dark:text-cyan-300 border-cyan-300/60 dark:border-cyan-800 rounded-[5px]`
  - Line 213 (Idle Check - Verified / Passed):
    Replace `bg-emerald-100 text-emerald-800 dark:bg-emerald-950/70 dark:text-emerald-300` with cyan styling:
    `bg-cyan-50 text-cyan-800 dark:bg-cyan-950/70 dark:text-cyan-300 border-cyan-300/60 dark:border-cyan-800 rounded-[5px]`
    Change pulse dot to: `w-1.5 h-1.5 rounded-full bg-cyan-500 animate-pulse`
  - Line 658 (Audit Table Status - Success / Dispatch OK):
    Replace `bg-emerald-50 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300`
    With: `bg-cyan-50 text-cyan-700 dark:bg-cyan-950/50 dark:text-cyan-300 border-cyan-200/60 dark:border-cyan-800/60 rounded-[5px]`

---

## 4. Verification & Quality Acceptance Criteria

1. **Color Audit**:
   - Zero occurrences of harsh neon green (`#1af18d`, `#10b981`, saturated lime/emerald glows) in quota bars or milestone bubbles.
   - Quota progress bars smoothly gradient across VS Code teal (`#0d9488`), cyan (`#06b6d4`), and sky (`#38bdf8`).
2. **Visual Row Separation**:
   - In both Light Clean and Dark themes (One Dark Pro, Tokyo Night Slate), every table row displays a clear 1px horizontal dividing line.
3. **Middle Section Grouping**:
   - The 4H Quota and Weekly Quota columns are visually anchored as a distinct grouped middle section with subtle shaded background and outer group border lines.
4. **Audit Badges Modernization**:
   - All audit badges in `src/pages/Audit.tsx` render in muted cyan/teal/slate with 5–6px border radius (`rounded-[5px]`).
5. **Component Parity**:
   - Both `AccountTable.tsx` (sortable) and `AccountRow.tsx` (standard) render identical border lines and middle column styling.
