# System Specification: UI and Functionality Corrections

- **Spec Identifier**: `02-spec/21-app/ui-and-functionality-corrections/spec.md`
- **Component**: Prompt Dispatch Pipeline, IDE Workspace State Sync, Tree Panel Layout, Theme Engine, Scoreboard Hover Tooltip
- **Author**: Maintainer Team (@aukgit)
- **Status**: DRAFT / IN PROGRESS
- **Related Plan**: `.ai-memory/plans/ui-and-functionality-corrections.md`

---

## 1. Executive Summary & Problem Classification

This specification defines the corrective architecture and implementation standards for six critical UI and operational capabilities within Antigravity-Manager:

1. **Preemptive "Send" vs. Non-Intrusive "Enqueue"**:
   - Current Defect: Clicking "Send" behaves like an enqueue action or stalls when an active turn is running. Clicking "Enqueue" is non-functional or redundant.
   - Target Architecture: "Send" is an **immediate preemptive interrupt**. If the target instance is actively running a prompt/task, "Send" terminates/cancels that task immediately and injects the new prompt for real-time execution. In contrast, "Enqueue" appends the prompt to the FIFO backlog without interrupting the running task.

2. **Bidirectional IDE Conversation Title Synchronization**:
   - Current Defect: Renaming a conversation title via double-click in AGM updates local SQLite records (`repo_prompts.db`), but does not propagate to the running Antigravity IDE instance or its workspace storage (`state.vscdb` / session transcripts). The IDE continues showing the old title.
   - Target Architecture: AGM establishes a synchronized title write pipeline that updates both the local SQLite cache and the Antigravity IDE workspace state on disk, ensuring strict name parity.

3. **Draggable & Resizable Tree View Splitter**:
   - Current Defect: The left-hand prompt tree width is rigid or fixed, preventing operators from shrinking or widening the panel to accommodate long project titles or narrow viewport constraints.
   - Target Architecture: A responsive vertical drag handle (splitter) separating `TreePanel` and the detail view with bounded limits (220px to 650px) and persistent width saved in `localStorage`.

4. **Dynamic Theme Adaptability & Smooth Non-Displacing Hover Transitions**:
   - Current Defect: The middle view hardcodes dark purple palette values (`#120a1f`, `#40285e`), causing theme disharmony when selecting themes like "One Dark Pro". Card hover effects currently displace or shift element positioning (`scale`, `translate`), causing visual jitter.
   - Target Architecture: Transition to semantic theme tokens (`bg-base-200`, `border-base-300`, `bg-card`) that dynamically adapt to the active theme. Hover states must strictly alter background/border color and contrast using CSS3 animations (`transition: all 0.3s ease`) with **zero pixel displacement**.

5. **Scoreboard Formula Popover Overlap Containment**:
   - Current Defect: Hovering over scoreboard score rows renders a formula calculation tooltip that collides with or hides beneath sticky table headers.
   - Target Architecture: Elevate z-index to top portal layer, reposition tooltip downward or bounded within table container viewports, preventing pointer collision.

6. **End-to-End Verification & Minor Release Ceremony**:
   - Comprehensive test cases covering all 3 core functionality fixes, followed by clean minor version bump (`npm run bump minor`) with strict `@aukgit` attribution.

---

## 2. Technical Requirements & Invariants

### 2.1 Invariant I1: Preemptive Interrupt on Immediate Send
- When `sendPromptNow` / `send_prompt_now` is invoked:
  1. Check if the target instance has an active running process or active turn lock.
  2. If running, invoke task cancellation: terminate the running `agy` worker or signal cancel to the active Antigravity task runner.
  3. Purge intermediate in-flight flags in SQLite `active_prompts`.
  4. Immediately dispatch the new prompt to the target instance via the `agy` CLI or task bridge.
  5. UI reflects immediate dispatch with emerald feedback.
- When `enqueuePrompt` / `enqueue_prompt` is invoked:
  1. Do NOT terminate or interrupt the running instance.
  2. Insert the prompt into SQLite `active_prompts` with status `QUEUED` and strict FIFO sequence (`ORDER BY created_at ASC, id ASC`).
  3. UI reflects queue staging with amber feedback.

### 2.2 Invariant I2: IDE Workspace Title Synchronization
- When `rename_conversation(instance_id, conversation_id, new_title)` executes:
  1. Update SQLite `conversations.title = ?` in `repo_prompts.db`.
  2. Locate target instance workspace state:
     - Antigravity IDE workspace storage database (`state.vscdb`) under `storage.json` / `workspaceStorage`.
     - Update SQLite key `antigravity.chat.session.metadata` or relevant conversation title records.
     - Update `.antigravity_resume_task.json` if matching active session.
  3. Broadcast change to AGM frontend store and invalidate tree cache.

### 2.3 Invariant I3: Non-Displacing Hover Transitions
- No component in `PromptTreeView` may apply `transform: scale(...)`, `translateY(...)`, or margin alterations on `:hover`.
- All hover interactions must use smooth CSS transitions:
  ```css
  transition: background-color 0.3s ease, border-color 0.3s ease, color 0.3s ease;
  ```

---

## 3. Verification & Compliance Contract

1. **Preemptive Send Test**: Run a mock task; invoke Send; verify the previous task is cancelled and the new prompt executes immediately.
2. **Enqueue Staging Test**: Run a mock task; invoke Enqueue; verify previous task continues uninterrupted while the new prompt is recorded in queue.
3. **IDE Title Sync Test**: Rename a conversation; verify both SQLite and IDE workspace storage reflect the new title.
4. **Resizable Splitter Test**: Drag splitter; verify width changes and persists across page reloads.
5. **Theme Audit**: Verify One Dark Pro mode renders neutral dark slate/gray tones without stubborn purple elements.
