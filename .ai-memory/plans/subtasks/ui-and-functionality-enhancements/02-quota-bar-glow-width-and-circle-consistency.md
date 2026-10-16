---
plan: ui-and-functionality-enhancements
subtask: "02"
title: QuotaProgressBar Glow Softening, 10% Width Shrinkage, and 4 Equal-Distance Checkpoint Consistency
domain: frontend-ui
depends_on: 01-titlebar-maximize-and-compact-status-capsule.md
citations:
  architecture_spec: ../../../../02-spec/21-app/ui-and-functionality-enhancements/01-architecture-and-ui-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/accounts/QuotaProgressBar.tsx
  - src/components/accounts/AccountTable.tsx
  - src/components/accounts/AccountTable/AccountRowContent.tsx
status: pending
---

# Subtask 02: QuotaProgressBar Glow Softening, 10% Width Shrinkage, and 4 Equal-Distance Checkpoint Consistency

## 1. Context & Motivation
Feedback on the account quota visualization highlighted three visual and spatial deficiencies:
1. **Harsh Neon Glare**: The current hover and ambient neon glows (`rgba(26,241,141,0.85)` / `shadow-[0_0_12px_...]`) are overly intense, producing visual fatigue and halo distortion against dark table rows.
2. **Uneven & Inconsistent Checkpoints**: Currently, the progress bar places 5 checkpoints at `[20, 40, 60, 80, 90]`, creating an awkward asymmetric gap between 80% and 90% and using jarring chromatic changes (red $\to$ orange $\to$ yellow $\to$ green). The checkpoints must be restructured to 4 equal-distance intervals at `[20, 40, 60, 80]` with a unified consistent emerald styling.
3. **Table Column Horizontal Bloat**: Quota columns currently consume excessive table width (`min-w-[220px]` in headers, `min-w-[210px]` in cells, and `min-w-[140px]` in progress tracks), starving the Email column of necessary display room. Reducing the width by 10% reclaims horizontal real estate for email readability.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/accounts/QuotaProgressBar.tsx` | `QuotaProgressBar`, `DEFAULT_CHECKPOINTS`, styles | 1. Change default checkpoints to `[20, 40, 60, 80]`.<br/>2. Unify filled node color to `#10b981` (emerald-500).<br/>3. Reduce track `min-w-[140px]` to `min-w-[126px]`.<br/>4. Soften neon box shadows and drop-shadows to gentle ambient emerald glows. |
| `src/components/accounts/AccountTable.tsx` | `AccountTable` table headers | Reduce 4H Quota header `min-w-[220px]` $\to$ `min-w-[195px]`, reduce Weekly Quota header `min-w-[220px]` $\to$ `min-w-[195px]`. |
| `src/components/accounts/AccountTable/AccountRowContent.tsx` | `AccountRowContent` quota cells | Reduce 4H Quota cell `min-w-[210px]` $\to$ `min-w-[185px]`, reduce Weekly Quota cell `min-w-[210px]` $\to$ `min-w-[185px]`. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Restructure Checkpoints and Soften Glows in `src/components/accounts/QuotaProgressBar.tsx`

1. Update default `checkpoints` prop in `QuotaProgressBarProps` to `[20, 40, 60, 80]`:
   ```tsx
   checkpoints = [20, 40, 60, 80],
   ```
2. Define the unified emerald milestone color constant:
   ```tsx
   const UNIFIED_EMERALD_COLOR = '#10b981';
   ```
3. Soften container hover glow:
   - Change:
     `hover:drop-shadow-[0_0_8px_rgba(26,241,141,0.5)] group-hover:drop-shadow-[0_0_8px_rgba(26,241,141,0.5)]`
   - To:
     `hover:drop-shadow-[0_0_5px_rgba(16,185,129,0.25)] group-hover:drop-shadow-[0_0_5px_rgba(16,185,129,0.25)]`
4. Reduce track width by 10%:
   - Change:
     `<div className="relative flex-1 min-w-[140px] flex items-center">`
   - To:
     `<div className="relative flex-1 min-w-[126px] flex items-center">`
