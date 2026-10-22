# Subtask 05: Batch 5 (Imp 385–390: Desktop React UI & UX Component Modernization)

- **Parent Plan**: `.ai-memory/plans/pending/169-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/169-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-385: Origin Tier Badge Pill** (`src/components/common/OriginTierBadgePill.tsx`)
   - Renders 4-tier prompt origin classifications (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`) with icon, label, and high-contrast color styling.
2. **Imp-386: Segmented Tab Group** (`src/components/common/SegmentedTabGroup.tsx`)
   - Seamless pill segmented tab bar component with smooth indicator slide, dark-glass borderless styling, and ARIA keyboard support.
3. **Imp-387: Escape Dialog Dismissal Hook** (`src/hooks/useEscapeDialogDismissal.ts`)
   - Custom hook providing reliable Escape key modal dismissal, stopping propagation and tracking active modal stacks.
4. **Imp-388: Truncated Text Tooltip Capsule** (`src/components/common/TruncatedTextTooltip.tsx`)
   - Truncated text wrapper with interactive hover popover and 1-click copy action, avoiding layout distortion.
5. **Imp-389: Micro Time Ago Formatter** (`src/utils/microTimeAgo.ts`)
   - Compact humanized time difference helper (`now`, `12s`, `4m`, `2h`, `1d`) with sub-second precision.
6. **Imp-390: Micro Action Button Group** (`src/components/common/MicroActionButtonGroup.tsx`)
   - Reusable contiguous segmented pill capsule for table and card row actions (`Copy`, `Rerun`, `Delete`) with shared borders and hover states.
