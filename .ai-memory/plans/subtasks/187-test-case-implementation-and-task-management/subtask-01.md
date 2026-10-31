# Subtask 01: UI DropdownMenu & AccordionSection Accessibility Modernization

- **Task**: Test Case Implementation and Task Management (Sequence 187)
- **Slug**: `187-test-case-implementation-and-task-management`
- **Subtask**: 01 (Steps 1 & 2)
- **Status**: [COMPLETED]

## Scope
1. **Step 1 (Track C - Step 13)**: Modernize `src/components/common/DropdownMenu.tsx`.
   - Add `aria-haspopup="true"` and `aria-expanded={isOpen}` to trigger.
   - Implement keyboard navigation on trigger (`Enter`, `Space`, `ArrowDown`) to open dropdown.
   - Implement focus-visible ring styles and dark-glass contrast tokens.
2. **Step 2 (Track C - Step 14)**: Modernize `src/components/common/AccordionSection.tsx`.
   - Add `aria-labelledby` linking collapsible region to header title.
   - Add accessible focus-visible ring on header trigger.
   - Ensure clean chevron animation and color contrast in dark/light mode.
