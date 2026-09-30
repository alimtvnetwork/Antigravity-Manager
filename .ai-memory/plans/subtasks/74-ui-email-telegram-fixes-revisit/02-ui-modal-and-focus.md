# Subtask 02: Fix UI Modals, Focus Auto-scroll & High Contrast
Traceability ID: Task-02, Task-03
Spec Reference: [02-spec/21-app/74-ui-email-telegram-fixes-revisit.md](../../../02-spec/21-app/74-ui-email-telegram-fixes-revisit.md)
Target Files: src/components/navbar/InstanceSelector.tsx, src/pages/Instances.tsx
Action: Fix dark-on-dark selection contrast (use white background / dark text for selected states). Ensure panels like Instances and Parts are mutually exclusive or have an explicit close button.
Acceptance Criteria: 
- Explicit close buttons 'X' on panels.
- Hover/selected states are easily readable.
- Smooth auto-scroll for active instance card on mount/switch.
Targeted Verification: UI component modal dismissal, contrast styling, and scrollIntoView integration.

Status: COMPLETED
Outcome:
1. `src/components/navbar/InstanceSelector.tsx`:
   - Added `Plus` button for creating new profiles directly from the dropdown header.
   - Added dropdown auto-closing (`setIsOpen(false)`) when opening any modal (Create, Copy, Edit, Delete).
   - Added explicit 'X' close buttons and backdrop click dismissal to all 4 dialogs.
   - High-contrast selected/hover state colors: avoids blue-under-blue or washed-out white in light and dark themes.
   - Escape key listener to close dropdowns and modals.
2. `src/pages/Instances.tsx`:
   - Added `activeCardRef` with smooth `scrollIntoView` for active instance on mount and selection.
   - Added explicit 'X' close buttons and backdrop click dismissal to Create, Copy, and Edit modals.
   - Added Escape key listener to dismiss all modals.
   - Enhanced active card styling with high-contrast text and border.
3. `src/pages/Accounts.tsx`:
   - Added state-driven `focusedAccountId` with 100ms polling retry loop across pagination and search filters, ensuring `scrollIntoView` works every time.
   - Added luminous animation ring (`ring-4 ring-blue-500 dark:ring-amber-400 ring-offset-2 dark:ring-offset-slate-900 shadow-2xl scale-[1.01]`).
4. `src/components/accounts/AccountCard.tsx` & `AccountRow.tsx`:
   - Eliminated muddy dark-on-dark contrast (`dark:bg-amber-950/40`, `dark:bg-amber-950/30`).
   - Replaced with high-contrast, theme-adaptive styling (`dark:bg-slate-800/95`, `dark:text-white font-bold`, luminous amber/blue borders, and non-clashing hover/selected styles).
5. `src/components/navbar/NavSettings.tsx`:
   - Added explicit 'X' close button to Preferences popup header.
   - Added Escape key listener to dismiss Preferences and Quick Clean modals.
   - Dispatched `agm:dropdown-open` on opening Quick Clean modal to prevent overlapping with InstanceSelector.
