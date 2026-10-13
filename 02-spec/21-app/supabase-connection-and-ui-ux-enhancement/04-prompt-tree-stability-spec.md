# Prompt Tree Stability, Clipboard Concurrency & Action Capsule Specification

**Document ID:** `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/04-prompt-tree-stability-spec.md`  
**Parent Epic:** Supabase Connection & UI/UX Enhancement  
**Status:** DRAFT / APPROVED FOR IMPLEMENTATION  
**Target Version:** v4.191.0+  
**Target Modules & Components:**
- `src-tauri/src/modules/repo_db/agy.rs`
- `src-tauri/src/modules/process/locations.rs`
- `src-tauri/src/commands/cmd_paths.rs`
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- `src/components/instances/PromptTreeView/ModalHeader.tsx`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src/components/instances/PromptTreeView/InspectorModal.tsx`
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src/utils/clipboard.ts`

---

## 1. Executive Summary & Problem Classification

The Prompt Tree View system in Antigravity-Manager serves as the operational nerve center for viewing multi-instance project trees, inspecting prompt origin tiers, triggering immediate prompt re-injection to IDE windows, and enqueuing tasks into the FIFO scheduler.

Two critical stability defects and two key UI ergonomics opportunities have been identified:

1. **Subprocess Pipe Deadlock in `copy_to_system_clipboard`:** In `src-tauri/src/modules/repo_db/agy.rs`, the system clipboard helper spawns native OS clipboard executables (`clip.exe` on Windows, `pbcopy` on macOS, `wl-copy`/`xclip` on Linux) with a piped `stdin`. The implementation kept `child.stdin` open while calling `child.wait()`. Because OS clipboard utilities read until receiving an EOF (End-of-File) marker before terminating, leaving `stdin` open causes the child process to block waiting for EOF while the parent blocks on `child.wait()`, resulting in an unrecoverable thread freeze during prompt dispatch.
2. **Uncaught Clipboard Promise Exceptions:** Multiple frontend components (`DetailHeader.tsx`, `InspectorModal.tsx`, `PromptInstructionView.tsx`) invoke `navigator.clipboard.writeText()` directly without `.catch()` or `await` handlers. When the webview or window loses user focus or operating system permissions are denied, the browser throws an unhandled `NotAllowedError: Document is not focused` exception, crashing subsequent UI event callbacks.
3. **Missing Loading Progress Bar:** When loading or synchronizing large conversation trees across multiple project workspaces, `PromptTreeViewModal.tsx` provides no horizontal visual feedback during async fetches, causing users to perceive the interface as unresponsive.
4. **Scattered Action Controls:** Send Now, Enqueue, destination modes, and confirmation suffixes are laid out loosely across the detail view header, consuming excessive horizontal width and lacking capsule segmentation.
5. **v4.190.0 Stability Invariant Preservation:** All improvements must preserve the hardened core invariants: atomic OAuth refresh, 98% threshold triggers, active window restoration, candidate scoring algorithms, and secure credential handling.

---

## 2. Root Cause Analysis: Clipboard Subprocess Pipe Deadlock

### 2.1 The Pipe Deadlock Phenomenon

In `src-tauri/src/modules/repo_db/agy.rs`, prompt execution dispatches via `send_prompt_now_for_instance` and `enqueue_prompt_for_instance` call `copy_to_system_clipboard(prompt_content)`.

The legacy implementation was structured as:

```rust
// DEADLOCK DEFECT IN agy.rs
pub fn copy_to_system_clipboard(content: &str) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(mut child) = std::process::Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                let _ = std::io::Write::write_all(stdin, content.as_bytes());
            }
            // DEADLOCK: child.stdin is still held open by the parent!
            // clip.exe never receives EOF and waits indefinitely.
            // Parent blocks forever on child.wait().
            let _ = child.wait();
        }
    }
}
```

### 2.2 Mechanism of Failure
1. `std::process::Command::new("clip").stdin(Stdio::piped()).spawn()` creates an OS anonymous pipe between the Antigravity-Manager parent process and `clip.exe`.
2. `write_all(stdin, content.as_bytes())` sends the bytes through the pipe buffer.
3. `ref mut stdin` borrows the stdin file descriptor without closing it.
4. `clip.exe` reads the data into memory, but per Win32 pipe semantics, does not flush to the clipboard or terminate until the pipe handle is closed (`EOF`).
5. The parent thread calls `child.wait()`, blocking until `clip.exe` exits.
6. Since `child.stdin` is still open in the parent, the pipe never closes, `clip.exe` never exits, and the parent process deadlocks.
7. Identical failure modes occur on macOS (`pbcopy`) and Linux (`wl-copy` / `xclip`).

### 2.3 The Architectural Fix: Explicit `stdin.take()` and Drop Before `wait()`

