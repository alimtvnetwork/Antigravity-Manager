# Subtask 02: Scoreboard Hover Candidate Card Contrast & Popover Polish

## Objective
Fix the Candidate Score hover popover in the Account table from blinding white `bg-white/95` to dark-glass styling with high-contrast, crystal-clear typography.

## Target Files
- `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx`

## Technical Details
1. Replace `dark:bg-[var(--app-surface,#18181b)]/95` with `dark:bg-slate-900/95 backdrop-blur-md border border-slate-700/60 shadow-2xl text-slate-100`.
2. Style the formula code container with `dark:bg-slate-950/80 border border-slate-800 text-emerald-400 font-mono`.
3. Ensure labels, multipliers, and readiness percentages render with crisp contrast against the dark background.
