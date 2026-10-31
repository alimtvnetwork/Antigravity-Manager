# Subtask 01: UI List View Layout and PID Semantics

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `01-ui-list-view-layout-and-pid`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Clarify the process ID semantics in the instance list view and bring the executable trailing path and data directory trailing path onto the same line in a unified horizontal pill layout.

## Scope of Changes
- Confirm that the number (e.g. `1632`) is the OS PID (`process_id`).
- Display running state as `● Running (PID: 1632)` with clear emerald badge.
- Render executable tail and data directory tail inline on the same row using dark-glass badge pills with full path tooltips.
- Ensure no awkward text wrapping or vertical column bloat in table/list rows.

## Verification
- Component renders executable ending and data directory ending inline.
- PID clearly identified with `PID: ` prefix.
- Zero lint/build regressions in `npm run build`.
