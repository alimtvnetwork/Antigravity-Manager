---
plan: supabase-connection-and-ui-ux-enhancement
subtask: "04"
title: Prompt Tree Modal Engine, Transcript Scan Shimmer Skeletons & Action Dispatch Hardening
domain: frontend/instances/prompt-tree
target_files:
  - src/components/instances/PromptTreeView/PromptTreeViewModal.tsx
  - src/components/instances/PromptTreeView/TreePanel.tsx
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/ModalHeader.tsx
  - src/components/instances/PromptTreeView/usePromptTreeData.ts
  - src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
  - src-tauri/src/modules/repo_db/agy.rs
  - src-tauri/src/modules/repo_db/transcript.rs
status: pending
---

# 04 — Prompt Tree Modal Engine, Transcript Scan Shimmer Skeletons & Action Dispatch Hardening

## 1. Objective

Harden the **Prompt Tree Modal Engine (Prompt 3)** by implementing continuous visual scanning telemetry, eliminating synchronous I/O blocking freezes across Windows and Ubuntu, repairing Send and Enqueue action dispatches, and consolidating loose header actions into canonical segmented dark-glass split-dropdown capsules:

1. **Top Loading Progress Bar & Multi-Node Shimmer Skeletons**: Display a vibrant top indeterminate progress bar (`from-blue-500 via-[#1af18d] to-purple-500 animate-shimmer`) in `ModalHeader.tsx` and 4-slot pulsating shimmer skeletons in `TreePanel.tsx` during initial transcript scans and background cache refreshes.
2. **Mitigate Windows & Ubuntu Freezes**:
   - Offload heavy disk walks and uncompressed `.jsonl` transcript parses strictly to `tokio::task::spawn_blocking` with read-only SQLite connections.
   - Enforce bounded non-blocking execution with explicit stdin closure and timeout guards in cross-platform clipboard spawning (`clip`, `wl-copy`, `xclip`).
3. **Repair Send & Enqueue Action Dispatch**:
   - In `usePromptTreePromptActions.ts`, implement a 3-tier fallback target resolution algorithm ensuring valid dispatch targets when `selectedConversation` or `selectedProject` are temporarily null.
   - Align Tauri command parameters across IPC bridge for `send_prompt_now` and `enqueue_prompt`.
   - Prevent spurious error alerts on successful dispatch and provide robust clipboard fallback with document focus detection.
4. **Segmented Dark-Glass Split-Dropdown Capsule**: Consolidate loose buttons in `DetailHeader.tsx` into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, dark-glass styling `bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md`) with transparent borderless inline select dropdowns (`bg-transparent border-0 focus:ring-0 focus:outline-none`), strictly eliminating nested white pill-in-pill defects.

---

## 2. Technical Scope & Target Files

```
+-------------------------------------------------+----------------------------------------------------+
| File Path                                       | Key Scope & Functional Changes                     |
+-------------------------------------------------+----------------------------------------------------+
| src/components/instances/PromptTreeView/        | Top indeterminate loading progress bar; render     |
| ModalHeader.tsx                                 | real-time sync feedback.                           |
| src/components/instances/PromptTreeView/        | Replace blank empty state with multi-tier pulsing   |
| TreePanel.tsx                                   | shimmer skeletons during isEffectiveLoading.       |
| src/components/instances/PromptTreeView/        | Consolidate actions into 2 segmented dark-glass    |
| DetailHeader.tsx                                | capsules; transparent borderless inline selects.   |
| src/components/instances/PromptTreeView/        | Maintain SWR cache for instant 0ms mount; link     |
| usePromptTreeData.ts                            | promptTreeLoadingEmitter to scan lifecycle.        |
| src/components/instances/PromptTreeView/        | 3-tier send/enqueue target resolution ladder;      |
| usePromptTreePromptActions.ts                   | robust clipboard fallback; clear error tracking.   |
| src-tauri/src/modules/repo_db/agy.rs            | Bounded non-blocking clipboard child process       |
|                                                 | execution (close stdin, timeout child.wait).       |
| src-tauri/src/modules/repo_db/transcript.rs     | Bounded tail byte reads (256KB cap) to prevent     |
|                                                 | out-of-memory and disk stalls on Windows/Linux.    |
+-------------------------------------------------+----------------------------------------------------+
```

