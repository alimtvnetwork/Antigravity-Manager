# Subtask 3: Import/Export Menu Options: Combined Running+Queued & Restore

## Objective
1. Add "Export Running & Queued Prompts" option to `DetailHeader.tsx`.
2. Add "Restore Prompts from Backup" option to `DetailHeader.tsx` under the IMPORT section.
3. Wire handlers in `usePromptTreeCoreActions.ts` to dispatch combined backup and restore into `repo_prompts.db`.

## Key Files
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts`

## Verification
- Menu items visible in UI dropdown.
- Handlers invoke backup and restore cleanly.
