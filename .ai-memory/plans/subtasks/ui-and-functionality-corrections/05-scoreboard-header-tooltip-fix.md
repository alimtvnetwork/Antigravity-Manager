# Subtask 05: Scoreboard Header Tooltip Collision & Overlap Fix

- **Slug**: `scoreboard-header-tooltip-fix`
- **Component**: Scoreboard Formula Tooltip & Popover Layer
- **Primary Files**:
  - `src/components/instances/auto-switcher/Scoreboard.tsx` (or relevant Scoreboard component)
- **Status**: QUEUED

---

## 1. Problem Statement
The user explicitly stated:
> "For the scoreboard, if I hover over to the above level, some of the header actually gets stuck with the above level header, as you can see in the screenshot I have given you. Make sure you fix all these things."

The user provided screenshot shows:
Hovering over the score row shows:
`Formula: Score = (S_active * M_tier * Q_weekly) / 1000 ... Disqualification: In Use`
The tooltip overlaps/collides awkwardly with the row header or table container above it.

## 2. Technical Requirements
1. **Locate Tooltip / Popover**:
   - Locate the formula hover card / tooltip in the Scoreboard component.
2. **Prevent Stuck / Header Overlap**:
   - Ensure the tooltip has proper portal rendering or positioning (`top-full mt-2` or dynamic boundary flipping, `z-[9999]`).
   - Add pointer-events handling so moving the mouse doesn't get snagged by overlapping parent header elements.
   - Constrain within viewport boundaries.

## 3. Verification Criteria
- Scoreboard formula popover displays without getting trapped or stuck behind/over sticky headers.
