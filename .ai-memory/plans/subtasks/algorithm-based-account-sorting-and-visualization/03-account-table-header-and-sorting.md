# Subtask 03: Account Table Header & Bidirectional Sorting

## Objective
Update `src/components/accounts/AccountTable.tsx` to:
1. Replace `USED` in `th` Column 6 with `SCORE` (or `ALGORITHM SCORE`).
2. Add `'score'` to `sortConfig.key` and update `handleSortToggle('score')`.
3. In `sortedAccounts`, implement sorting by `computeAccountAlgorithmScore(account).score` in `asc` / `desc`.
4. Update tooltip on sort button to reflect algorithm score sorting.
5. Ensure drag-and-drop reordering is properly paused when sorting by score (`isSortingActive`).

## Target Files
- `src/components/accounts/AccountTable.tsx`

## Acceptance Criteria
- Clicking `SCORE` sorts descending first (top-performing accounts at the top), then ascending, then resets.
- Sorting icons (`ArrowUp`, `ArrowDown`, `ArrowUpDown`) correctly toggle.
- DnD is disabled while sorting is active.
