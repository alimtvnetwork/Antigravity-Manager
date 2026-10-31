# Master Execution Plan: UI and Functionality Enhancement Instructions

**Slug:** `ui-and-functionality-enhancement-instructions`  
**Status:** `IN_PROGRESS`  
**Author:** `@aukgit`

---

## 1. Executive Problem Summary & Core Remediation Tracks

1. **Track 1: Instance Prompt Isolation (`repo_db/liveness.rs`, `gemini_dirs.rs`, `tree_node_conv.rs`)**:
   - Prevent `antigravity-cli` directories from polluting default GUI instance scope.
   - Populate `instance_seq_num` across conversation nodes so sequence numbers match frontend expectations.
   - Restrict empty instance IDs in `observe.rs` from defaulting to `default`.
2. **Track 2: Prompt Tree View Ergonomics & Reduced Button Rounding (`PromptTreeView`)**:
   - Refactor excessive `rounded-full` pill buttons to crisp `rounded-md` / `rounded-[5px]`.
   - Compact `[Input | Output | Expand Full]` into right-hand header alongside word/byte counts.
   - Implement right-click context menu on tree sidebar items.
   - Display enqueued prompts in dedicated permanent FIFO queue section.
   - Enhance active conversation styling for high visual contrast.
3. **Track 3: Rich Multimodal Copy & Image Previews**:
   - Split copy into "Copy with Images" (with visible badge) vs "Copy Text Only".
   - Restore interactive Detail button showing Instance ID, Name, Seq #, PID, Path, and Liveness.
   - Enable image preview and native PNG clipboard copying.
4. **Track 4: Standalone Portable JSON & Overlay Fixes**:
   - Add Standalone Export mode with base64 embedded images and `${workDir}` path normalization.
   - Deconflict modal backdrop: remove `backdrop-blur-md` on child modals, bump z-index, fix outer scrollbar click dismissal.
5. **Track 5: SSH Cluster & Node W2 Parity**:
   - Update `gitmap-ssh-nodes.json` node `w2` to Windows.
   - Render offline nodes in `FleetMachinesTable.tsx` rather than collapsing into disabled banner.
   - Allow AGM to fall back to GitMap SSH execution.
6. **Track 6: App Updater Pinned-Version & v4.190 Trap Remediation**:
   - Update `install.ps1` to extract and shortcut to `agm-ui.exe` while maintaining `agm-alim.exe` compatibility.
   - Parse SemVer from GitHub release URLs in `args.rs` and `install.ps1`.
7. **Track 7: Coding Guidelines (CG) Refactoring**:
   - Guard clause inversion, positive boolean prefixes (`is`/`has`), constants/enums normalization, and sequence-14 function return signatures.

---

## 2. Subtask Breakdown & File Boundaries

| Subtask ID | Title | Assigned Agent | Exclusive Owned Files |
|---|---|---|---|
| `Subtask-01` | CG Refactoring & Guard Clause Simplification | Worker 01 | Touched modules in `src/` and `src-tauri/` |
| `Subtask-02` | Instance Running Prompt Isolation | Worker 02 | `src-tauri/src/modules/repo_db/gemini_dirs.rs`, `liveness.rs`, `tree_node_conv.rs`, `tree_collect.rs` |
| `Subtask-03` | Prompt Tree View UI/UX & Button Rounding | Worker 01 | `src/components/instances/PromptTreeView/DetailHeader.tsx`, `DetailTabs.tsx`, `TreePanel.tsx`, `PromptInstructionView.tsx` |
| `Subtask-04` | Image Clipboard Copy & Markdown Preview | Worker 01 | `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`, `clipboard.ts`, `PromptMarkdownView.tsx` |
| `Subtask-05` | Standalone Portable JSON & Overlay Deconfliction | Worker 03 | `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`, `PromptTreeImportModal.tsx`, `src-tauri/src/modules/repo_db/export.rs` |
| `Subtask-06` | SSH Cluster & Node W2 Parity | Worker 03 | `gitmap-ssh-nodes.json`, `src/pages/Instances/components/FleetMachinesTable.tsx`, `src-tauri/src/modules/ssh_manager/` |
| `Subtask-07` | App Updater Pinned-Version & v4.190 Trap Fix | Worker 02 | `src-tauri/src/modules/delegate_updater/args.rs`, `install.ps1` |
| `Subtask-08` | Verification & Release Ceremony | Lead | Quality gates, bump, changelog, atomic commit |
