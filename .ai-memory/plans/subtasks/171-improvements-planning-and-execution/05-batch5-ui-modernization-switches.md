# Subtask 05: Batch 5 (Imp 445–450: Desktop React UI Modernization, Accessibility & Primitives)

- **Parent Plan**: `.ai-memory/plans/pending/171-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/171-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-445: Contiguous Segmented Pill Toggle Switch** (`src/components/common/SegmentedPillSwitch.tsx`)
   - Contiguous dark-glass segmented switch component with animated sliding active pill highlight and keyboard navigation support.
2. **Imp-446: Top-Level Modal Portal Layer** (`src/components/common/ModalPortalLayer.tsx`)
   - Top-level modal backdrop portal container with z-index elevation (`z-[99999]`), focus containment, and click-outside dismissal.
3. **Imp-447: Accessible Keyboard-Only Focus Ring Detection Hook** (`src/hooks/useKeyboardFocusRing.ts`)
   - React hook detecting Tab/keyboard navigation to conditionally render high-contrast focus rings while suppressing mouse click rings.
4. **Imp-448: Pulsing Multi-State Status Dot Indicator** (`src/components/common/StatusDotIndicator.tsx`)
   - Compact status dot with configurable pinging animation, color tokens (green, amber, rose, slate, sky), and accessible ARIA attributes.
5. **Imp-449: Compact Metric Number Abbreviator Utility** (`src/utils/numberAbbreviator.ts`)
   - Formats large numbers into readable compact metric units (`1.2K`, `45.6M`, `2.1B`) with decimal precision and boundary safety.
6. **Imp-450: Contiguous Dark-Glass Compact Action Toolbar Capsule** (`src/components/common/CompactActionBar.tsx`)
   - Contiguous segmented pill capsule grouping action buttons with borderless styling, micro-dividers, and dark-glass aesthetics.
