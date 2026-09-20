# Subtask: Model Quota Filter Button Styling & Quota Badges

**Target Files:**
- `src/components/accounts/AccountTable.tsx`
- `src/components/accounts/QuotaItem.tsx`
- `src/components/accounts/AccountRow.tsx`

**Action:**
1. In `src/components/accounts/AccountTable.tsx`:
   - Redesign the `All | Gemini | Claude` segmented control in the table header:
     - Container: `inline-flex p-0.5 rounded-lg bg-gray-200 dark:bg-slate-900 border border-gray-300/80 dark:border-slate-800`.
     - Active selection: Prominent amber/yellow highlight `bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/30 font-bold shadow-xs`.
     - Inactive state: `text-gray-600 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-200`.
2. In `src/components/accounts/QuotaItem.tsx` and `AccountRow.tsx`:
   - Refine quota pill backgrounds to avoid muddy dark teal/green appearance.
   - Brighten label text to `text-slate-300 font-medium` and ensure percentage and countdown indicators are crisp and clean.

**Constraints:**
- Selected button MUST have distinct amber/yellow highlight per user request.
- No blurry subpixel text.
