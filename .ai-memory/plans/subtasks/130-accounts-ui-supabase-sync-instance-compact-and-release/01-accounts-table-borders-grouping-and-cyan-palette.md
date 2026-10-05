# Subtask: 01-accounts-table-borders-grouping-and-cyan-palette

- **Task ID**: `130-accounts-ui-supabase-sync-instance-compact-and-release`
- **Subtask Number**: `01`
- **Status**: `[QUEUED]`
- **Spec Reference**: [02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md](../../../../02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md)
- **Parent Plan**: [.ai-memory/plans/130-accounts-ui-supabase-sync-instance-compact-and-release.md](../../130-accounts-ui-supabase-sync-instance-compact-and-release.md)

---

## 1. Objective

Refine the Accounts view styling by:
1. Replacing harsh neon green (`#1af18d`, `#10b981`) progress bar tracks and blinding milestone checkpoint glows with a professional VS Code cyan/teal theme palette (`from-teal-500 via-cyan-500 to-sky-500`).
2. Restoring crisp row borders on every table row (`border-b border-slate-200/80 dark:border-slate-800/80`).
3. Introducing subtle visual grouping for the middle quota section (`4H Model Quota` and `Weekly Quota`) so quota metrics stand out cleanly without blending into surrounding columns.
4. Softening audit badges (`CURRENT`, `DISABLED`, `PROXY_DISABLED`, `FORBIDDEN`, `VALIDATION_BLOCKED`, `LEASED`) to muted VS Code styling with 5–6px border radius (`rounded-[5px]`).

---

## 2. Visual Ground Truth References

- **Theme Catalogue & Neon Green Defect**: `![Theme Catalogue and Quota Progress](assets/screenshots/130-accounts-ui-01.png)`
- **Quota Bars & Middle Section Defect**: `![Accounts Table and Quota Bars](assets/screenshots/130-accounts-ui-03.png)` (Note the two red-boxed areas highlighting the 4H and Weekly quota bars).

---

## 3. Target Files & Surgical Code Modifications

### 3.1 `src/components/accounts/QuotaProgressBar.tsx`
- **Track Gradient Overhaul (`getTrackGradient`)**:
  ```tsx
  // Replace neon hex codes with VS Code teal/cyan/sky palette
  const getTrackGradient = (pct: number) => {
      if (pct >= 75) return 'bg-gradient-to-r from-teal-500 via-cyan-500 to-sky-500';
      if (pct >= 50) return 'bg-gradient-to-r from-teal-600 via-cyan-500 to-amber-400';
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

### 3.2 `src/components/common/WaterDrainProgressBar.tsx`
- **Track Gradient Alignment**:
  Ensure identical VS Code cyan/teal progression matching `QuotaProgressBar.tsx`.
- **Milestone Checkpoint Node Softening (`getNodeStyle`)**:
  Eliminate harsh neon green glow (`#1af18d` / `shadow-[0_0_8px_rgba(26,241,141,0.6)]`), replacing Checkpoint 0 with:
  ```tsx
  case 0: // 100% Filled Bubble: soft cyan with subtle glow
      return "bg-cyan-400 dark:bg-cyan-500 border-[1.5px] border-cyan-300 dark:border-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.4)]";
  case 1: // 75% Filled Bubble: calm teal, no glow
      return "bg-teal-500 border-[1.5px] border-teal-400 shadow-none";
  case 2: // 50% Filled Bubble: amber, no glow
      return "bg-amber-400 dark:bg-amber-500 border-[1.5px] border-amber-300 dark:border-amber-400 shadow-none";
  case 3: // 25% Filled Bubble: orange, no glow
      return "bg-orange-500 border-[1.5px] border-orange-400 shadow-none";
  ```

### 3.3 `src/components/accounts/AccountRow.tsx`
- **Crisp Table Row Separation**:
  Ensure every `<tr>` carries consistent bottom border and transition:
  ```tsx
  className={cn(
      "group transition-all duration-200 border-b border-slate-200/80 dark:border-slate-800/80 border-l-2",
      isFocused
          ? "bg-teal-50/90 dark:bg-[#0e2c44] text-slate-900 dark:text-cyan-300 font-bold border-l-cyan-500 dark:border-l-cyan-400 shadow-md ring-1 ring-cyan-500/30"
          : isCurrent
          ? "bg-blue-50/70 dark:bg-[#091b2c] border-l-blue-600 dark:border-l-amber-400 border-blue-200 dark:border-amber-400/40 font-semibold text-blue-900 dark:text-amber-300 shadow-xs ring-1 ring-blue-400/30 dark:ring-amber-400/30 hover:bg-blue-100/60 dark:hover:bg-[#0c2438]"
          : selected
          ? "bg-blue-50/90 dark:bg-[#0f273d] text-blue-950 dark:text-blue-100 border-l-blue-500 dark:border-l-blue-500 font-semibold shadow-xs ring-1 ring-blue-500/30"
          : "border-l-transparent text-gray-800 dark:text-gray-200 hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60 hover:text-slate-900 dark:hover:text-white hover:border-l-blue-500/70"
  )}
  ```
- **Middle Section Quota Column Grouping**:
  Enclose the 4H and Weekly quota cells with subtle shading and dividing borders:
  ```tsx
  {/* 4H 模型配额 */}
  <td className="px-2.5 py-1 align-middle min-w-[210px] w-1/2 bg-slate-50/40 dark:bg-slate-900/25 border-l border-slate-200/50 dark:border-slate-800/50">
      ...
  </td>

  {/* Weekly 配额 */}
  <td className="px-2.5 py-1 align-middle min-w-[210px] w-1/2 bg-slate-50/40 dark:bg-slate-900/25 border-r border-slate-200/50 dark:border-slate-800/50">
      ...
  </td>
  ```
- **Status & Audit Badges Corner Radius Standard**:
  Update all badges (`CURRENT`, `DISABLED`, `PROXY_DISABLED`, `FORBIDDEN`) to use `rounded-[5px]`.

### 3.4 `src/components/accounts/AccountTable.tsx`
- **Table Header Grouping (`thead tr`)**:
  Group the Quota column headers with matching border accents and background tint, highlighting the `Gemini` / `Claude` sub-toggle cleanly.

---

## 4. Verification & Quality Acceptance Criteria

1. **Glow & Color Elimination**: Codebase search across `src/components/accounts/QuotaProgressBar.tsx` and `src/components/common/WaterDrainProgressBar.tsx` confirms zero instances of `#1af18d` or `#10b981`.
2. **Visual Inspection**: Progress bars display smooth cyan-to-sky gradients for healthy quotas (>= 75%) without eye-straining neon auras.
3. **Border Visibility**: In both light and dark modes, every account row has a visible 1px dividing border (`border-b border-slate-200/80 dark:border-slate-800/80`).
4. **Middle Section Grouping**: The quota region (4H and Weekly columns) forms a coherent visual group with subtle background shading (`bg-slate-50/40 dark:bg-slate-900/25`).
5. **No Regressions**: Account selection, switching, deletion, and details modal triggers remain 100% functional.
