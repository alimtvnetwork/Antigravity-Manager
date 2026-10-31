# Architecture Specification: UI and Functionality Corrections

## 1. System Intent & Architecture
This specification governs three mission-critical operational corrections across Antigravity-Manager's Prompt Tree View, IDE State Synchronization, and Theme Engine:
1. **Send vs Enqueue Execution Inversion**:
   - `Send`: Action must pre-empt / abort any running execution on the target instance workspace and immediately dispatch the active prompt turn.
   - `Enqueue`: Preserves current in-flight execution and registers the prompt turn into the FIFO execution queue to be serviced sequentially.
2. **Dynamic Tree View Layout (Expandable / Resizable Panel)**:
   - Provide user-controlled width resizing for the left-hand project tree column (support narrowing down to compact widths or expanding for deep paths) using draggable splitter or collapsible presets.
3. **IDE Workspace State Synchronization on Rename**:
   - Double-clicking to rename a conversation title must persist not only in AGM SQLite (`repo_prompts.db`) and frontend UI state, but must directly synchronize with Antigravity / VS Code workspace storage (`state.vscdb` / transcript metadata) so that the IDE conversation tree reflects the renamed title in real time.
4. **Theme Harmonization & Stable Card Hover**:
   - Modernize the color palette in the middle pane of the Prompt Tree View to dynamically derive styling from the active theme (e.g. One Dark Pro, Dracula, Synthwave) rather than static purple (`#8b5cf6` / `text-purple-400`).
   - Eliminate hover positional shifts (`translate-y`, `scale`), substituting 0.3s–0.5s CSS3 background/border color transitions.
5. **Scoreboard Popover Stacking Isolation**:
   - Prevent Algorithm Score calculation formula tooltips from colliding with or sticking against the sticky table header.

## 2. Blast Radius & Affected Modules
- `src/components/instances/PromptTreeView/` (DetailHeader.tsx, TreePanel.tsx, types.ts, styling)
- `src-tauri/src/modules/repo_db/` (prompts_crud.rs, dispatch.rs, queue.rs, tree.rs)
- `src-tauri/src/commands/repo_prompts.rs` (Tauri commands for send, enqueue, rename_conversation)
- `src/components/accounts/AccountTable/cells/AlgorithmScoreCell.tsx` (Scoreboard tooltip stacking)
- `src/pages/instances/InstanceCard.tsx` (Card hover transition stabilization)
- `src/theme/` and theme class configurations
