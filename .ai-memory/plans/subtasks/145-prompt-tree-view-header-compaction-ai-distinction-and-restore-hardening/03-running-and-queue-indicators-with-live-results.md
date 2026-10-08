# Subtask 03 Spec: Running & Queue Indicators with Live Results & Window Focus Hardening

**Document ID:** `.ai-memory/plans/subtasks/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/03-running-and-queue-indicators-with-live-results.md`  
**Parent Task:** `145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening`  
**Target Codebase:**  
- `src-tauri/src/modules/repo_db.rs`  
- `src-tauri/src/modules/instance.rs`  
- `src-tauri/src/modules/process.rs`  
- `src/components/instances/PromptTreeViewModal.tsx`  
- `src/pages/Instances.tsx`  
- `src/services/instanceService.ts`  
**Lead Author:** Author 02 (Backend & Liveness Engine Specialist)  
**Status:** `COMPLETED`  

---

## 1. Scope & Objective

The objective of Subtask 03 is to solve defects **D3 and D4** by delivering comprehensive in-flight and queued task visibility across the AGM GUI, live results streaming into the prompt preview pane, and rock-solid cross-platform IDE window focusing:

1. **Queued & Running Telemetry Propagation:** Add `queued_count: usize` and `running_count: usize` to `AgmProjectTreeNode` so project cards prominently display `{runningCount} RUNNING` in glowing emerald (`#1af18d`) alongside `{queuedCount} QUEUED` in amber (`#f59e0b`).
2. **Repeat Group Queue State Preservation:** Ensure `group_identical_conversation_runs` inherits queued states (`is_queued`) onto parent cluster nodes when any underlying sub-run is queued.
3. **Unified GUI Pulse & Ping Styling:** Harmonize running pulse indicators across `Instances.tsx` and `PromptTreeViewModal.tsx` into contiguous segmented pill capsules using standard `#1af18d` emerald glow.
4. **Live Execution Step Streaming in Preview Pane:** Display the latest execution step (`latest_step_summary`), active tool actions, and thought snippets in the prompt preview pane when an active conversation is selected.
5. **Hardened Workspace Focus & Launch:** Overhaul `focusOrLaunchInstance` in `instance.rs` and `process.rs` to enumerate windows and match specific workspace directory titles (User32 / AppleScript / wmctrl) before falling back to launching the target executable.

---

## 2. Architecture & Data Flow

```
┌────────────────────────────────────────────────────────────────────────┐
│                      AgmProjectTreeNode (Backend)                      │
│   - running_count: usize                                               │
│   - queued_count:  usize                                               │
│   - conversations: [ AgmConversationNode (is_running, is_queued) ]     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                         Tauri IPC / REST API
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                      PromptTreeViewModal & Instances                   │
│                                                                        │
│  [ P001 | GM:#1 ]  Antigravity-Manager                                 │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  (●) 2 RUNNING  │  (⏳) 1 QUEUED   (Contiguous Segmented Capsule) │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  Live Preview Pane:                                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ ⚡ Live Step: Step 14 · Tool: run_command (running)              │  │
│  │ 💭 Thinking: "Analyzing directory structure for prompt tree..."  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Click "Focus IDE Window"
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│              Cross-Platform Window Activation (process.rs)             │
│   - Windows: EnumWindows + GetWindowText + SetForegroundWindow         │
│   - macOS:   AppleScript AXRaise + open -a                             │
│   - Linux:   wmctrl -a <repo_name> + xdotool                           │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Concrete Implementation Details

### 3.1 Backend Data Model & Aggregation (`repo_db.rs`)

#### 1. Expand `AgmProjectTreeNode` Struct
In `src-tauri/src/modules/repo_db.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmProjectTreeNode {
    pub seq_id: i64,
    pub seq_code: String,
    pub gitmap_seq_code: String,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub instance_id: String,
    pub instance_seq_num: Option<u32>,
    pub instance_name: String,
    #[serde(default)]
    pub instance_exe_name: String,
    pub bound_email: Option<String>,
    pub is_running: bool,
    #[serde(default)]
    pub running_count: usize,
    #[serde(default)]
    pub queued_count: usize,
    pub conversations: Vec<AgmConversationNode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_badge: Option<String>,
}
```

#### 2. Propagate Counts in `get_project_conversation_tree`
In `src-tauri/src/modules/repo_db.rs`:

```rust
let running_count = grouped_convs.iter().filter(|c| c.is_running).count();
let queued_count = grouped_convs.iter().filter(|c| c.is_queued).count();

