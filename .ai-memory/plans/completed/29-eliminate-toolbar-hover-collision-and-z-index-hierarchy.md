# Plan 29: Eliminate Toolbar Hover Collision and Elevate Navbar Stacking Context

> **Plan Path:** `.ai-memory/plans/completed/29-eliminate-toolbar-hover-collision-and-z-index-hierarchy.md`
> **Status:** Completed
> **Task Origin & Inception:** User reported "Fix the UI please hover creates issue" with screenshot showing the Add Account `+` button rendered directly over the opened Instance dropdown menu, capturing hover events and blocking clicks.
> **Target Release:** v4.25.0

---

## 1. Problem Statement & Root Cause Analysis

### A. Add Account Button Poking Through Instance Dropdown
- **Root Cause:** In `src/components/accounts/AddAccountDialog.tsx`, line 468 had `className="... relative z-[100]"` applied to the inline trigger button.
- Because `Navbar.tsx` had `zIndex: 50`, `z-[100]` from the page body created a higher stacking context than the sticky navbar.
- When the `<InstanceSelector />` dropdown opened downwards, the `+` button in the Accounts toolbar sat directly underneath it and poked right through the dropdown, covering `Default` and capturing all mouse hover and click events.
- **Impact:** Users hovering over the instance profile menu inadvertently hovered the `+` button, causing flicker, broken selections, and modal popups.

---

## 2. Key Implementations & Enhancements

### A. Removed Rogue Z-Index on Add Account Trigger (`AddAccountDialog.tsx`)
- Stripped `relative z-[100]` from the trigger button on line 468.
- The button is now in standard document flow alongside the rest of the toolbar buttons.

### B. Navbar Stacking Context Elevation (`Navbar.tsx`)
- Elevated sticky navbar container to `zIndex: 60`, ensuring all navigation dropdowns always layer cleanly over body contents.

### C. Consistent Dropdown Z-Index & Bounds (`NavDropdowns.tsx`)
- Added `z-50 max-w-[calc(100vw-32px)]` to `LanguageDropdown` and `MoreDropdown`, ensuring all navbar popovers are consistently styled and safely bounded.

---

## 3. Verification & Deliverables

- [x] Removed `relative z-[100]` from `AddAccountDialog.tsx`.
- [x] Navbar elevated to `zIndex: 60`.
- [x] Dropdowns in `NavDropdowns.tsx` given `z-50 max-w-[calc(100vw-32px)]`.
- [x] All quality gates and repository files verified clean.
