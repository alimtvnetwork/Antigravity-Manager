# Subtask 004: Quota Progress Bar UI Overhaul

## Owner: Worker 02 (Frontend Specialist)
## Target Files: `src/components/accounts/QuotaProgressBar.tsx`, `src/pages/Accounts.tsx`, `src/components/accounts/AccountRow.tsx`

### Requirements
1. **Equal Widths for 4H and Weekly Quota Bars**:
   - In `Accounts.tsx` and `AccountRow.tsx`, balance the column widths between 4H Model Quota and Weekly Quota to have equal width (`minmax(240px, 2fr)` each).
2. **Same Column for Hour and Percentage**:
   - Instead of placing time and percentage in horizontal separate columns/elements, stack them together in a single column:
     - Top line: Remaining time (e.g. `4h 19m` / `3d 11h`)
     - Bottom line: Percentage (e.g. `100%` / `33%`)
   - This frees up horizontal space, allowing the progress bar to stretch into available space.
3. **Resilient "Fatty" Progress Bar Height**:
   - Increase progress bar container height from `h-1.5` / `h-2` to `h-2.5` / `h-3` (`rounded-full`).
4. **Orange-to-Red Color Progression**:
   - Ensure the final segment/threshold transitions to red (`from-amber-500 to-rose-600` or `bg-rose-500`), providing an unmistakable high-usage warning.
5. **Enlarge 30% and 50% Weekly Quota Indicators**:
   - Ensure typography and pill badges for 30% and 50% weekly quota states are prominent and legible.