tree_nodes.push(AgmProjectTreeNode {
    seq_id: p_seq,
    seq_code: format!("P{:03}", p_seq),
    gitmap_seq_code: format!("GM:#{}", p_seq),
    project_id: proj.id.clone(),
    repo_name: proj.repo_name.clone(),
    repo_path: proj.repo_path.clone(),
    instance_id: proj.instance_id.clone(),
    instance_seq_num,
    instance_name,
    instance_exe_name,
    bound_email,
    is_running: proj_is_running,
    running_count,
    queued_count,
    conversations: grouped_convs,
    byte_size: Some(proj_byte_size),
    repeat_count: None,
    repeat_badge: None,
});
```

#### 3. Propagate Queued State in `group_identical_conversation_runs`
In `src-tauri/src/modules/repo_db.rs`:

```rust
if matches.len() > 1 {
    let count = matches.len();
    let mut parent = matches[0].clone();
    let any_running = matches.iter().any(|m| m.is_running);
    let any_queued = matches.iter().any(|m| m.is_queued);
    
    if any_running {
        parent.is_running = true;
        parent.status = "RUNNING".to_string();
    } else if any_queued {
        parent.is_queued = true;
        parent.status = "QUEUED".to_string();
    }
    
    parent.byte_size = Some(parent.prompt_preview_200w.len());
    parent.repeat_count = Some(count);
    parent.repeat_badge = Some(format!("x{}", count));
    parent.sub_runs = matches;
    grouped.push(parent);
}
```

---

### 3.2 Frontend Indicator Modernization (`PromptTreeViewModal.tsx`)

#### 1. TypeScript Interface Update
In `src/components/instances/PromptTreeViewModal.tsx`:

```typescript
export interface AgmProjectTreeNode {
    seq_id: number;
    seq_code: string;
    gitmap_seq_code: string;
    project_id: string;
    repo_name: string;
    repo_path: string;
    instance_id: string;
    instance_seq_num?: number;
    instance_name: string;
    instance_exe_name?: string;
    bound_email?: string;
    is_running: boolean;
    running_count: number;
    queued_count: number;
    conversations: AgmConversationNode[];
    byte_size?: number;
    repeat_count?: number;
    repeat_badge?: string;
}

export interface AgmConversationNode {
    seq_id: number;
    seq_code: string;
    gitmap_seq_code: string;
    conversation_id: string;
    short_id: string;
    title: string;
    status: string;
    is_running: boolean;
    is_queued?: boolean;
    step_count: number;
    instance_id: string;
    prompt_preview_200w: string;
    prompt_word_count: number;
    last_modified: string;
    prompt_category?: 'USER_PROMPT' | 'SUBAGENT_INSTRUCTION' | 'SYSTEM_STEP';
    latest_step_summary?: string;
    sub_runs?: AgmConversationNode[];
}
```

#### 2. Project Card Contiguous Segmented Pill Badge
Replace lines 2177-2186 in `PromptTreeViewModal.tsx` with unified contiguous indicators:

```tsx
<div className="flex items-center gap-1 shrink-0">
    {(project.running_count > 0 || project.queued_count > 0) && (
        <div className="inline-flex items-center rounded-full border border-slate-700/50 bg-slate-900/80 p-0.5 shadow-sm">
            {project.running_count > 0 && (
                <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400">
                    <span className="relative flex h-2 w-2">
                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#1af18d] opacity-75"></span>
                        <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)]"></span>
                    </span>
                    <span className="text-[9px] font-bold font-mono text-[#1af18d]">
                        {project.running_count} RUNNING
                    </span>
                </div>
            )}
            {project.running_count > 0 && project.queued_count > 0 && (
                <span className="w-px h-3 bg-slate-700/60 mx-0.5" />
            )}
            {project.queued_count > 0 && (
                <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-400">
                    <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
                    <span className="text-[9px] font-bold font-mono text-amber-400">
                        {project.queued_count} QUEUED
                    </span>
                </div>
            )}
        </div>
    )}
    
    {/* Refresh and Pin buttons wrapped in compact capsule */}
    <div className="inline-flex items-center rounded-full border border-slate-700/40 bg-slate-900/60 p-0.5">
        <button
            type="button"
            onClick={(e) => handleRefreshSingleProject(project.project_id, e)}
            disabled={refreshingProjectId === project.project_id}
            className="p-1 rounded-full text-slate-400 hover:text-cyan-400 hover:bg-slate-800 transition-colors cursor-pointer"
            title="Refresh this project"
        >
            <RotateCw className={cn("w-3 h-3", refreshingProjectId === project.project_id && "animate-spin text-cyan-400")} />
        </button>
        <span className="w-px h-3 bg-slate-700/50" />
        <button
            type="button"
            onClick={(e) => togglePinProject(project.project_id, e)}
            className={cn(
                "p-1 rounded-full transition-colors cursor-pointer",
                isPinned ? "text-amber-400 bg-amber-400/10" : "text-slate-400 hover:text-amber-400 hover:bg-slate-800"
            )}
            title={isPinned ? "Unpin project" : "Pin project to top"}
        >
            <Pin className="w-3 h-3" />
        </button>
    </div>
