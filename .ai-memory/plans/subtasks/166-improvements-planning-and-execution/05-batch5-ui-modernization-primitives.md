# Subtask 05: Batch 5 (Imp 295–300: Desktop React UI & UX Component Modernization)

- **Parent Plan**: `.ai-memory/plans/pending/166-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/166-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- Created `ProgressBarCapsule.tsx` with dynamic thresholds, milestone nodes, and size scaling.
- Created `TooltipPortal.tsx` with viewport collision protection and portal-level unclipped rendering.
- Created `numberFormatter.ts` supporting SI prefixes (k, M, B), token metrics, bytes, and durations.
- Created `SearchFilterBar.tsx` featuring keyboard shortcuts (Cmd+K / Ctrl+K) and active filter pills.
- Created `useClickOutside.ts` providing multi-ref mousedown/touchstart dismiss handling.
- Created `EmptyStateCard.tsx` with dashed border styling and primary/secondary call-to-actions.

## Targeted Improvements
- **Imp-295: Dark-Glass Progress Bar Capsule** (`src/components/common/ProgressBarCapsule.tsx`)
- **Imp-296: Portaled Non-Clipping Tooltip** (`src/components/common/TooltipPortal.tsx`)
- **Imp-297: High-Efficiency Number & Token Formatter** (`src/utils/numberFormatter.ts`)
- **Imp-298: Search & Filter Toolbar Primitive** (`src/components/common/SearchFilterBar.tsx`)
- **Imp-299: Multi-Target Click Outside Hook** (`src/hooks/useClickOutside.ts`)
- **Imp-300: Standard Empty State Placeholder Card** (`src/components/common/EmptyStateCard.tsx`)
