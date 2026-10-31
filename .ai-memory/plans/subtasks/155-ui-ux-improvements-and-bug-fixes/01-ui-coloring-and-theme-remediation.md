# Subtask 01: UI Coloring & Theme IACVT Invalidation Remediation

## Objective
Restore deep dark-glass aesthetics across the Prompt Tree View modal, eliminating the stark white left panel and faded borders introduced by invalid Tailwind arbitrary opacity declarations.

## Target Files
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- `src/components/instances/PromptTreeView/TreePanel.tsx`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/common/ThemeManager.tsx`

## Technical Details
1. Replace `dark:bg-[var(--app-bg,#090611)]/95` on the modal shell with `dark:bg-[#0c0817]/95` or `dark:bg-slate-950/95`.
2. Replace `dark:bg-[var(--app-bg,#090611)]/60` on the left sidebar container with `dark:bg-[#090611]/80` or `dark:bg-slate-900/60` and restore `dark:border-purple-900/40` or `dark:border-slate-800`.
3. In `ThemeManager.tsx`, provide RGB triplets (e.g. `--app-bg-rgb: 9, 6, 17;`) alongside hex codes so any opacity-modified classes compile cleanly.
4. Darken hover states across list items, action buttons, and tree rows.
