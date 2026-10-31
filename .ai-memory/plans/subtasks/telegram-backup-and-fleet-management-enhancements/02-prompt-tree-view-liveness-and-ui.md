# Subtask 02: Prompt Tree View Liveness, Expandable Hierarchy & UI Modernization

> **Subtask ID:** `02-prompt-tree-view-liveness-and-ui`  
> **Parent Plan:** `.ai-memory/plans/telegram-backup-and-fleet-management-enhancements.md`  
> **Component Spec:** `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/02-component-spec.md`  
> **Owner:** Worker 02 (Frontend & Rust Subagent)  
> **Status:** READY FOR IMPLEMENTATION  
> **Target Release:** `v4.224.0`  

---

## 1. Overview & Objectives

This subtask implements robust live running prompt detection, responsive sidebar expansion, real-time conversation rename synchronization to `state.vscdb`, terminal tree glyphs, header image filtering, segmented auto-hide capsule styling, and details button health status visualization in the Prompt Tree View.

### Key Objectives:
1. **Default Instance Running Prompt Detection:**
   - Standardize instance specifier normalization across `observe.rs` (`""`, `"default"`, `"__default__"`).
   - Harden canonical directory sweeps in `gemini_dirs.rs` ensuring canonical home, default sandbox, and APPDATA folders are discovered without leaking secondary sandboxes.
   - Enforce strict FIFO `ORDER BY created_at ASC, id ASC` and status validation in `liveness.rs`.
   - Wire IDE heartbeat fallback into `cmd_running_up_next.rs` so live sessions return `now_running` even before database persistence.
2. **Conversation Rename Synchronization:**
   - Implement immutable React state cloning in `usePromptTreeData.ts` to instantly propagate renamed titles to `selectedConversation` and `selectedProject`.
   - Ensure double-clicking in `ConversationNode.tsx` auto-selects the conversation before opening the inspector modal.
   - Extend `sequences.rs` (`update_conversation_title`) to update `state.vscdb` (`ItemTable`) in addition to `conversation_summaries.db` and `repo_prompts.db`.
3. **Expandable Tree View Layout:**
   - Widen sidebar in `TreePanel.tsx` from `w-80 sm:w-96` to responsive `w-[460px] lg:w-[520px]`.
   - Expand project separation with `space-y-3.5` between distinct repositories.
   - Replace legacy arrow glyphs (`↳`) with standard terminal box-drawing glyphs (`├──`, `└──`) in `ProjectNode.tsx` and `ConversationNode.tsx`.
4. **Image Filter Toggle:**
   - Add a pill toggle button (`🖼️ Images` / `Image Filter`) to the segmented header capsule in `ModalHeader.tsx`.
   - Wire filtering logic in `usePromptTreeData.ts` and `TreePanel.tsx` to filter projects and conversations containing image attachments.
5. **Auto-Hide Button Sizing & Segmented Capsule:**
   - Replace the standalone auto-hide button in `TreePanel.tsx` with a unified contiguous segmented pill capsule (`rounded-full`, shared border, subtle divider lines).
   - Style the adjacent scope `<select>` with transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`).
6. **Details Button Relocation & Health Pulse:**
   - Relocate the Details button in `DetailTabs.tsx` featuring the `Activity` icon (from `lucide-react`).
   - Add an animated `#1af18d` pulsing green health dot when `selectedConversation?.is_running` is active.
7. **Markdown Background Consistency:**
   - Confirm and enforce `dark:bg-[#19191c]` across `PromptInstructionView.tsx`, `InspectorModal.tsx`, and `ResultsView.tsx`.

---

## 2. Owned Files

