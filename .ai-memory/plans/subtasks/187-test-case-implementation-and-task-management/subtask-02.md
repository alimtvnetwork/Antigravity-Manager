# Subtask 02: UI TierBadge & SegmentedPillSwitch Accessibility Modernization

- **Task**: Test Case Implementation and Task Management (Sequence 187)
- **Slug**: `187-test-case-implementation-and-task-management`
- **Subtask**: 02 (Steps 3 & 4)
- **Status**: [COMPLETED]

## Scope
1. **Step 3 (Track C - Step 15)**: Modernize `src/components/common/TierBadge.tsx`.
   - Add semantic `role="status"` and accessible `aria-label` describing account tier.
   - Ensure WCAG 2.1 AA text-to-background contrast on Ultra, Pro, Free, and unknown tiers.
2. **Step 4 (Track C - Step 16)**: Modernize `src/components/common/SegmentedPillSwitch.tsx`.
   - Add explicit `aria-label="Options"` to radiogroup container.
   - Refine focus-visible ring styles and roving tabIndex support.
