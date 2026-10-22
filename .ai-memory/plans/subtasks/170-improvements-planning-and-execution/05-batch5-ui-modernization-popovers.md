# Subtask 05: Batch 5 (Imp 415–420: Desktop React UI & UX Component Modernization)

- **Parent Plan**: `.ai-memory/plans/pending/170-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/170-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-415: Dark Glass Popover Card** (`src/components/common/DarkGlassPopoverCard.tsx`)
   - Floating dark-glass popover card component with portal rendering, backdrop blur, arrow positioning, and automatic boundary clamping.
2. **Imp-416: Inline Copy Chip** (`src/components/common/InlineCopyChip.tsx`)
   - Compact code/text pill with click-to-copy, hover tooltip, and checkmark feedback.
3. **Imp-417: Click Outside Listener Hook** (`src/hooks/useClickOutsideListener.ts`)
   - Custom hook to dismiss popovers, menus, and flyouts when clicking outside target elements with multi-ref support.
4. **Imp-418: Quota Health Status Capsule** (`src/components/common/QuotaHealthStatusCapsule.tsx`)
   - Visual health status pill showing quota percentage with dynamic color gradients and pulsing state.
5. **Imp-419: Byte Size Formatter Utility** (`src/utils/formatByteSize.ts`)
   - High-precision zero-dependency byte formatting helper (`B`, `KB`, `MB`, `GB`) with decimal alignment.
6. **Imp-420: Expandable Code Block Banner** (`src/components/common/ExpandableCodeBlockBanner.tsx`)
   - Interactive dark-glass code/JSON viewer with collapse/expand toggle, line numbering, and copy-all action.