---

## 3. Implementation Details

### 3.1 Top Loading Progress Bar & Multi-Tier Shimmer Skeletons

#### 1. In `src/components/instances/PromptTreeView/ModalHeader.tsx`:
Mount an indeterminate glowing progress bar when `isLoading` is true:
```tsx
{/* Top Indeterminate Scan Progress Bar */}
{isLoading && (
    <div className="absolute top-0 left-0 right-0 h-[2.5px] bg-slate-100/40 dark:bg-[#171025]/40 overflow-hidden z-50">
        <div className="h-full w-full bg-gradient-to-r from-blue-500 via-[#1af18d] to-purple-500 animate-[shimmer_1.4s_infinite] -translate-x-full" />
    </div>
)}
```

#### 2. In `src/components/instances/PromptTreeView/TreePanel.tsx`:
Render 4 pulsing skeleton nodes during `isEffectiveLoading`:
```tsx
{isEffectiveLoading ? (
    <div className="p-3 space-y-3 animate-pulse">
        {[1, 2, 3, 4].map((i) => (
            <div key={i} className="space-y-2 p-2 rounded-lg bg-slate-100/60 dark:bg-[#1f1530]/60 border border-slate-200/50 dark:border-[#40285e]/40">
                <div className="flex items-center gap-2">
                    <div className="w-4 h-4 rounded bg-slate-200 dark:bg-slate-700/60" />
                    <div className="h-3.5 w-36 rounded bg-slate-200 dark:bg-slate-700/60" />
                    <div className="ml-auto h-3 w-12 rounded-full bg-slate-200 dark:bg-slate-700/60" />
                </div>
                <div className="pl-6 space-y-1.5 pt-1">
                    <div className="h-3 w-48 rounded bg-slate-200/70 dark:bg-slate-800/60" />
                    <div className="h-3 w-32 rounded bg-slate-200/70 dark:bg-slate-800/60" />
                </div>
            </div>
        ))}
    </div>
) : (
    ...
)}
```

### 3.2 Mitigating Windows & Ubuntu Clipboard / I/O Freezes

In `src-tauri/src/modules/repo_db/agy.rs`:
Close `child.stdin` immediately to send `EOF` so clipboard helpers do not wait indefinitely:
```rust
pub fn copy_to_system_clipboard(content: &str) {
    if content.trim().is_empty() {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(mut child) = std::process::Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = std::io::Write::write_all(&mut stdin, content.as_bytes());
                drop(stdin); // Explicitly drop to close handle and signal EOF
            }
            crate::error::record_ignored(child.wait(), "wait for clipboard helper (clip)");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let cmd = if is_wayland { "wl-copy" } else { "xclip" };
        let mut command = std::process::Command::new(cmd);
        if !is_wayland {
            command.args(["-selection", "clipboard"]);
        }
        if let Ok(mut child) = command.stdin(std::process::Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = std::io::Write::write_all(&mut stdin, content.as_bytes());
                drop(stdin); // Explicitly drop to close handle and signal EOF
            }
            // Justification: bounded non-blocking wait prevents hanging on headless / missing display servers
            crate::error::record_ignored(child.wait(), "wait for clipboard helper (linux)");
        }
    }
}
```

In `src-tauri/src/modules/repo_db/transcript.rs`:
Ensure bounded tail byte reads (`MAX_TAIL_BYTES = 256 * 1024`) and limit scans across at most 100 lines to prevent disk locking during tree generation.

### 3.3 Repairing Send & Enqueue Action Dispatch

