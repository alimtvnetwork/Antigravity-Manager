---
name: ui-and-functionality-corrections
description: Autonomously diagnose, repair, and verify Prompt Tree View Send/Enqueue mechanics, expandable/resizable tree panels, conversation title rename synchronization with IDE, theme adaptability without purple tints, card hover stability, and scoreboard popover stacking.
---

# UI and Functionality Corrections

## Core Responsibilities & Invariants
1. **Send vs Enqueue Execution Semantics**:
   - `Send`: Immediately preempts/cancels any actively running execution on the target project/instance and executes the selected prompt turn right now.
   - `Enqueue`: Preserves current running activity and appends the selected prompt turn to the execution queue (what previously was assigned to Send).
2. **Resizable / Expandable Tree View Panel**:
   - The left-hand Prompt Tree View column must support adjusting width (collapsing/reducing width on demand or dragging divider) so users can optimize workspace area.
3. **Conversation Rename Sync with IDE**:
   - Double-clicking to rename a conversation title in AGM Prompt Tree must synchronize to the IDE workspace storage (`state.vscdb`, transcripts, or workspace storage database) so that the IDE reflects the exact updated title immediately.
4. **Theme Adaptability & Non-Intrusive Card Hover**:
   - Themes such as One Dark Pro must dynamically adapt middle panel accents, eliminating hardcoded purple tints.
   - Cards must NEVER shift layout, transform position, or jitter on hover; transitions must be pure CSS3 color/background/contrast transitions over 0.3s–0.5s.
5. **Scoreboard Tooltip / Popover Stacking**:
   - Formula tooltips in the scoreboard must avoid overlapping or getting stuck behind/against the sticky header level.
6. **Coding Guideline Audit & Adherence**:
   - Maintain strict accounting of repository coding guidelines, reporting precise compliance percentages and actionable areas for improvement.
