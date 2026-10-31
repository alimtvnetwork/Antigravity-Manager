# Subtask 03: Global 24-Hour Datetime Formatting & Relative Badges

**File**: `.ai-memory/plans/subtasks/ui-ux-redesign-and-bug-fix-instructions/03-datetime-formatting-and-relative-badges.md`  
**Parent Plan**: `.ai-memory/plans/ui-ux-redesign-and-bug-fix-instructions.md`  
**Status**: Pending  

## Objectives
1. In `src/utils/timeFormatters.ts` (or central datetime helper):
   - Implement `formatDateTime24h(isoString, includeSeconds)` formatting as `DD-MMM-YYYY HH:mm` (e.g. `18-Oct-2026 21:40`) using local system timezone.
   - Omit seconds when time range/duration is large (seconds only if within recent 1 minute or explicitly requested).
   - Implement `formatRelativeTimeBadge(isoString)` returning human-friendly relative labels like "ran 5 minutes ago", "ran 1 day ago", "running now".
   - Support user configuration in settings for time format preferences while enforcing UTC0 storage invariant in SQLite / backend.
2. Update `PromptInstructionView.tsx`, `DetailHeader.tsx`, and `ResultsView.tsx` to display timestamps and relative badges cleanly.