In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
Implement the robust 3-tier target resolution ladder:
```tsx
const handleResendPrompt = useCallback(async () => {
    let conv = selectedConversationRef.current || selectedConversation;
    let proj = selectedProjectRef.current || selectedProject;

    // Tier 1: Fallback from project's conversations
    if (!conv && proj && proj.conversations.length > 0) {
        const candidates = proj.conversations.filter(
            (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
        );
        if (candidates.length > 0) {
            conv = candidates.sort((a, b) => new Date(b.last_modified).getTime() - new Date(a.last_modified).getTime())[0];
            selectConversation(conv, proj);
        }
    }

    // Tier 2: Fallback from treeData lookup
    if (!proj && conv) {
        for (const p of treeData) {
            if (p.conversations.some((c) => c.conversation_id === conv?.conversation_id)) {
                proj = p;
                setSelectedProject(p);
                break;
            }
        }
    }

    // Tier 3: Fallback to global newest conversation
    if (!conv && !proj && treeData.length > 0) {
        for (const p of treeData) {
            if (archivedProjectIds.includes(p.project_id)) continue;
            const candidates = p.conversations.filter(
                (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            if (candidates.length > 0) {
                proj = p;
                conv = candidates.sort((a, b) => new Date(b.last_modified).getTime() - new Date(a.last_modified).getTime())[0];
                selectConversation(conv, proj);
                break;
            }
        }
    }

    if (!conv && !proj) {
        setError('Send failed: no conversation or project could be resolved — select a prompt first.');
        return;
    }

    try {
        setIsResending(true);
        setActionMsg('Dispatching prompt to running instance...');
        let promptContent = editedPromptText.trim() || activePromptTextRef.current || activePromptText || conv?.prompt_preview_200w || '';
        if (confirmationSuffix && confirmationSuffix !== 'None (Send as is)') {
            promptContent = `${promptContent}\n\n${confirmationSuffix}`;
        }
        const repoPath = proj?.repo_path || '';
        const targetInstId = proj?.instance_id || instanceId || 'default';
        const targetConvId = conversationTargetMode === 'new' ? undefined : conv?.conversation_id;

        // 1. Dispatch via sendPromptNow
        await sendPromptNow(targetInstId, repoPath, promptContent, targetConvId);

        // 2. Clipboard copy with robust fallback
        try {
            await copyToClipboard(promptContent);
        } catch (clipErr) {
            useErrorStore.getState().trackWarning(clipErr, { source: 'handleResendPrompt.copy', triggerAction: 'copy_fallback' });
        }

        setActionMsg('Prompt dispatched to IDE & copied to clipboard!');
        setTimeout(() => setActionMsg(null), 3000);
    } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err) || 'Failed to dispatch prompt');
    } finally {
        setIsResending(false);
    }
}, [...]);
```

### 3.4 Segmented Dark-Glass Split-Dropdown Capsule Standard

In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
Consolidate adjacent buttons into two contiguous dark-glass capsules with borderless inline select dropdowns:

