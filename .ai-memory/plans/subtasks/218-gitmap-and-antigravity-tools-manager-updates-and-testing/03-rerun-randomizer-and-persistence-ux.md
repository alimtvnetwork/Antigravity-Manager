# Subtask 218-03: Rerun Randomizer UX & Persistent Execution Engine
Task ID: `Imp-717`
Sequence: 218
Step: 303
Status: COMPLETED

## Scope
- Set default `rerunMessageOption` to `'randomized'` across DetailHeader, context menus, and modals.
- Elevate default rerun count to 4 (range 1–8).
- Expand actionable variation set to 16+ distinct nudges in `RERUN_RANDOM_MESSAGE_SET` and add extended options to `RERUN_MESSAGE_OPTIONS`.
- Persist user rerun preference in `localStorage` under `agm:prompt-tree:rerun-option` and `agm:prompt-tree:rerun-count`.
- Ensure DetailHeader modal/popover, tree context menu, and keyboard shortcut 'R' all pick from the randomized set when `rerunMessageOption === 'randomized'`.

## Completed Verifications
- `RERUN_DEFAULT_MESSAGE_OPTION`: `'randomized'`
- `RERUN_DEFAULT_COUNT`: `4` (min `1`, max `8`)
- `RERUN_RANDOM_MESSAGE_SET`: Contains 20 items (including all 16 specified actionable prompts)
- `RERUN_MESSAGE_OPTIONS`: Expanded with 'run-verification' and 'confirm-pushed'
- `rerunActions.test.ts`: Verified 16+ items and random pick validation.
