# Subtask 02: Algorithm Score Cell & Hover Tooltip

## Objective
Author `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx` and integrate it into `src/components/accounts/AccountTable/AccountRowContent.tsx`:
1. Render score percentage / points with high-contrast text.
2. Render sleek progress bar with dynamic width and green/amber/red color gradients.
3. Render interactive dark-glass hover tooltip (`pointer-events-none`) displaying:
   - Header with tier badge and score.
   - Exact mathematical breakdown (multiplication, division, percentage steps).
   - Rolling and weekly quota bottleneck statistics.
   - Status / Cooldown summary.
4. Replace raw `account.last_used` cell in `AccountRowContent.tsx`.

## Target Files
- `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`
- `src/components/accounts/AccountTable/AccountRowContent.tsx`

## Acceptance Criteria
- Clean dark-glass styling matching design tokens.
- Hover tooltip shows complete math formula with operations.
- Zero horizontal overflow; fits cleanly within table column bounds.