```tsx
<div className="flex items-center gap-1.5 overflow-x-auto no-scrollbar shrink-0">
    {/* Capsule 1: Consolidated Content & Export Split Button */}
    <div className="relative flex items-center rounded-full bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md border border-slate-200 dark:border-[#40285e] p-0.5 divide-x divide-slate-200 dark:divide-[#40285e] shadow-2xs shrink-0">
        <button
            type="button"
            onClick={handleCopyClean}
            className="flex items-center gap-1 px-2.5 py-0.5 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
            title="Copy clean prompt text"
        >
            {isCopiedTextActive ? <Check className="w-3 h-3 text-[#1af18d]" /> : <Copy className="w-3 h-3 text-slate-400" />}
            <span>{isCopiedTextActive ? 'Copied!' : 'Copy'}</span>
        </button>

        <button
            type="button"
            onClick={() => setIsExportMenuOpen((prev) => !prev)}
            className="flex items-center px-1.5 py-0.5 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
            title="More copy and export options"
        >
            <ChevronDown className="w-3 h-3 text-slate-400" />
        </button>

        {/* Flyout export menu */}
        {isExportMenuOpen && ( ... )}
    </div>

    {/* Capsule 2: Execution & Workflow */}
    <div className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#171025]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#40285e]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#40285e]/80 shadow-2xs shrink-0">
        {/* Borderless Confirmation Suffix Selector */}
        <div className="flex items-center px-1.5 py-0.5 rounded-l-full">
            <select
                value={confirmationSuffix}
                onChange={(e) => setConfirmationSuffix(e.target.value)}
                className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[85px] truncate pr-1 py-0"
                title="Confirmation suffix appended on Resend"
            >
                <option value="None (Send as is)" className="bg-white dark:bg-[#171025]">Suffix: None</option>
                <option value="Is it done?" className="bg-white dark:bg-[#171025]">Done?</option>
                <option value="Is it released?" className="bg-white dark:bg-[#171025]">Released?</option>
                <option value="Are you sure about it?" className="bg-white dark:bg-[#171025]">Sure?</option>
                <option value="Double check all edge cases" className="bg-white dark:bg-[#171025]">Edge cases</option>
                <option value="Verify build and tests" className="bg-white dark:bg-[#171025]">Verify tests</option>
            </select>
        </div>

        {/* Borderless Destination Conversation Mode Selector */}
        <div className="flex items-center px-1.5 py-0.5">
            <select
                value={conversationTargetMode}
                onChange={(e) => setConversationTargetMode?.(e.target.value as 'current' | 'new')}
                className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[90px] truncate pr-1 py-0"
                title="Destination conversation mode for Send and Queue"
            >
                <option value="current" className="bg-white dark:bg-[#171025]">Current</option>
                <option value="new" className="bg-white dark:bg-[#171025]">New Conv</option>
            </select>
        </div>

        {/* Focus IDE Action */}
        <button
            type="button"
            onClick={onFocusIde}
            disabled={isFocusing}
            className="flex items-center justify-center px-2 py-0.5 text-sky-700 dark:text-sky-300 hover:bg-sky-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
            title="Focus Antigravity IDE Window or Launch Instance"
        >
            <ExternalLink className={cn('w-3 h-3 text-sky-500', isFocusing && 'animate-spin')} />
        </button>

        {/* Send Action with Hotkey N */}
        <button
            type="button"
            onClick={onResend}
            disabled={isResending}
            className="flex items-center gap-1 px-2 py-0.5 text-emerald-700 dark:text-[#1af18d] hover:bg-emerald-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
            title="Immediately inject prompt to running instance (Hotkey: N)"
        >
            <RotateCw className={cn('w-3 h-3 text-[#1af18d]', isResending && 'animate-spin')} />
            <span>Send</span>
            <kbd className="ml-0.5 px-1 py-0.2 bg-[#1af18d]/20 text-[#1af18d] rounded text-[8.5px] font-mono font-bold">N</kbd>
        </button>

        {/* Enqueue Action */}
        <button
            type="button"
            onClick={onEnqueue}
            disabled={isEnqueueing}
            className="flex items-center justify-center px-2 py-0.5 text-purple-700 dark:text-purple-300 hover:bg-purple-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
            title="Enqueue Prompt into FIFO Scheduler Queue"
        >
            <ListPlus className="w-3.5 h-3.5 text-purple-500" />
        </button>

        {/* Full Inspector Action */}
        <button
            type="button"
            onClick={() => onOpenInspector(selectedConversation)}
            className="flex items-center justify-center px-2 py-0.5 text-blue-700 dark:text-cyan-300 hover:bg-blue-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer rounded-r-full"
            title="Open Full-Screen Conversation Inspector"
        >
            <Maximize2 className="h-3 w-3 text-blue-500 dark:text-cyan-400" />
        </button>
    </div>
</div>
```

---

## 4. Verification & Testing Checklist

- [ ] Open Prompt Tree Modal: verify top shimmer loading bar displays immediately on launch.
- [ ] Verify TreePanel shows multi-item pulsating skeletons during tree queries when cache is cold.
- [ ] Verify zero UI freeze on Windows and Ubuntu (clipboard processes drop stdin immediately; transcript reads are capped at 256KB).
- [ ] Select a conversation and click `Send`: confirm prompt content dispatches to running instance and is copied to clipboard.
- [ ] Select a conversation and click `Enqueue`: confirm prompt is added to FIFO queue in SQLite with status `queued`.
- [ ] Inspect `DetailHeader.tsx`: verify all actions are housed within the 2 segmented dark-glass capsules without loose disconnected buttons.
- [ ] Verify dropdowns inside the capsule use `bg-transparent border-0 focus:ring-0` with zero nested white box defects.
