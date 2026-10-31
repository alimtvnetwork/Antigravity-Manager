# Plan: Algorithm-Based Account Sorting and Visualization

## User Request (Verbatim)
> Instead of using this row, we should actually use the algorithm's percentage or the value of the algorithm, the internal algorithm, how we sort the accounts. So that should be placed in here instead of used. Can you please do that for me? And also based on the high number, it would have more green, and low number would show a little bit of red, like the progress bar, something like this. And we can sort the accounts using that as well.
>
> Also, if we hover over to that number, that would show all the details of the number, how this number actually calculated. For example, when you weekly put all these multiplication, division, everything would be show them the full calculation and how the value came out, it would also show that. Can you please do that for me?

---

## Architecture & Subtask Decomposition
This task replaces the `USED` date column in `AccountTable.tsx` / `AccountRowContent.tsx` with a dynamic Algorithm Score progress bar, hover breakdown card, and sorting engine.

### Subtasks:
1. `01-account-algorithm-scoring-utility.md`:
   - Centralize pure mathematical scoring calculation in `src/utils/accountAlgorithmScore.ts`.
   - Calculate normalized score, readiness percentage (0–100%), color scale, and detailed arithmetic formula breakdown (multiplication, division, status factor, tier factor).
   - Author comprehensive unit tests in `src/utils/accountAlgorithmScore.test.ts`.

2. `02-algorithm-score-cell-and-tooltip.md`:
   - Create `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`.
   - Implement the dynamic color-ramped progress bar (green for high numbers, amber for mid numbers, red for low numbers).
   - Implement the interactive dark-glass hover tooltip displaying the complete mathematical steps, multiplication, division, and tier details.
   - Wire `AlgorithmScoreCell` into `AccountRowContent.tsx` replacing the raw `last_used` date span.

3. `03-account-table-header-and-sorting.md`:
   - Update `AccountTable.tsx` table header: replace `USED` with `SCORE` (with sort toggle).
   - Add `'score'` / `'algorithm_score'` to `sortConfig` and `handleSortToggle`.
   - Implement descending/ascending sorting on `computeAccountAlgorithmScore(account).score`.
   - Ensure DnD reordering remains disabled while sorting is active (`isSortingActive`).

---

## Status Ledger
- [x] Phase 1A: Comprehension Mandate & Task Extraction
- [x] Phase 1B: Discovery Subagents (Research 01, 02, 03)
- [x] Phase 1C: Specification & Master Plan Authoring
- [x] Phase 2: Spec Subtasks & Subagent Execution Wave
  - [x] Subtask 01: Scoring Utility & Mathematical Engine (`src/utils/accountAlgorithmScore.ts`)
  - [x] Subtask 02: Progress Bar Cell & Mathematical Tooltip (`src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`)
  - [x] Subtask 03: Table Header & Bidirectional Sorting (`src/components/accounts/AccountTable.tsx`)
- [x] Phase 3: Targeted Verification, Pre-flight & GitMap Atomic Commit
