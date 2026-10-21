# Component Specification: UI and Functionality Corrections

## 1. DetailHeader Action Buttons (Send & Enqueue)
- **Send Button**:
  - Semantic: Direct immediate dispatch.
  - Behavior:
    1. Checks if instance/project has an active prompt running.
    2. If running, triggers preemptive cancellation via `cancel_running_prompt` or abort IPC.
    3. Triggers immediate dispatch (`send_prompt_to_instance`).
    4. Icon: Direct Play / Send icon (Emerald / Sky tone).
- **Enqueue Button**:
  - Semantic: Deferred sequential execution.
  - Behavior:
    1. Appends prompt to SQLite FIFO queue (`active_prompts` with status `queued`).
    2. Does NOT interrupt or abort currently executing processes.
    3. Icon: Queue / ListPlus icon (Amber tone).

## 2. TreePanel Left Column Resizing
- Current fixed or min-width layout in `TreePanel.tsx` / `PromptTreeViewModal.tsx` replaced or enhanced with dynamic width state:
  - Default width: `360px` (or `24rem`).
  - Min width: `220px`.
  - Max width: `600px`.
  - Persistent resize drag-handle divider between TreePanel and DetailView.
  - Double-click divider to toggle between compact (`240px`) and default (`360px`).

## 3. Conversation Rename Sync
- When `rename_conversation(conversationId, newTitle)` is triggered:
  - Update `repo_prompts.db` via `repo_db::rename_conversation`.
  - Update IDE workspace storage:
    - Locate the active workspace storage `state.vscdb` for the conversation's project.
    - Update `antigravity.chat.sessions` or `workbench.panel.chatSidebar` or relevant key in SQLite `state.vscdb` where title is stored.
    - Emit Tauri notification / state change event so IDE reads the updated title.

## 4. Theme Harmonization & Non-Moving Card Hover
- Replace hardcoded `purple-500` / `purple-400` / `border-purple-800` in middle pane with theme-aware tokens (`text-primary`, `bg-base-200`, `border-base-300`, or theme accent variables).
- In `InstanceCard.tsx`:
  - Remove `hover:-translate-y-1` or `transform transition-transform`.
  - Apply `transition-colors duration-300 ease-in-out` with subtle contrast adjustment (`hover:bg-opacity-80 hover:border-slate-500`).

## 5. AlgorithmScoreCell Popover Stacking
- Ensure score popover renders via Portal or with elevated `z-[9999]` and `overflow-visible` container, preventing sticky header collision.
