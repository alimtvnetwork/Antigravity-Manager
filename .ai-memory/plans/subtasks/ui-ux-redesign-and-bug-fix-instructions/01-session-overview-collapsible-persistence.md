# Subtask 01: Session Overview Collapsible & State Persistence

**File**: `.ai-memory/plans/subtasks/ui-ux-redesign-and-bug-fix-instructions/01-session-overview-collapsible-persistence.md`  
**Parent Plan**: `.ai-memory/plans/ui-ux-redesign-and-bug-fix-instructions.md`  
**Status**: Pending  

## Objectives
1. In `src/components/instances/PromptTreeView/PromptInstructionView.tsx`:
   - Wrap the session overview section (turns, status, time range, chronological chips) into a collapsible container.
   - Default to collapsed (`isExpanded = false`).
   - Add a subtle purple arrow button (`ChevronDown` when expanded, `ChevronRight` when collapsed) with subtle styling (`text-purple-400/70 hover:text-purple-300`).
   - Persist state in `localStorage` under `agm_prompt_tree_session_overview_expanded`.
   - Hide cryptic `#C211` code and redundant turn headers from the main overview, moving deep technical metadata to the Details modal.
