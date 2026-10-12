# Subtask 05: Prompt Tree Stability, Clipboard Deadlock Resolution & Action Capsule

**Subtask ID:** `subtask-05`  
**Parent Plan:** Supabase Connection & UI/UX Enhancement (`supabase-connection-and-ui-ux-enhancement`)  
**Spec Reference:** `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/04-prompt-tree-stability-spec.md`  
**Status:** READY FOR IMPLEMENTATION  
**Lead Worker:** Worker 02 (UI/UX, Progress Bar & Theming Author)  

---

## 1. Context & Objectives

This subtask resolves critical runtime deadlock hazards during prompt injection and enhances prompt tree interactivity:

1. **Subprocess Deadlock Fix in `copy_to_system_clipboard`:** In `src-tauri/src/modules/repo_db/agy.rs`, take ownership of `child.stdin.take()` and explicitly drop it before calling `child.wait()`. This signals immediate EOF to operating system clipboard utilities (`clip.exe`, `pbcopy`, `wl-copy`/`xclip`), permanently preventing subprocess pipe deadlocks.
2. **Eliminate Uncaught Clipboard Promises:** Guard all `navigator.clipboard.writeText()` invocations across `DetailHeader.tsx`, `InspectorModal.tsx`, and `PromptInstructionView.tsx` with unified try-catch fallbacks via `src/utils/clipboard.ts`, preventing unhandled DOMException crashes when window focus is absent.
3. **agy CLI Path Resolution Ladder Hardening:** Ensure `src-tauri/src/modules/process/locations.rs:368` resolves the `agy` binary through the 6-stage fallback ladder without blocking or panicking.
4. **Sleek Horizontal Loading Progress Bar:** Integrate a 2px indeterminate animated horizontal bar (`from-emerald-500 via-[#1af18d] to-cyan-400`) pinned immediately below `ModalHeader` in `PromptTreeViewModal.tsx` when `isLoading` is true.
5. **Right-Aligned Segmented Action Capsule:** Consolidate Send Now (`N`), Enqueue, destination mode, and suffix controls into a compact, right-aligned segmented pill capsule (`ml-auto`) in `DetailHeader.tsx`.
6. **Preservation of v4.190.0 Invariants:** Strictly protect account switching token lifecycle, FIFO dispatch queries, and sandboxed instance isolation.

---

## 2. Target Files & Concrete Modifications

```
+-------------------------------------------------------+-----------------------------------------------------------+
| Target File Path                                      | Specific Scope of Modification                            |
+-------------------------------------------------------+-----------------------------------------------------------+
| src-tauri/src/modules/repo_db/agy.rs                  | Fix copy_to_system_clipboard: child.stdin.take() + drop   |
|                                                       | before child.wait(); dispatch in background thread        |
| src-tauri/src/modules/process/locations.rs            | Verify and harden get_antigravity_cli_executable_path      |
|                                                       | 6-stage ladder resolution                                 |
| src/components/instances/PromptTreeView/PromptTree    | Add sleek 2px horizontal indeterminate progress bar under |
| ViewModal.tsx                                         | ModalHeader when isLoading is true                        |
| src/components/instances/PromptTreeView/DetailHeader  | Consolidate Send, Enqueue, Suffix, and Mode buttons into  |
| .tsx                                                  | a unified right-aligned segmented pill capsule (ml-auto); |
|                                                       | replace raw clipboard calls with safe copyToClipboard     |
| src/components/instances/PromptTreeView/Prompt        | Replace raw navigator.clipboard calls with                |
| InstructionView.tsx                                   | copyToClipboard or copyPromptWithRichImages               |
| src/components/instances/PromptTreeView/Inspector    | Replace raw navigator.clipboard calls with safe           |
| Modal.tsx                                             | copyToClipboard handler                                   |
| src/utils/clipboard.ts                                | Ensure robust fallback to textarea execCommand('copy')    |
+-------------------------------------------------------+-----------------------------------------------------------+
```

---

## 3. Detailed Implementation Directives

### 3.1 Rust Clipboard Pipe Deadlock Remediation (`src-tauri/src/modules/repo_db/agy.rs`)
Replace the legacy `ref mut stdin` implementation:

```rust
pub fn copy_to_system_clipboard(content: &str) {
    if content.trim().is_empty() {
        return;
    }
    let content_owned = content.to_string();
    
    // Spawn detached thread to avoid stalling caller runtime
    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            if let Ok(mut child) = std::process::Command::new("clip")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = std::io::Write::write_all(&mut stdin, content_owned.as_bytes());
                    drop(stdin); // Explicit drop closes pipe; clip.exe sees EOF and flushes
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
                    drop(stdin); // Explicit drop closes pipe; pbcopy sees EOF and flushes
                }
                let _ = child.wait();
            }
        }
        #[cfg(target_os = "linux")]
        {
            let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
            let binary = if is_wayland { "wl-copy" } else { "xclip" };
            let args: &[&str] = if is_wayland { &[] } else { &["-selection", "clipboard"] };
            if let Ok(mut child) = std::process::Command::new(binary)
                .args(args)
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = std::io::Write::write_all(&mut stdin, content_owned.as_bytes());
                    drop(stdin);
                }
                let _ = child.wait();
            }
        }
    });
}
```

### 3.2 Frontend Uncaught Clipboard Rejection Elimination
In `DetailHeader.tsx`, `InspectorModal.tsx`, and `PromptInstructionView.tsx`:
- Import `{ copyToClipboard }` from `src/utils/clipboard.ts`.
- Replace all unadorned `navigator.clipboard.writeText(str)` calls with `await copyToClipboard(str)`.
- Never leave an unhandled Promise floating without catch handlers.

### 3.3 Sleek Horizontal Loading Bar in `PromptTreeViewModal.tsx`
Add right below `<ModalHeader ... />`:

```tsx
{/* Sleek Indeterminate Progress Bar */}
<div className="h-0.5 w-full bg-slate-200/50 dark:bg-[#171025] overflow-hidden relative shrink-0">
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

### 3.4 Right-Aligned Segmented Action Capsule in `DetailHeader.tsx`
Wrap toolbar controls into a unified segmented pill:

```tsx
<div className="inline-flex items-center rounded-full border border-slate-300 dark:border-[#40285e] bg-slate-100/90 dark:bg-[#171025]/90 p-0.5 backdrop-blur-md shadow-xs ml-auto divide-x divide-slate-300 dark:divide-[#40285e]">
    {/* Suffix Select */}
    {/* Mode Select */}
    {/* Focus IDE Button */}
    {/* Send Now Button (Primary with Hotkey N) */}
    {/* Enqueue Button */}
    {/* Full Inspector Button */}
</div>
```

---

## 4. Verification & Testing Protocol

1. **Deadlock Test:** Inject a large multi-line prompt (>50KB) via `send_prompt_now_for_instance`. Confirm the backend thread returns in <200ms without thread starvation or process hang.
2. **Unfocused Clipboard Test:** Trigger copy actions while clicking outside the window or blurring focus; confirm no uncaught Promise rejection appears in console logs.
3. **Loading Progress Bar Verification:** Open Prompt Tree View with slow network throttling; confirm sleek horizontal bar pulses smoothly below the header.
4. **Action Capsule Alignment:** Verify segmented capsule stays aligned to the right (`ml-auto`) across various modal widths.
5. **Regression Verification:** Confirm that hotkey `N` still triggers Send Now and enqueues to SQLite FIFO table with correct `created_at ASC` sequencing.
