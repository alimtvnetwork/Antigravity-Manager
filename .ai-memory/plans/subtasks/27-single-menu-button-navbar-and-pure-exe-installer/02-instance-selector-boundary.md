# Subtask 02: Instance Selector Boundary & Space Utilization

> **Parent Plan:** `.ai-memory/plans/pending/27-single-menu-button-navbar-and-pure-exe-installer.md`
> **Target Files:** `src/components/navbar/Navbar.tsx`, `src/components/navbar/InstanceSelector.tsx`

## Objectives
1. Verify `Navbar.tsx` flex layout provides clean whitespace between left Logo, center Menu button, and right Instance Selector / Settings.
2. Confirm `shrink-0` ensures `<InstanceSelector />` is never compressed, truncated, or pushed off-screen.
3. Validate responsive visibility at 1024x700 window size and below.
