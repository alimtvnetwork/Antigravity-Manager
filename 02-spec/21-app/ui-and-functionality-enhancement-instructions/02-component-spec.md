# Component Specification: UI and Functionality Enhancement Instructions

**Slug:** `ui-and-functionality-enhancement-instructions`  
**Status:** `ACTIVE`  
**Version:** `1.0.0`  
**Author:** `@aukgit`

---

## 1. Frontend Component Contracts

### 1.1 `DetailHeader.tsx` & `DetailTabs.tsx`
- **Button Rounding Invariant:**
  - Standardize all pill buttons to `rounded-md` with `border border-slate-200 dark:border-[#40285e]`.
  - Child start button: `rounded-l-md`.
  - Child end button: `rounded-r-md`.
  - Child divider: `w-px h-3 bg-slate-200 dark:bg-[#40285e]`.
- **Right-Aligned Compact Controls:**
  - Group `[Input | Output | Expand Full]` into a contiguous segmented capsule positioned at `ml-auto`.
  - Next to capsule, render compact word and byte stats: `{activeWordCount} words · {formatByteSize(activeByteCount)}`.
- **Instance Telemetry Chip:**
  - Render instance badge as an interactive click target:
    ```tsx
    <button
        type="button"
        onClick={() => setIsDetailsModalOpen(true)}
        className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md border border-slate-200 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#120a1f] text-[10px] font-mono hover:border-blue-500 transition-colors cursor-pointer"
        title="View instance execution telemetry"
    >
        <span>Instance #{instanceSeqNum} · {instanceNameDisplay}</span>
        <FileText className="w-3 h-3 text-blue-400" />
    </button>
    ```

### 1.2 `DetailsModal.tsx`
- **Telemetry Grid Enhancement:**
  - Display explicit `Instance ID` (`selectedConversation?.instance_id || instanceId`).
  - Provide 1-click copy action with visual confirmation tooltip.
  - Display `PID`, `Executable Path`, `Data Directory Path`, `Sequence Number`, and `Liveness State`.

### 1.3 `TreePanel.tsx` & Tree Item Context Menu
- **Context Menu Interaction:**
  - Lift context menu state to `TreePanel.tsx`:
    ```typescript
    interface TreeContextMenuState {
        type: 'project' | 'conversation' | 'turn';
        target: any;
        x: number;
        y: number;
    }
    ```
  - Support right-click actions on conversations:
    - Copy Prompt (with Images)
    - Copy Text Only (without Images)
    - View in Raw Mode
    - Inspect Instance Telemetry
    - Rerun Prompt
    - Export Conversation as JSON
- **High-Contrast Active Conversation Styling:**
  - Active conversation styling: `bg-blue-600/25 dark:bg-purple-600/30 text-blue-900 dark:text-cyan-200 border-l-4 border-blue-500 dark:border-cyan-400 font-semibold shadow-xs`.
  - Green active dot on running conversations; amber dot on enqueued conversations.

### 1.4 `PromptInstructionView.tsx` (FIFO Queue Panel)
- **Permanent Enqueued Prompts Section:**
  - Whenever `enqueuedTurns.length > 0`, mount a non-collapsible FIFO Queue box directly below the active prompt card.
  - Each item displays: `QUEUE #{index + 1}`, prompt preview snippet, timestamp, and quick-preview button.

### 1.5 `PromptTreeExportModal.tsx` & `PromptTreeImportModal.tsx`
- **Modal Overlay Deconfliction:**
  - Remove `backdrop-blur-md` on child modals; keep `bg-black/50`.
  - Set z-index to `z-[100100]`.
  - Remove `overflow-y-auto` from outer backdrop to prevent clicks on scrollbar from closing modal.
  - Export Mode Toggle: Standalone Portable JSON (embeds images in base64, replaces absolute paths with `${workDir}`) vs Lean JSON (compact relative paths).

---

## 2. Backend Rust Subsystems

### 2.1 `src-tauri/src/modules/repo_db/gemini_dirs.rs`
- Isolate `antigravity-cli`:
  - Tag `antigravity-cli` directories with `"__cli__"` rather than `"default"`.
  - Exclude `antigravity-cli` from `gemini_dirs_for_instance("default")`.

### 2.2 `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- Populate `instance_seq_num`:
  - Resolve sequence number using `registry.instances.iter().find(|i| i.id == proj.instance_id).and_then(|i| i.seq_num)`.
  - Default instance emits `instance_seq_num: Some(1)`.

### 2.3 `src-tauri/src/modules/delegate_updater/args.rs` & `install.ps1`
- Robust Version Parsing:
  - Extract SemVer from GitHub release URLs via regex:
    `(?:\/tag\/|v|^v)?(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?)$`.
- Binary Name Parity in `install.ps1`:
  - Extract `agm-ui.exe` as primary GUI binary.
  - Overwrite or copy `agm-ui.exe` to `agm-alim.exe` for legacy compatibility.
  - Point Desktop and Start Menu shortcuts to `agm-ui.exe`.
  - Read installed version from `agm-ui.exe` first.

### 2.4 `src/pages/Instances/components/FleetMachinesTable.tsx`
- Show Offline Nodes:
  - Render unreachable/offline nodes with an "Offline" badge instead of collapsing into `<FleetDisabledState />`.
  - Update `gitmap-ssh-nodes.json` node `w2` to `"os": "windows"`.
