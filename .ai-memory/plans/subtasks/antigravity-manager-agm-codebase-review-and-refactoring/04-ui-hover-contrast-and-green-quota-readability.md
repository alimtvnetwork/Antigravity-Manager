# Subtask 04: UI Hover Contrast & Green Duration Quota Readability

**Parent Plan:** `.ai-memory/plans/antigravity-manager-agm-codebase-review-and-refactoring.md`  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md)  
**Owner:** Worker 02  
**Status:** Queued  

---

## 1. Owned Files
- `src/components/accounts/AccountTable/SortableAccountRow.tsx`
- `src/components/accounts/AccountTable/AccountRowContent.tsx`
- `src/pages/instances/InstanceCard.tsx`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/accounts/QuotaProgressBar.tsx`

---

## 2. Implementation Steps

### 2.1 Table Row Hover & White Flashing Elimination
1. In `src/components/accounts/AccountTable/SortableAccountRow.tsx`:
   - Replace any `hover:bg-slate-50` or `hover:!bg-white dark:hover:!bg-white` with balanced dark-glass opacity styling:
     `hover:bg-slate-100/80 dark:hover:bg-slate-800/60`
2. In `src/components/accounts/AccountTable/AccountRowContent.tsx`:
   - Ensure all regular `td` cells maintain transparent backgrounds (`bg-transparent`) so row hover colors flow through continuously.
   - Strip all cell-level `group-hover:!bg-white` and `group-hover:!text-black` overrides.
   - For sticky cells (e.g. `ActionsCell.tsx`), match the opaque background to the dark-glass theme to prevent text ghosting only during horizontal scrolling.

### 2.2 Instance Level Default Copy Badge Upgrade
1. In `src/pages/instances/InstanceCard.tsx` and `src/components/accounts/AccountTable/AccountRowContent.tsx`:
   - Locate the `default-copy` / cloned instance badge rendered adjacent to the Pro subscription tier badge.
   - Upgrade badge tokens to high-contrast specification:
     - **Light Mode:** `bg-indigo-100 text-indigo-950 border border-indigo-300 font-black tracking-wider`
     - **Dark Mode:** `dark:bg-indigo-950 dark:text-indigo-200 dark:border-indigo-500 font-black`
   - Ensure badge is sharply legible against both light and dark row hover states.

### 2.3 Green Duration Text to High-Contrast Micro-Pill Badge
1. In `src/components/instances/PromptTreeView/DetailHeader.tsx`, `InstanceCard.tsx`, and `QuotaProgressBar.tsx`:
   - Identify low-contrast `#1af18d` plain duration and status strings (e.g., elapsed execution time, quota reset hours).
   - Replace plain text with structured micro-pill badge:
     ```tsx
     <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[9.5px] font-mono font-bold bg-emerald-500/15 text-emerald-950 dark:text-emerald-200 border border-emerald-500/30 shadow-2xs">
         <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse shrink-0" />
         <span>{formattedDuration}</span>
     </span>
     ```
   - Ensure contrast ratio meets WCAG AA standards in both light and dark modes.

---

## 3. Verification & Quality Gates
- `npm run build` compiles with zero errors.
- Visual inspection confirms table rows do not flash stark white when hovering over cells.
- Visual inspection confirms `default-copy` badge renders dark indigo bold text on indigo background.
- Visual inspection confirms duration and running badges are enclosed in high-contrast emerald micro-pills.