</div>
```

#### 3. Conversation Node Status Badges
When rendering individual conversation rows:

```tsx
{conv.is_running ? (
    <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[8.5px] font-bold font-mono bg-emerald-500/15 text-[#1af18d] border border-emerald-500/30">
        <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
        <span>RUNNING</span>
    </span>
) : conv.is_queued ? (
    <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[8.5px] font-bold font-mono bg-amber-500/15 text-amber-400 border border-amber-500/30">
        <span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" />
        <span>QUEUED</span>
    </span>
) : (
    <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[8.5px] font-mono text-slate-400 bg-slate-800/60 border border-slate-700/40">
        DONE
    </span>
)}
```

---

### 3.3 Live Execution Results in Preview Pane (`PromptTreeViewModal.tsx`)

When a user selects an active conversation or running prompt in the tree view, the right-hand preview pane must display a live progress status bar showing current execution step and tool output:

```tsx
{/* Live Execution Activity Banner */}
{(selectedConversation?.is_running || selectedConversation?.latest_step_summary) && (
    <div className="mb-3 p-2.5 rounded-lg border border-emerald-500/30 bg-emerald-950/20 backdrop-blur-md flex flex-col gap-1.5 shadow-sm">
        <div className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-2">
                <span className="relative flex h-2.5 w-2.5">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#1af18d] opacity-75" />
                    <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-[#1af18d] shadow-[0_0_8px_rgba(26,241,141,1)]" />
                </span>
                <span className="font-bold text-[#1af18d] tracking-wide uppercase text-[10px]">
                    Live Execution Telemetry
                </span>
            </div>
            <span className="text-[10px] font-mono text-emerald-400/80">
                Steps: {selectedConversation?.step_count || 1}
            </span>
        </div>
        
        {selectedConversation?.latest_step_summary ? (
            <div className="flex items-center gap-1.5 text-xs text-slate-200 font-mono bg-slate-900/60 px-2 py-1 rounded border border-slate-800">
                <Terminal className="w-3.5 h-3.5 text-cyan-400 shrink-0" />
                <span className="truncate">{selectedConversation.latest_step_summary}</span>
            </div>
        ) : (
            <div className="text-[11px] text-slate-400 italic">
                In-flight agent activity detected. Gathering transcript stream...
            </div>
        )}
    </div>
)}
```

---

### 3.4 Hardened "Focus IDE Window" with Target Workspace Matching

#### 1. Frontend Command Dispatch
In `src/components/instances/PromptTreeViewModal.tsx`:

```typescript
const handleFocusWorkspace = async (instanceId: string, repoPath: string, repoName: string) => {
    try {
        setBusyFocusing(true);
        // Call backend hardened focus command passing repo target
        await invoke('focus_instance_workspace', {
            instanceId,
            repoPath,
            repoName,
        });
        showToast(`Brought ${repoName} to the foreground`, 'success');
    } catch (err: any) {
        showToast(`Failed to focus IDE: ${err?.message || err}`, 'error');
    } finally {
        setBusyFocusing(false);
    }
};
```

#### 2. Backend Command Binding (`src-tauri/src/commands/instance.rs`)
```rust
#[tauri::command]
pub fn focus_instance_workspace(
    instance_id: String,
    repo_path: String,
    repo_name: String,
) -> Result<bool, crate::error::AppError> {
    crate::modules::instance::focus_or_launch_workspace(&instance_id, &repo_path, &repo_name)
}
```

#### 3. Core Engine Implementation (`src-tauri/src/modules/instance.rs`)
```rust
pub fn focus_or_launch_workspace(
    instance_id: &str,
    repo_path: &str,
    repo_name: &str,
) -> Result<bool, crate::error::AppError> {
    let registry = load_registry().map_err(crate::error::AppError::Config)?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| crate::error::AppError::Config(format!("Instance {} not found", instance_id)))?;

    let is_default_inst = inst.is_default || inst.id == "default";
    let pids = find_pids_for_data_dir(&inst.data_dir, is_default_inst);

    if !pids.is_empty() {
        // Attempt exact window focus by matching window title containing repo_name
        let focused = crate::modules::process::focus_instance_workspace_window(&pids, repo_name);
        if focused {
            return Ok(true);
        }
    }

    // If window not focused, launch/open target instance with explicit workspace folder
    crate::modules::logger::log_info(&format!(
        "[Instance] Launching instance '{}' targeted at workspace folder: {}",
        instance_id, repo_path
    ));
    launch_instance_with_workspaces(instance_id, Some(&[repo_path.to_string()]), true)?;
    Ok(false)
}
```

---

## 4. Acceptance Criteria & Test Verification

- [ ] **Project Header Indicators:** Every project node with active tasks displays `{runningCount} RUNNING` in emerald and `{queuedCount} QUEUED` in amber wrapped in a contiguous capsule.
- [ ] **Repeat Group Inheritance:** Grouped conversations inherit `is_queued = true` and `status = "QUEUED"` when any underlying sub-run is queued and none are running.
- [ ] **Unified Pulse Styling:** `Instances.tsx` and `PromptTreeViewModal.tsx` share the `#1af18d` emerald color token and smooth CSS keyframe pulse.
- [ ] **Preview Telemetry Display:** Active conversations display the live step index, tool name, and status summary in the preview pane.
- [ ] **Exact Window Foregrounding:** Clicking "Focus IDE" brings the specific VSCode/Antigravity window for that project to the top of the window stack without flashing or creating duplicate windows.
