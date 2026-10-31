# Learned Memory 30: Prompt Send Preemption, IDE Title Synchronization, Resizable Splitter, and Theme Non-Displacement

- **ID**: `learned-30`
- **Scope**: Prompt dispatch engine, Chrome DevTools Protocol (CDP) preemption, Antigravity Language Server protobuf summary overrides, responsive tree splitters, theme design tokens, and scoreboard portal tooltips.
- **Date**: 2026-10-21
- **Related Spec**: `02-spec/21-app/ui-and-functionality-corrections/`
- **Related Plan**: `.ai-memory/plans/ui-and-functionality-corrections.md`
- **Version Baseline**: `v4.250.0`

---

## 1. Antigravity IDE Composer Execution Semantics: Send vs. Queue

### The Inherent IDE Delivery Strategy
When Antigravity IDE is actively executing a prompt, its internal React component (`main.js` -> `AgentInputBox` -> `E4b`) applies `MessageDeliveryStrategy.WHEN_IDLE`. Pressing Enter or clicking Send while an invocation is active does **not** interrupt execution; instead, it natively appends the message to the bottom queue of the active session.

### The Send (Preempt) Protocol
To achieve true immediate dispatch (Send):
1. **Preemption Cancellation**: Check if the instance webview is currently executing (`isRunning`). If true, trigger cancellation before inserting text:
   - Click Antigravity's native Cancel button: `button[data-tooltip-id="input-send-button-cancel-tooltip"]`, `aria-label="Cancel (Ctrl+D)"`.
   - As a reliable fallback, traverse the React Fiber tree on `[data-testid="send-button"]` or `[aria-label="Message input"]` to locate `fiber.memoizedProps.cancelInvocation()` and invoke it.
2. **Settlement Window**: Sleep 250ms for the worker process to terminate and for the composer UI to settle from running state back to idle.
3. **CLI Worker Purge**: Terminate background CLI workers via `terminate_agy_workers_for_instance(&canonical_inst)` and transition in-flight SQLite rows to `'cancelled'`.
4. **Prompt Injection**: Focus `[aria-label="Message input"]`, insert prompt text via CDP `Input.insertText`, and trigger Send/Enter.

### The Enqueue (FIFO) Protocol
To stage prompts into the queue without interruption:
1. Bypass cancellation (`cancel_running_first = false`).
2. If the IDE is running, insert text and trigger Send/Enter; Antigravity's composer natively queues the prompt behind the active turn.
3. Concurrently save to SQLite `active_prompts` with `status: 'queued'` and strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).

---

## 2. Antigravity Language Server Title Synchronization

### The Protobuf Raw Summary Fallback Invariant
Antigravity's Go Language Server (`language_server.exe`) runs `jetbox_summaries_store.RecordToSummary` to populate in-memory conversation titles:
- If `conversation_summaries.raw_summary` (a serialized Protobuf blob of type `CascadeTrajectorySummary`) is non-null and non-empty, the language server reads the title exclusively from Protobuf Field 1 and Field 15. It **ignores** the SQLite `title` column!
- Only when `raw_summary IS NULL` or has zero length does it execute the fallback:
  ```text
  RecordToSummary: RawSummary is empty for cascade %s (falling back to columns); summary may be lossy until backfilled
  ```
- **The Invariant**: Whenever updating conversation titles in `conversation_summaries.db`, you **MUST** set `raw_summary = NULL` alongside `title = ?1`. Leaving `raw_summary` populated results in the IDE permanently displaying the stale title.
- **Watcher Trigger**: Touch `conversations/<conversation_id>.db` to trigger the language server's `fsnotify` directory watcher (`summaries_store.runWatcher`) so the running IDE reloads the title immediately without restart.
- **Prompt Backups Sync**: Update `prompt_backups SET conversation_name = ?1 WHERE conversation_id = ?2` in `backup_prompts.db` to prevent post-switch rollbacks.

---

## 3. Responsive Tree Panel Splitter Architecture

In `PromptTreeViewModal.tsx`:
- Never lock column widths using rigid Tailwind classes (`w-[420px] lg:w-[480px] shrink-0`).
- Manage width via dynamic state `treeWidth` bounded between `280px` and `760px` (default `450px`).
- Mount a vertical draggable splitter handle (`role="separator"`, `cursor-col-resize`) listening on window `mousemove` and `mouseup` with text-selection and pointer-event protection.
- Persist state to `localStorage` under `agm_prompt_tree_panel_width` and provide double-click reset to `450px`.

---

## 4. Theme Token Synchronization & Non-Displacing Hover Transitions

- **Eliminate Hardcoded Purple**:
  - Replace `#120a1f`, `#171025`, `#40285e` across Prompt Tree views with theme-adaptive tokens (`dark:bg-slate-900`, `dark:border-slate-800`, `dark:hover:bg-slate-800/80`).
  - In `ThemeManager.tsx`, synchronize `--ui-*` variables (`--ui-canvas`, `--ui-surface-1`, `--ui-surface-hover`, `--ui-border-subtle`, `--ui-interactive`) on every theme change.
  - In `App.css`, explicitly map legacy hex classes under `.theme-onedarkpro` to One Dark Pro charcoal colors (`#21252b`, `#282c34`, `#181a1f`, `#3a3f4b`).
- **Zero Layout Displacement Invariant**:
  - Never apply positional shifts (`hover:-translate-y-*`, `hover:scale-*`, dynamic margin/padding) on cards or list items. They cause sub-pixel rendering blur and layout reflow.
  - Apply strictly CSS3 color, illumination, and contrast transitions: `transition-all duration-300 ease-in-out hover:brightness-105 hover:shadow-md`.

---

## 5. Portal Layering for Sticky Table Tooltips

In `AlgorithmScoreCell.tsx`:
- Tooltips rendered inside `<td>` elements risk clipping and header overlap due to parent `overflow-auto` containers and `sticky` table headers.
- Always portal floating breakdown cards directly to `document.body` via `createPortal` with `fixed` coordinates and `z-[99999]`.
- Enforce downward positioning (`top = rect.bottom + 8`) by default, only flipping upward when `spaceAbove >= 340px`.
- Add `pointer-events-none` so cursor movement between rows is never trapped.
