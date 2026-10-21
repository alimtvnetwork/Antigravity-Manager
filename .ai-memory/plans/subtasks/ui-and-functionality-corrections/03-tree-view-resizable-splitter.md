# Subtask 03: Expandable & Resizable Prompt Tree View Width

- **Slug**: `tree-view-resizable-splitter`
- **Component**: Prompt Tree Layout & Responsive Splitter
- **Primary Files**:
  - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
  - `src/components/instances/PromptTreeView/TreePanel.tsx`
- **Status**: QUEUED

---

## 1. Problem Statement
The user explicitly stated:
> "In the prompt tree view, the left-hand side, the tree view, I should be able to reduce the width at any time. Try to have that expandable behavior. The UI looks much more compact now compared to the past. It is a good work."

## 2. Technical Requirements
1. **Draggable Resize Splitter**:
   - Add a vertical drag handle between `TreePanel` and the right detail pane.
   - On mouse down, capture mouse movements horizontally and update the tree panel width.
   - Min width: 220px, Max width: 650px (or 50% of viewport).
   - Double-click on splitter to reset to default (e.g. 340px).
2. **Width State Persistence**:
   - Store preferred width in `localStorage` under `agm_prompt_tree_sidebar_width`.
   - Restore on modal open.
3. **Smooth Visual Interaction**:
   - Subtle hover highlight on splitter handle (`hover:bg-purple-500/50` or theme accent).
   - Dragging indicator (`cursor-col-resize`, active color).
   - Ensure child items adjust gracefully with text truncation (`truncate`).

## 3. Verification Criteria
- Dragging the splitter dynamically resizes the tree view.
- Width persists across modal open/close cycles.
