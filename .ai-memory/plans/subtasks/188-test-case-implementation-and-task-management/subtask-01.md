# Subtask 01: UI SegmentedTabGroup & SegmentedCapsuleRadio Accessibility Modernization

- **Task**: Test Case Implementation and Task Management (Sequence 188)
- **Slug**: `188-test-case-implementation-and-task-management`
- **Subtask**: 01 (Steps 1 & 2)
- **Status**: [COMPLETED]

## Scope
1. **Step 1 (Track C - Step 17)**: Modernize `src/components/common/SegmentedTabGroup.tsx`.
   - Add `aria-label` to container with `role="tablist"`.
   - Add `Home` and `End` key handling for roving tab selection.
   - Refine focus-visible ring styles and dark-glass contrast tokens.
2. **Step 2 (Track C - Step 18)**: Modernize `src/components/common/SegmentedCapsuleRadio.tsx`.
   - Add `aria-label` to container with `role="radiogroup"`.
   - Implement roving `tabIndex` (`0` for selected, `-1` for unselected).
   - Implement arrow-key keyboard navigation and focus-visible rings.