5. Soften track ambient glow overlay:
   - Change:
     `id="agm-progress-glow" className="... shadow-[0_0_12px_rgba(26,241,141,0.6)]"`
   - To:
     `id="agm-progress-glow" className="... shadow-[0_0_6px_rgba(16,185,129,0.25)]"`
6. Update milestone nodes to use equal 20% intervals and unified emerald styling:
   ```tsx
   {showCheckpoints && sortedCheckpoints.map((cp, idx) => {
       const isFilled = clamped >= cp;
       return (
           <div
               key={cp}
               id={`agm-progress-node-${idx + 1}`}
               className={cn(
                   "agm-progress-node absolute top-1/2 -translate-y-1/2 -translate-x-1/2 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none border-[1.5px]",
                   compact ? "w-3.5 h-3.5" : "w-4.5 h-4.5",
                   isFilled
                       ? "border-transparent bg-emerald-500 shadow-[0_1px_4px_rgba(0,0,0,0.3)] group-hover:shadow-[0_0_6px_rgba(16,185,129,0.4)]"
                       : "bg-slate-300/50 dark:bg-white/10 border-transparent shadow-[inset_0_1px_2px_rgba(0,0,0,0.2)]"
               )}
               style={{
                   left: `${cp}%`,
                   ...(isFilled ? { backgroundColor: UNIFIED_EMERALD_COLOR } : {}),
               }}
               title={`Milestone ${cp}%`}
           >
               <svg
                   className={cn(
                       "pointer-events-none transition-colors duration-200",
                       compact ? "w-2 h-2" : "w-2.5 h-2.5",
                       isFilled ? "text-white drop-shadow-[0_1px_1px_rgba(0,0,0,0.4)]" : "text-slate-400 dark:text-white/35"
                   )}
                   viewBox="0 0 16 16"
                   fill="none"
                   stroke="currentColor"
                   strokeWidth="2.6"
                   strokeLinecap="round"
                   strokeLinejoin="round"
               >
                   <path d="M4 6.5L8 10.5L12 6.5" />
               </svg>
           </div>
       );
   })}
   ```

### 3.2 Step 2: Shrink Quota Column Headers in `src/components/accounts/AccountTable.tsx`
1. Locate the 4H Quota column header `<th>` (around line 197):
   - Replace `min-w-[220px]` with `min-w-[195px]`.
2. Locate the Weekly Quota column header `<th>` (around line 250):
   - Replace `min-w-[220px]` with `min-w-[195px]`.

### 3.3 Step 3: Shrink Quota Row Cells in `src/components/accounts/AccountTable/AccountRowContent.tsx`
1. Locate the 4H Quota row cell `<td>` (around line 196):
   - Replace `min-w-[210px]` with `min-w-[185px]`.
2. Locate the Weekly Quota row cell `<td>` (around line 237):
   - Replace `min-w-[210px]` with `min-w-[185px]`.

---

## 4. Visual & DOM Invariants
- The checkpoints rendered MUST strictly be positioned at `left: 20%`, `left: 40%`, `left: 60%`, `left: 80%`.
- All filled checkpoints MUST render the unified emerald green (`#10b981`) background, eliminating red/orange/yellow milestone deviations.
- The minimum width of `#agm-progress-track` flex container MUST be `min-w-[126px]`.
- Table header and cell widths for quota columns MUST be `min-w-[195px]` and `min-w-[185px]` respectively.

---

## 5. Verification Checklist
- [ ] Inspect quota progress bar on account table: 4 checkpoint circles appear at uniform 20%, 40%, 60%, 80% marks.
- [ ] For an account with 75% quota, nodes 1, 2, and 3 (20%, 40%, 60%) are filled emerald green, while node 4 (80%) is unfilled.
- [ ] For an account with 100% quota, all 4 nodes are filled with identical emerald color (`#10b981`).
- [ ] Hover over progress bar: gentle emerald ambient glow activates without blinding neon halo.
- [ ] Measure table columns in Chrome DevTools: quota columns take 195px/185px, giving extra room to the Email column.
- [ ] Pre-flight checks: `npm run build` succeeds without warnings.