| File Path | Subsystem | Responsibility |
| :--- | :--- | :--- |
| `src-tauri/src/modules/instance/observe.rs` | Backend / Instance | Specifier normalization (`default`, `__default__`, `""`) |
| `src-tauri/src/modules/repo_db/gemini_dirs.rs` | Backend / Repo DB | Default directory matching and secondary sandbox isolation |
| `src-tauri/src/modules/repo_db/liveness.rs` | Backend / Repo DB | FIFO query ordering and active status validation |
| `src-tauri/src/commands/cmd_running_up_next.rs` | Backend / IPC | IDE heartbeat fallback for live prompt detection |
| `src-tauri/src/modules/repo_db/sequences.rs` | Backend / Repo DB | `state.vscdb` workspace storage title synchronization |
| `src/components/instances/PromptTreeView/usePromptTreeData.ts` | Frontend / Data Hook | Immutable state cloning, SWR cache update, image filtering |
| `src/components/instances/PromptTreeView/ConversationNode.tsx` | Frontend / UI | Auto-selection on double-click, terminal glyphs (`├──`, `└──`) |
| `src/components/instances/PromptTreeView/ProjectNode.tsx` | Frontend / UI | Terminal tree glyphs (`├──`, `└──`) for subagent tasks |
| `src/components/instances/PromptTreeView/TreePanel.tsx` | Frontend / UI | Responsive sidebar width (`w-[460px] lg:w-[520px]`), `space-y-3.5`, segmented capsule |
| `src/components/instances/PromptTreeView/ModalHeader.tsx` | Frontend / UI | Header pill toggle button for image filtering |
| `src/components/instances/PromptTreeView/DetailTabs.tsx` | Frontend / UI | Relocated Details button, `Activity` icon, `#1af18d` health pulse |
| `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Frontend / UI | `dark:bg-[#19191c]` markdown background |
| `src/components/instances/PromptTreeView/InspectorModal.tsx` | Frontend / UI | `dark:bg-[#19191c]` markdown background |
| `src/components/instances/PromptTreeView/ResultsView.tsx` | Frontend / UI | `dark:bg-[#19191c]` markdown background |

---

## 3. Detailed Task Breakdown

### Phase 1: Backend Liveness & Rename Synchronization (Rust)
- [ ] **Step 1.1: Specifier Normalization in `observe.rs`**
  - Implement unified normalization for instance queries in `resolve_instance_id`.
  - Guarantee that `""`, `"default"`, and `"__default__"` resolve deterministically to the default instance configuration.
- [ ] **Step 1.2: Default Instance Directory Matching in `gemini_dirs.rs`**
  - Verify that `gemini_dirs_for_instance("default")` includes canonical host home, default sandbox, and APPDATA without duplicate entries.
  - Ensure `is_secondary_sandbox_path` cleanly excludes non-default instances.
- [ ] **Step 1.3: FIFO Liveness Validation in `liveness.rs`**
  - Enforce `ORDER BY created_at ASC, id ASC` across active prompt queries.
  - Cross-check `CascadeRunStatusType` and heartbeat freshness before declaring a session idle.
- [ ] **Step 1.4: IDE Heartbeat Fallback in `cmd_running_up_next.rs`**
  - Add fallback ladder to `query_now_running`: if SQLite yields `None`, inspect `inspect_prompt_goal_status(&target, &workspaces)`.
  - When an active worker process is alive with a fresh heartbeat, synthesize a `PromptQueueRow` with `status: "running"`.
- [ ] **Step 1.5: `state.vscdb` Workspace Storage Synchronization in `sequences.rs`**
  - Implement `sync_title_to_state_vscdb(&data_dir, conversation_id, new_title)`.
  - Update `update_conversation_title` to trigger `sync_title_to_state_vscdb` alongside SQLite updates.

### Phase 2: Frontend Data Hook & Selection Synchronization (React/TS)
- [ ] **Step 2.1: Immutable State Cloning in `usePromptTreeData.ts`**
  - Update `renameConversationOptimistic` to immutably map tree projects and conversation nodes.
  - Provide callback dispatch to update parent `selectedConversation` state without requiring full reload.
- [ ] **Step 2.2: Image Filter Logic in `usePromptTreeData.ts`**
  - Add `isImageFilterActive` state flag.
  - Prune projects and conversations that lack image attachments when the filter is active.
- [ ] **Step 2.3: Double-Click Auto-Selection in `ConversationNode.tsx`**
  - In `onDoubleClick`, invoke `onSelect({ ...conv, selected_turn_number: defaultTurnNum }, project)` before calling `onOpenInspector`.

### Phase 3: Frontend Layout Expansion & UI Modernization (React/TS)
- [ ] **Step 3.1: Responsive Sidebar Width & Project Spacing in `TreePanel.tsx`**
  - Update panel root container width to `w-[460px] lg:w-[520px]`.
  - Change project container list spacing to `space-y-3.5`.
- [ ] **Step 3.2: Terminal Tree Branch Glyphs in `ProjectNode.tsx` & `ConversationNode.tsx`**
  - Replace `↳` with monospace `├──` for intermediate child turns/subagents and `└──` for the terminating child.
- [ ] **Step 3.3: Header Pill Toggle for Image Filter in `ModalHeader.tsx`**
  - Add the `🖼️ Images` pill toggle button inside the header capsule with active sky styling.
- [ ] **Step 3.4: Auto-Hide Button Segmented Capsule in `TreePanel.tsx`**
  - Wrap Auto-Hide toggle and Scope `<select>` in a single contiguous segmented pill capsule (`rounded-full`, shared border, subtle divider lines).
  - Apply borderless transparent styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to the `<select>`.
- [ ] **Step 3.5: Details Button Relocation & Health Pulse in `DetailTabs.tsx`**
  - Relocate the Details button with `Activity` icon (from `lucide-react`).
  - Add animated glowing `#1af18d` pulsing health indicator dot when `selectedConversation?.is_running` is true.
- [ ] **Step 3.6: Markdown Background Uniformity**
  - Verify and enforce `dark:bg-[#19191c]` across `PromptInstructionView.tsx`, `InspectorModal.tsx`, and `ResultsView.tsx`.

---

## 4. Verification Steps

1. **Rust Pre-flight Checks:**
   ```bash
   cd src-tauri && cargo fmt -- --check
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
2. **Backend Unit Testing:**
   ```bash
   cd src-tauri && cargo test --bin antigravity-manager modules::repo_db::liveness
   cd src-tauri && cargo test --bin antigravity-manager modules::repo_db::sequences
   ```
3. **Frontend Build Verification:**
   ```bash
   npm run build
   ```
4. **Behavioral Acceptance Criteria:**
   - [ ] Default instance running prompt detected and rendered with `RUNNING` status in Prompt Tree View.
   - [ ] Conversation renaming immediately updates sidebar node, header title, and persists into `state.vscdb`.
   - [ ] Double-clicking any conversation node auto-selects it and opens Inspector with matching data.
   - [ ] Sidebar renders with `w-[460px] lg:w-[520px]` width and `space-y-3.5` project spacing.
   - [ ] Terminal branch glyphs `├──` and `└──` rendered in monospace font for subagents and prompt turns.
   - [ ] Image filter header button toggles visibility of image-bearing conversations.
   - [ ] Auto-hide button and scope dropdown styled as a contiguous segmented pill capsule with borderless `<select>`.
   - [ ] Details button displays `Activity` icon and `#1af18d` pulsing green health dot when active.
   - [ ] Markdown containers in `PromptInstructionView.tsx`, `InspectorModal.tsx`, and `ResultsView.tsx` display `dark:bg-[#19191c]`.
