# Subtask 01: Instance Section UI Polish & Switch Button Styling

## Target Files
- `src/pages/instances/InstanceCard.tsx`
- `src/pages/instances/InstanceCardActions.tsx`
- `src/components/instances/table/InstanceRowActions.tsx`
- `src/components/instances/table/InstanceRowCells.tsx`

## Objectives
1. Eliminate washed out / low-contrast container styling in `InstanceCard.tsx`. Use Royal Violet dark glass `dark:bg-[#171025]/90 border border-slate-200/80 dark:border-[#40285e]/80 hover:dark:bg-[#1f1533] hover:dark:border-[#7c3aed]/60`.
2. Upgrade Switch Account button in `InstanceCardActions.tsx` (Slot 2):
   - Replace dull transparent state with an illuminated amber badge (`text-amber-500 dark:text-amber-400 bg-amber-500/10 dark:bg-amber-500/15 border-x border-amber-500/30`).
   - Maintain 3D yellow tactile hover gradient (`hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:text-black font-bold shadow-[0_2px_0_0_#A16207]`).
   - Add explicit DOM IDs: `id="instance-switch-${inst.config.id}"`, `data-ui-name="instance-switch-btn"`, `data-ui-id="instance-switch-${inst.config.id}"`.
3. Add DOM IDs to all lifecycle buttons in `InstanceCardActions.tsx`:
   - `instance-launch-${inst.config.id}`
   - `instance-stop-${inst.config.id}`
   - `instance-restart-${inst.config.id}`
   - `instance-fastforward-${inst.config.id}`
   - `instance-more-${inst.config.id}`
4. Ensure Table view `InstanceRowActions.tsx` maintains visual parity and tactile contrast.
