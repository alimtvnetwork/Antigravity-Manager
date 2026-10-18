# Subtask 04: Switch Account Modal Royal Violet UI/UX Modernization

## Objectives
1. Modernize `src/pages/instances/SwitchAccountDialog.tsx` to align with the Royal Violet design system:
   - Base modal container: `dark:bg-[var(--ui-surface-1)]/95` (`#171025`) and `dark:border-[var(--ui-border-subtle)]` (`#40285e`).
   - Header accent: replace Sky blue with Royal Violet brand accent (`var(--ui-interactive)` / `#7c3aed`).
   - Search input: focus ring `focus:ring-[var(--ui-focus-ring)]`.
   - Card rows: `.ui-interactive-row` whole-card hover and focus states using `var(--ui-surface-hover)`.
   - Current account badge: clean emerald styling with `var(--ui-success)` (`#1af18d`).
   - Segmented quota visualization pill capsule: `4H: <val>% | Weekly: <val>% | Total: <val>%`.
2. Keyboard accessibility: support `Escape` key safe dismissal and `Enter`/`Space` activation.

## Owned Files
- `src/pages/instances/SwitchAccountDialog.tsx`

## Verification
- Frontend builds with zero TypeScript/CSS errors.
- Visual inspection shows unified dark-glass Royal Violet aesthetic.