To permanently eliminate this deadlock class:
1. Use `child.stdin.take()` to take complete ownership of the `ChildStdin` handle out of the `Child` struct.
2. Write content to the taken stdin stream.
3. Explicitly call `drop(stdin)` (or allow it to fall out of scope) before invoking `child.wait()`. Dropping the handle sends EOF across the operating system pipe immediately.
4. Enforce a defensive non-blocking timeout or spawn in a detached background thread so clipboard operations can never block main or async runtime threads.

```rust
// CANONICAL CORRECT IMPLEMENTATION (src-tauri/src/modules/repo_db/agy.rs)
pub fn copy_to_system_clipboard(content: &str) {
    if content.trim().is_empty() {
        return;
    }
    
    let content_owned = content.to_string();
    
    // Dispatch in dedicated detached worker to ensure zero latency on caller thread
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            if let Ok(mut child) = std::process::Command::new("clip")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = std::io::Write::write_all(&mut stdin, content_owned.as_bytes());
                    drop(stdin); // Explicitly close pipe to deliver EOF to clip.exe
                }
                let _ = child.wait();
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(mut child) = std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = std::io::Write::write_all(&mut stdin, content_owned.as_bytes());
                    drop(stdin); // Explicitly close pipe to deliver EOF to pbcopy
                }
                let _ = child.wait();
            }
        }
        #[cfg(target_os = "linux")]
        {
            let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
            let binary = if is_wayland { "wl-copy" } else { "xclip" };
            let args = if is_wayland { vec![] } else { vec!["-selection", "clipboard"] };
            
            if let Ok(mut child) = std::process::Command::new(binary)
                .args(&args)
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = std::io::Write::write_all(&mut stdin, content_owned.as_bytes());
                    drop(stdin); // Explicitly close pipe to deliver EOF to Linux helper
                }
                let _ = child.wait();
            }
        }
    });
}
```

---

## 3. Frontend Clipboard Exceptions & agy CLI Path Ladder

### 3.1 Uncaught Clipboard Promises Resolution
Direct calls to `navigator.clipboard.writeText(...)` without error handling fail whenever the webview does not have active window focus or OS clipboard permissions are throttled.

#### Concrete Violations in Existing Code:
- `src/components/instances/PromptTreeView/DetailHeader.tsx:119`
- `src/components/instances/PromptTreeView/InspectorModal.tsx:50`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx:105, 148`

#### Unified Safe Clipboard Protocol:
All copy operations must route through `copyToClipboard` in `src/utils/clipboard.ts` or `copyPromptWithRichImages` in `src/components/instances/PromptTreeView/clipboard.ts`.

Both helpers implement:
1. `navigator.clipboard.writeText` protected by `try/catch`.
2. Fallback to hidden `document.execCommand('copy')` with `textarea` DOM injection when the Clipboard API is unavailable or rejected.
3. Guaranteed boolean return (`Promise<boolean>`) with zero unhandled exceptions.

### 3.2 agy CLI Path Resolution Ladder

When launching prompts via the native CLI, the resolution ladder in `src-tauri/src/modules/process/locations.rs:368` must execute strictly in hierarchical sequence:

```
+-------------------------------------------------------------------------------+
| Stage 1: Explicit User Configuration                                         |
| -> config.antigravity_cli_executable (verified via Path::exists)              |
+-------------------------------------------------------------------------------+
                                      │ (if None or not found)
                                      ▼
+-------------------------------------------------------------------------------+
| Stage 2: Standard User Local Bin                                              |
| -> dirs::home_dir()/.local/bin/agy[.exe]                                      |
+-------------------------------------------------------------------------------+
                                      │ (if not found)
                                      ▼
+-------------------------------------------------------------------------------+
| Stage 3: Windows LocalAppData Bin                                             |
| -> %LOCALAPPDATA%/agy/bin/agy.exe (Windows only)                              |
+-------------------------------------------------------------------------------+
                                      │ (if not found)
                                      ▼
+-------------------------------------------------------------------------------+
| Stage 4: Gemini Antigravity Bin                                               |
| -> ~/.gemini/antigravity/bin/agy[.exe]                                        |
+-------------------------------------------------------------------------------+
                                      │ (if not found)
                                      ▼
+-------------------------------------------------------------------------------+
| Stage 5: System Environment PATH Resolution                                   |
| -> Iteration through std::env::split_paths(&PATH)                             |
+-------------------------------------------------------------------------------+
                                      │ (if not found)
                                      ▼
+-------------------------------------------------------------------------------+
| Stage 6: Discovered Active IDE Directory                                      |
| -> Parent / sibling binary search relative to detected IDE install root       |
+-------------------------------------------------------------------------------+
```

---

## 4. Sleek Horizontal Loading Progress Bar

### 4.1 UI Design in `PromptTreeViewModal.tsx`

When asynchronous tree loading or syncing is active (`isLoading === true`), the modal must render a sleek, high-visibility 2px horizontal loading indicator immediately below `ModalHeader`:

```tsx
{/* Sleek Horizontal Loading Bar */}
<div className="h-0.5 w-full bg-slate-200 dark:bg-[#171025] overflow-hidden relative shrink-0">
    {isLoading && (
        <div
            className="h-full bg-gradient-to-r from-emerald-500 via-[#1af18d] to-cyan-400 absolute inset-0 animate-pulse"
            style={{
                animation: 'agm-progress-indeterminate 1.5s cubic-bezier(0.4, 0, 0.2, 1) infinite',
            }}
        />
    )}
