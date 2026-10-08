# Root Cause Analysis: Task 147 - Smart Instance Process Cache & Prompt Dispatch Cleanup

## 1. What Failed?
1. **Prompt Dispatch / Enqueue Failure**:
   - Clicking "Send Now" or pressing 'N' failed to inject the selected prompt to the active IDE session.
   - Clicking "Enqueue" reported failure because `enqueue_prompt` IPC handler was missing in Tauri backend.
2. **Infinite IDE Re-launch Loops**:
   - Every prompt dispatch attempt triggered a full launch of the Antigravity IDE executable, even when the IDE instance was already open and active.
3. **Ghost 'RUNNING' Indicators Across Idle Workspaces**:
   - Multiple idle project nodes showed `1 RUNNING` in green, and the detail inspector showed `RUNNING (PID: 278499) NaNm NaNs`.
4. **Visual Bracket Tag Pollution**:
   - Heavy bracket labels like `[AGM:P006 | GM:#6]` cluttered project tree nodes.

## 2. Root Cause (Why Did It Fail?)
1. **Missing IPC Handlers**:
   - `src/components/instances/PromptTreeViewModal.tsx` contained calls to `invoke('enqueue_prompt', ...)`. However, `enqueue_prompt` was never defined in `src-tauri/src/commands/instance.rs` or registered in `generate_handler![]` in `src-tauri/src/lib.rs`.
   - `handleResendPrompt` was relying on `resume_recent_project_prompts`, which queried old SQLite records instead of dispatching the prompt currently active in the modal.
2. **Unconditional Launch Fallthrough in `focus_or_launch_instance_with_workspace`**:
   - Line 3108 in `src-tauri/src/modules/instance.rs`: When `focus_instance_pids(&pids)` returned `false` (which commonly occurs on Linux without X11 window managers or when window focus fails), the code did not check whether `!pids.is_empty()`. It fell through directly to `launch_instance_with_workspaces(...)`, launching a duplicate IDE process every time.
3. **No Process Liveness Cache**:
   - AGM lacked a smart instance process cache with PID validation. It could not distinguish between a freshly closed instance, an instance with a living background process, or an instance that needed reopening.
4. **Malformed Date Parsing in `PromptTreeViewModal.tsx`**:
   - `last_modified` from backend was formatted as an integer timestamp string or Unix epoch string. `new Date(selectedConversation.last_modified).getTime()` failed and evaluated to `NaN`, leading to `NaNm NaNs`.

## 3. Corrective Measures & Prevention
1. Implement `send_prompt_now` and `enqueue_prompt` as first-class Tauri IPC commands and register them in `src-tauri/src/lib.rs`.
2. Implement `ensure_instance_running_smart`:
   - Checks if instance is already alive via `sysinfo` process table and cached PID.
   - If alive, NEVER re-launch; only focus window.
   - If cached PID is dead, re-scan OS process table for any new PID matching the instance.
   - If and only if confirmed not running anywhere, launch instance, wait for spawn, and cache the new verified PID.
3. Add NaN and negative number guards to `formatDuration` and implement robust timestamp parsing.
4. Simplify `formatDualBadge` to compact, elegant badges without heavy brackets (`[AGM:... | GM:...]`).
5. Prune dead running prompts in `repo_db.rs` to eliminate ghost running badges.
