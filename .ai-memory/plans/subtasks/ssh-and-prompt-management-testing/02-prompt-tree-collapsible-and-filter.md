---
plan: ssh-and-prompt-management-testing
subtask: "02"
title: Collapsible Prompt Tree & Default-Hidden Automated Prompts Filter
domain: frontend-ui-cli
depends_on: []
target_files:
  - src/components/instances/PromptTreeViewModal.tsx
  - src/components/instances/PromptTreeView/usePromptTreeState.ts
  - src/components/instances/PromptTreeView/TreePanel.tsx
  - src/components/instances/PromptTreeView/ModalHeader.tsx
  - src/components/instances/PromptTreeView/tierClassification.ts
status: pending
---

# Subtask 02: Collapsible Prompt Tree & Default-Hidden Automated Prompts Filter

## 1. Context & Motivation
In `PromptTreeViewModal`, modal real estate is essential for reading detailed prompt instructions, reviewing multimodal attached images, and editing prompts prior to re-injection. When the multi-project tree sidebar is always expanded, it occupies up to 384px (`w-96`) of screen width, forcing prompt content into a cramped view.

Additionally:
1. **Automated Noise Pollution**: Automated prompt goal heartbeats (every 5 seconds), system preambles (`<SYSTEM_MESSAGE>`), token usage telemetry, and tool outputs flood conversation lists. This makes finding genuine human prompts and autonomous subagent directives difficult.
2. **Default-Hidden Collapsible Tree**: Users require the prompt tree view to be collapsible and initialized in the collapsed/hidden state by default (`isTreeCollapsed = true`), allowing full-width prompt inspection until the user explicitly expands the tree.
3. **Default-Hidden Automated Prompts**: Automated system and heartbeat turns must be filtered out by default (`isAutomatedHidden = true`), with a quick-toggle pill button (`Hide Auto` / `Show Auto`) to reveal them on demand for diagnostic inspection.
4. **CLI Parity**: Terminal users must have equivalent capabilities via `agm observe` and `agm tree` to inspect live instances and filter automated turns.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/instances/PromptTreeView/usePromptTreeState.ts` | `usePromptTreeState`, `PromptTreeState` | 1. Initialize `isTreeCollapsed` to `true` by default.<br/>2. Initialize `isAutomatedHidden` to `true` by default.<br/>3. Export getters and setters in `PromptTreeState` interface. |
| `src/components/instances/PromptTreeView/tierClassification.ts` | `isAutomatedConversation`, `classifyPromptTier` | 1. Identify automated turns based on `PromptOriginTier.SystemMessage`, `PromptOriginTier.ToolOutput`, heartbeat tags, and token usage markers.<br/>2. Return boolean classification for conversation filtering. |
| `src/components/instances/PromptTreeView/TreePanel.tsx` | `TreePanel`, `TreePanelProps` | 1. Render `Hide Auto` / `Show Auto` segmented pill button with `Eye` / `EyeOff` icons.<br/>2. Provide tree collapse button (`PanelLeftClose`) in project scope header.<br/>3. Forward `isAutomatedHidden` to child `ProjectNode` components. |
| `src/components/instances/PromptTreeView/ModalHeader.tsx` | `ModalHeader`, `ModalHeaderProps` | 1. Render `Show Tree` / `Hide Tree` toggle button inside the Window Controls segmented dark-glass capsule.<br/>2. Use `PanelLeftOpen` / `PanelLeftClose` icons with tooltip hints. |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | `PromptTreeViewModal` | 1. Conditionally render the slim 40px icon rail (`w-10`) when `isTreeCollapsed === true`.<br/>2. Render `<TreePanel />` when `isTreeCollapsed === false`.<br/>3. Wire state and toggle handlers between header, panel, and detail views. |
| `src/components/instances/PromptTreeViewModal.tsx` | Backward-compatibility re-exports | Verify re-exports preserve public API for external consumers and test suites. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Default State Initialization in `usePromptTreeState.ts`

1. In `usePromptTreeState.ts`, verify that `isTreeCollapsed` and `isAutomatedHidden` are initialized to `true`:
   ```typescript
   // Default to collapsed tree panel and hidden automated turns
   const [isTreeCollapsed, setIsTreeCollapsed] = useState<boolean>(true);
   const [isAutomatedHidden, setIsAutomatedHidden] = useState<boolean>(true);
   ```

2. Export both state variables and setters in the returned `PromptTreeState` object and interface:
   ```typescript
   export interface PromptTreeState {
       ...
       isTreeCollapsed: boolean;
       setIsTreeCollapsed: React.Dispatch<React.SetStateAction<boolean>>;
       isAutomatedHidden: boolean;
       setIsAutomatedHidden: React.Dispatch<React.SetStateAction<boolean>>;
   }
   ```

---

### 3.2 Step 2: Automated Conversation Classification in `tierClassification.ts`

1. Ensure `isAutomatedConversation` detects all system, tool, heartbeat, and telemetry turns:
   ```typescript
   export function isAutomatedConversation(conv: AgmConversationNode): boolean {
       const preview = (conv.prompt_preview_200w || '').toLowerCase();
       const title = (conv.title || '').toLowerCase();
       const cat = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);

       // 1. Tier classification matches System Message or Tool Output
       if (cat.tier === PromptOriginTier.SystemMessage || cat.tier === PromptOriginTier.ToolOutput) {
           return true;
       }

       // 2. System message tags
       if (preview.includes('<system_message>') || preview.includes('system_message')) {
           return true;
       }

       // 3. Heartbeat goal tracking
       if (title.includes('heartbeat') || preview.includes('heartbeat') || title.startsWith('system:')) {
           return true;
       }

       // 4. Token usage and telemetry records
       if (preview.includes('token_usage') || preview.includes('telemetry') || title.includes('telemetry')) {
           return true;
       }

       return false;
   }
   ```

---

### 3.3 Step 3: Quick-Toggle Pill & Collapse Controls in `TreePanel.tsx`

1. In `TreePanel.tsx`, ensure the quick-toggle pill button is rendered in the filter toolbar:
   ```tsx
   {setIsAutomatedHidden && (
       <button
           type="button"
           onClick={() => setIsAutomatedHidden(!isAutomatedHidden)}
           className={cn(
               "px-1.5 py-0.5 text-[10px] font-medium rounded-md transition-colors cursor-pointer flex items-center gap-1 border",
               isAutomatedHidden
                   ? "bg-slate-100 dark:bg-[#201538] text-slate-600 dark:text-slate-400 border-slate-200 dark:border-[#40285e] hover:text-slate-900 dark:hover:text-slate-200"
                   : "bg-purple-500/10 text-purple-600 dark:text-purple-300 border-purple-500/30 font-semibold"
           )}
           title={isAutomatedHidden ? "Show automated system & heartbeat turns" : "Hide automated system & heartbeat turns"}
       >
           {isAutomatedHidden ? <EyeOff className="w-2.5 h-2.5 shrink-0" /> : <Eye className="w-2.5 h-2.5 shrink-0" />}
           <span>{isAutomatedHidden ? 'Hide Auto' : 'Show Auto'}</span>
       </button>
   )}
   ```

2. Forward `isAutomatedHidden` to `<ProjectNode />`:
   ```tsx
   <ProjectNode
       ...
       isAutomatedHidden={isAutomatedHidden}
   />
   ```

3. Provide a panel collapse button (`PanelLeftClose`) in the top project scope bar for easy collapse without scrolling to the header.

---

### 3.4 Step 4: Window Controls Capsule Toggle in `ModalHeader.tsx`

1. In `ModalHeader.tsx`, integrate the collapse/expand toggle into the contiguous segmented dark-glass controls capsule:
   ```tsx
   {/* Tree View Collapse / Expand Toggle */}
   {onToggleTree && (
       <button
           type="button"
           onClick={onToggleTree}
           className="flex items-center gap-1.5 px-2.5 py-1 rounded-l-md text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-colors cursor-pointer"
           title={isTreeCollapsed ? "Show Prompt Tree Panel" : "Hide Prompt Tree Panel"}
       >
           {isTreeCollapsed ? (
               <PanelLeftOpen className="w-3.5 h-3.5 text-blue-500" />
           ) : (
               <PanelLeftClose className="w-3.5 h-3.5 text-slate-400" />
           )}
           <span className="hidden sm:inline">{isTreeCollapsed ? 'Show Tree' : 'Hide Tree'}</span>
       </button>
   )}
   ```

---

### 3.5 Step 5: Collapsed Rail & Split View in `PromptTreeViewModal.tsx`

1. In `PromptTreeViewModal.tsx`, handle the collapsed state in the main split view layout:
   ```tsx
   {/* Main Split View */}
   <div className="flex flex-1 overflow-hidden">
       {!isTreeCollapsed ? (
           <TreePanel
               ...
               isAutomatedHidden={isAutomatedHidden}
               setIsAutomatedHidden={setIsAutomatedHidden}
               isCollapsed={isTreeCollapsed}
               onToggleCollapse={() => setIsTreeCollapsed(true)}
           />
       ) : (
           <div className="w-10 shrink-0 border-r border-slate-200 dark:border-[#40285e] bg-slate-50/70 dark:bg-[#090611]/70 flex flex-col items-center py-3 gap-3">
               <button
                   type="button"
                   onClick={() => setIsTreeCollapsed(false)}
                   className="p-1.5 rounded-lg text-slate-500 hover:text-blue-500 dark:text-slate-400 dark:hover:text-blue-400 hover:bg-slate-200/60 dark:hover:bg-[#201538] transition-colors cursor-pointer"
                   title="Expand Tree Panel"
               >
                   <PanelLeftOpen className="w-4 h-4" />
               </button>
               <span
                   className="text-[10px] font-semibold text-slate-400 dark:text-slate-500 tracking-wider uppercase cursor-pointer select-none [writing-mode:vertical-rl] rotate-180 hover:text-slate-700 dark:hover:text-slate-300 transition-colors"
                   onClick={() => setIsTreeCollapsed(false)}
               >
                   Show Tree
               </span>
           </div>
       )}

       {/* Right: Conversation Detail View */}
       ...
   </div>
   ```

---

### 3.6 Step 6: CLI Parity via `agm observe` and `agm tree`

1. `agm observe [instance_id]`:
   - Inspects running status, conscious PIDs, bound account email, and active 5-second prompt goal heartbeats.
   - Outputs clean summary without flooding the terminal with raw automated logs.
2. `agm tree`:
   - Renders the hierarchical tree of Projects (`[P001]`), Conversations (`[C001]`), and 200-word prompt previews.
   - Use `agm tree` (defaults to active/running) or `agm tree all` (all workspaces).
   - Use `agm observe` for live heartbeat verification and prompt queue tracking.

---

## 4. Acceptance Criteria & Edge Cases

- [ ] **AC-1 (Default Collapsed Tree)**: When opening `PromptTreeViewModal`, `isTreeCollapsed` is `true`. The modal opens with the slim 40px icon rail and full-width prompt preview/instruction area.
- [ ] **AC-2 (Expand/Collapse via Rail & Header)**: Clicking the rail or clicking `Show Tree` in `ModalHeader` expands `<TreePanel />`. Clicking `Hide Tree` or the collapse icon on `<TreePanel />` returns to the collapsed rail.
- [ ] **AC-3 (Default-Hidden Automated Prompts)**: `isAutomatedHidden` is `true` by default. Automated turns (system preambles, heartbeat prompts, tool outputs) are filtered out from the conversation lists.
- [ ] **AC-4 (Quick-Toggle Pill Button)**: `<TreePanel />` renders a pill toggle button displaying `Hide Auto` / `Show Auto`. Clicking it immediately toggles visibility of automated turns without reloading the tree.
- [ ] **AC-5 (Accurate Classification)**: `isAutomatedConversation` accurately identifies system messages, tool results, and prompt goal heartbeats while preserving genuine user prompts and subagent tasks.
- [ ] **AC-6 (CLI Parity)**: Running `agm observe` displays instance liveness, conscious PIDs, and heartbeat status from the terminal. Running `agm tree` renders the project-conversation hierarchy.
- [ ] **Edge Case (All-Automated Project)**: If all conversations in a project are automated and hidden, the project displays `(0 convs)` or filters cleanly without throwing an exception or rendering broken branch glyphs.
- [ ] **Edge Case (Deep-Linked Selection)**: When deep-linking to an initial project/conversation, the selected conversation is loaded into the detail view even if the tree panel is collapsed.

---

## 5. Verification Commands

Run the following commands to verify that code changes adhere to project standards:

```bash
# 1. Frontend build verification
npm run build

# 2. Rust formatting gate
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# 3. Rust compiler and clippy gate
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features

# 4. CLI live observation verification
agm observe active
agm tree
```
