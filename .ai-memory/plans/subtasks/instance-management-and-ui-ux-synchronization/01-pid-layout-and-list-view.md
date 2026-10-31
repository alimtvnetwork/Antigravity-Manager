# Subtask 01: PID Layout & List View Single-Row UI Refactor

## Objectives
1. Confirm numeric value (e.g. `1632`) is the Process ID (`PID`).
2. Merge separate `Status & PID` and `IDE EXE & DATA` table headers in `src/components/instances/InstanceTable.tsx` into a single combined column: `Status, Process & Environment`.
3. Refactor `src/components/instances/table/InstanceRowCells.tsx` to render the running status / PID badge (`PID: <pid>`), the trailing executable name (`.../Antigravity.exe`), and the trailing user data directory (`.../antigravity-data`) on the exact same horizontal flex row, separated by `·`.
4. Group copy action buttons into a contiguous segmented pill capsule (`Terminal` copy exe, `Copy` copy data dir).

## Owned Files
- `src/components/instances/InstanceTable.tsx`
- `src/components/instances/table/InstanceRowCells.tsx`

## Verification
- Frontend compiles with `npm run build` or Vite build check without errors.
- Visual check confirms single-row horizontal alignment.