</div>
```

Key attributes:
- **Height:** Exactly `h-0.5` (2px) to remain non-intrusive.
- **Palette:** Blends from emerald `#31c9a4` through neon terminal green `#1af18d` to cyan `#38bdf8`.
- **Transitions:** Seamless 150ms opacity transition when tree sync starts and finishes.

---

## 5. Right-Aligned Compact Segmented Action Capsule

### 5.1 Ergonomic Compacting in `DetailHeader.tsx`

The toolbar controls in `src/components/instances/PromptTreeView/DetailHeader.tsx` must be unified into a single, cohesive, segmented pill capsule positioned on the far right (`ml-auto`):

```
+---------------------------------------------------------------------------------------------------+
|  [Suffix: None ▾] | [Current Conv ▾] | [ 🗖 IDE ] | [ ↻ Send  N ] | [ + Queue ] | [ 🔍 Inspect ]   |
+---------------------------------------------------------------------------------------------------+
```

### 5.2 Capsule Construction Rules
1. **Outer Capsule Container:**
   - Classes: `inline-flex items-center rounded-full border border-slate-300 dark:border-[#40285e] bg-slate-100/90 dark:bg-[#171025]/90 p-0.5 backdrop-blur-md shadow-xs ml-auto`
2. **Dividers:**
   - Subtle vertical divider between segments: `h-3.5 w-px bg-slate-300 dark:bg-[#40285e] my-auto`
3. **Dropdown Selects:**
   - Must use borderless transparent styling (`bg-transparent border-0 focus:ring-0 focus:outline-none text-[10.5px] font-semibold text-slate-700 dark:text-slate-200 cursor-pointer pr-1 py-0`) to eliminate double pill nesting defects.
4. **Action Buttons:**
   - **Send Now (Primary):** Styled with high-contrast emerald/mint accents (`text-emerald-700 dark:text-emerald-300 hover:bg-emerald-500/15 active:scale-[0.98]`), with an embedded `<kbd>` badge for hotkey `N`.
   - **Enqueue (FIFO):** Styled with purple/violet accents (`text-purple-700 dark:text-purple-300 hover:bg-purple-500/15`).
   - **Focus IDE:** Styled with sky-blue accents (`text-sky-700 dark:text-sky-300 hover:bg-sky-500/15`).
   - **Inspect:** Styled with cyan accents (`text-cyan-700 dark:text-cyan-300 hover:bg-cyan-500/15`).

---

## 6. Preservation of v4.190.0 Stability Invariants

All changes introduced by this specification must strictly preserve core architecture invariants established in v4.190.0:

1. **Atomic Token Refresh & Quota Scoring:** Never bypass Google OAuth token refresh or alter 5h rolling vs 7-day weekly quota weightings.
2. **FIFO Dispatch Order Invariant:** All SQLite queries (`active_prompts`, `prompt_backups`, `repo_prompts.db`) must enforce `ORDER BY created_at ASC, id ASC`. LIFO inverted ordering is strictly forbidden.
3. **Process Liveness & Job Breakaway:** Multi-instance process launching must preserve `spawn_with_job_breakaway_fallback`.
4. **State Isolation:** Never leak cross-instance prompt state or bypass workspace directory boundaries.

---

## 7. Verification Checklist & Success Criteria

- [ ] `copy_to_system_clipboard` in `agy.rs` uses `child.stdin.take()` and drops `stdin` before `child.wait()`.
- [ ] No thread or process deadlock occurs when dispatching prompts with large payloads.
- [ ] Zero unhandled Promise rejections from `navigator.clipboard.writeText` across all Prompt Tree components.
- [ ] agy CLI executable path ladder traverses all 6 stages reliably.
- [ ] Sleek 2px horizontal progress bar renders during `isLoading` in `PromptTreeViewModal.tsx`.
- [ ] Action buttons in `DetailHeader.tsx` are consolidated into a right-aligned segmented pill capsule.
- [ ] Existing v4.190.0 automated test gates continue to pass without regression.

## 8. Send and enqueue must not kill the window

`send_prompt_now` and `enqueue_prompt` in `src-tauri/src/commands/instance.rs` run the external agy and clipboard work inside `catch_unwind`. A panic returns an error string that includes `Backtrace::force_capture` instead of tearing down the webview. The frontend stores that error in `localStorage` key `agm.lastClientError` and replays it on the next launch.

Instance quota bars in `src/components/instances/table/InstanceRowCells.tsx` fill a column at least 280px wide. The track keeps `min-w-[140px]`. The healthy stop stays `#1af18d`. The update card in `src/components/UpdateNotification.tsx` uses the same green.
