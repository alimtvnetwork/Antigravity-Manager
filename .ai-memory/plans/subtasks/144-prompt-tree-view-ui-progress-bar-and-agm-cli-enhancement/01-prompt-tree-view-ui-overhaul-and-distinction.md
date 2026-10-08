# Subtask 01: Prompt Tree View UI Overhaul & Prompt Type Distinction

**Slug:** `144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement`  
**File:** `.ai-memory/plans/subtasks/144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/01-prompt-tree-view-ui-overhaul-and-distinction.md`  
**Target Component:** `src/components/instances/PromptTreeViewModal.tsx`  
**Related Specs:** `02-spec/21-app/144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/01-architecture-spec.md`  
**Status:** COMPLETED & VERIFIED  
**Lead Author:** Antigravity Architect (Author 01)  

---

## 1. Objective

Overhaul `src/components/instances/PromptTreeViewModal.tsx` to provide an intuitive, high-density, and visually cohesive prompt analysis interface adhering to the dark-glass segmented pill capsule design system in `AGENTS.md`. Specifically:
1. Reorganize the loose header buttons and window controls into unified, contiguous segmented dark-glass capsules.
2. Implement heuristic prompt categorization distinguishing direct human requests (`User Prompt`) from autonomous system or subagent directives (`AI Subagent Instruction`).
3. Add real-time emerald pulse indicators and active running count badges on parent project cards and conversation nodes.
4. Deduplicate and group repeated prompts using hash-based grouping, rendering an `xN` frequency badge with collapsible sub-run execution history.
5. Replace raw text truncation with a high-authority byte-level truncation callout banner: `⚡ [Truncated: X.X KB preserved - Click to Expand Full Text]`.
6. Add a dedicated `Export` button supporting single-prompt downloads in Markdown (`.md`) and JSON (`.json`) formats alongside clean `Copy Text` and raw `Copy with Images` actions.

---

## 2. Target Files

- `src/components/instances/PromptTreeViewModal.tsx` (primary UI component, ~2,870 lines)
- `src/services/instanceService.ts` (if type annotations or export helpers require extension)

---

## 3. Detailed Implementation Specifications

### 3.1 Header Toolbar Reorganization into Segmented Pill Capsules

#### Problem:
Currently, lines 1845–1910 render toolbar actions using disjointed `rounded-[4px]` buttons and a detached circular Close button on the right, violating the `AGENTS.md` pill capsule invariant.

#### Solution:
Replace disjointed containers with a contiguous dark-glass capsule. Both operational actions (Backup, Restore, Refresh, Sync Selector, Fullscreen) and window control (Close) are consolidated into a cohesive pill capsule:

```tsx
{/* Unified Header Toolbar Segmented Pill Capsule */}
<div className="flex items-center gap-2">
    <div className="inline-flex items-center rounded-full bg-slate-100/80 dark:bg-[#0c2438]/85 border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/60 dark:divide-[#15334d]/70 shadow-xs backdrop-blur-md">
        {/* 1. Backup */}
        <button
            type="button"
            onClick={handleBackup}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/70 dark:hover:bg-[#15334d]/80 rounded-l-full transition-colors cursor-pointer active:scale-[0.98]"
            title="Backup Prompts to JSON"
        >
            <Download className="w-3.5 h-3.5 text-indigo-500" />
            <span>Backup</span>
        </button>

        {/* 2. Restore */}
        <button
            type="button"
            onClick={handleRestore}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/70 dark:hover:bg-[#15334d]/80 transition-colors cursor-pointer active:scale-[0.98]"
            title="Restore Running Prompts (7s stabilization delay)"
        >
            <Upload className="w-3.5 h-3.5 text-emerald-500" />
            <span>Restore</span>
        </button>

        {/* 3. Refresh */}
        <button
            type="button"
            onClick={() => loadTree(true, true)}
            disabled={isLoading}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/70 dark:hover:bg-[#15334d]/80 transition-colors cursor-pointer active:scale-[0.98] disabled:opacity-50"
            title="Refresh Prompt Tree"
        >
            <RefreshCw className={cn('w-3.5 h-3.5 text-blue-500', isLoading && 'animate-spin')} />
            <span>Refresh</span>
        </button>

        {/* 4. Sync Interval Selector */}
        <div className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200">
            <Clock className={cn("w-3.5 h-3.5 text-cyan-500", isAutoSyncing && "animate-spin")} />
            <span className="text-[11px] text-slate-500 dark:text-slate-400">Sync:</span>
            <select
                value={syncInterval}
                onChange={(e) => {
                    const val = e.target.value as SyncInterval;
                    setSyncInterval(val);
                    setPromptTreeSyncInterval(val);
                }}
                className="rounded-full bg-white dark:bg-[#071a27] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] text-xs px-2 py-0.5 focus:outline-none focus:ring-1 focus:ring-cyan-500 cursor-pointer"
                title="Auto-sync interval"
            >
                <option value="15s">15s</option>
                <option value="30s">30s</option>
                <option value="1m">1m</option>
                <option value="2m">2m</option>
                <option value="off">Off</option>
            </select>
        </div>

        {/* 5. Fullscreen Toggle */}
        <button
            type="button"
            onClick={() => setIsFullscreen(!isFullscreen)}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/70 dark:hover:bg-[#15334d]/80 transition-colors cursor-pointer active:scale-[0.98]"
            title={isFullscreen ? 'Exit Full Screen' : 'Full Screen Mode'}
        >
            {isFullscreen ? (
                <>
                    <Minimize2 className="w-3.5 h-3.5 text-blue-500" />
                    <span>Exit</span>
                </>
            ) : (
                <>
                    <Maximize2 className="w-3.5 h-3.5 text-blue-500" />
                    <span>Full</span>
                </>
            )}
        </button>

        {/* 6. Window Close Action */}
        <button
            type="button"
            onClick={onClose}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-rose-600 dark:text-rose-400 hover:bg-rose-100/80 dark:hover:bg-rose-950/40 rounded-r-full transition-colors cursor-pointer active:scale-[0.98]"
            title="Close Prompt Tree View (Esc)"
        >
            <X className="w-3.5 h-3.5" />
            <span>Close</span>
        </button>
    </div>
</div>
```

---

### 3.2 Prompt Type Discriminator Engine & Badges

#### Classification Rules:
Add a robust discriminator function `detectPromptCategory`:
- Checks for system tags (`<SYSTEM_MESSAGE>`, `<INSTRUCTION>`, `<USER_REQUEST>`, `<identity>`, `<RULE[...]>`, `<scratchpad>`).
- Checks for prompt header directives (`# High Priority Instruction`, `execute-parent-task-with-n-steps`, `plan-spec-steps-v2`, `Available skills:`, `Available subagents:`).
- Checks conversation title heuristics (`subagent`, `worker`, `author-`, `task-`).

#### Badge UI:
1. In conversation list items:
   ```tsx
   const promptCategory = detectPromptCategory(conv.prompt_preview_200w, conv.title);
   
   <span className={cn(
       "px-1.5 py-0.2 rounded-full text-[8.5px] font-mono font-bold uppercase tracking-wider shrink-0",
       promptCategory.category === 'SUBAGENT_INSTRUCTION'
           ? "bg-purple-500/15 text-purple-700 dark:text-purple-300 border border-purple-400/30"
           : "bg-sky-500/15 text-sky-700 dark:text-cyan-300 border border-sky-400/30"
   )}>
       {promptCategory.category === 'SUBAGENT_INSTRUCTION' ? 'Subagent' : 'User'}
   </span>
   ```
2. In prompt preview header:
   Render full prominent badge:
   - `User Prompt`: Sky pill badge with cyan border
   - `AI Subagent Instruction`: Purple pill badge with violet border

---

### 3.3 Running Pulse Indicators on Projects & Conversations

#### Project Card Summary:
In the project list renderer:
```tsx
const runningConversations = project.conversations.filter(c => c.is_running);
const runningCount = runningConversations.length;

{runningCount > 0 && (
    <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/30">
        <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)]"></span>
        </span>
        <span className="text-[9px] font-bold font-mono text-emerald-700 dark:text-[#1af18d]">
            {runningCount} RUNNING
        </span>
    </div>
)}
```

#### Conversation Node:
Active conversations display an animated green pulse beside their title and status badge:
```tsx
{conv.is_running && (
    <span className="flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[8.5px] font-bold font-mono bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40">
        <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
        RUNNING
    </span>
)}
```

---

### 3.4 Repeated Prompt Grouping & Deduplication

#### Grouping Logic:
1. Deduplicate consecutive or scattered identical prompts within a project.
2. Group key is computed via 32-bit FNV-1a hash over normalized prompt content.
3. Grouped results show the primary (latest) run with an amber repetition badge: `x4`.
4. Expanding the repetition badge reveals the historical sub-runs drawer with timestamps, step counts, and quick-inspect actions.

---

### 3.5 Byte-Level Truncation & Preview Indicators

#### Truncation Calculation:
- Compute exact UTF-8 byte count using `new TextEncoder().encode(prompt).length`.
- If prompt exceeds display word limit, render preview with preserved newlines.
- Display an interactive callout banner immediately below preview:
  `⚡ [Truncated: X.X KB preserved - Click to Expand Full Text]`
- Clicking the banner or button toggles `isExpandedFullPrompt` state, rendering the complete text in an expandable drawer or dedicated view mode.

---

### 3.6 Dedicated Export Button (.md / .json)

#### Implementation:
Add an `Export` dropdown or button in the prompt preview action capsule:
- **Export as Markdown (`.md`):** Formats title, project metadata, sequence code, prompt category, timestamp, and verbatim prompt body into a downloadable `.md` file.
- **Export as JSON (`.json`):** Dumps complete structured payload (`{ id, conversation_id, project_id, category, timestamp, prompt_text, step_count }`) as formatted JSON.
- Trigger file download using browser `Blob` and anchor download mechanism without requiring extra backend IPC.

---

## 4. Acceptance Criteria & Quality Gates

1. **Toolbar Capsule Compliance:** Zero loose square buttons in the header; all actions encapsulated in dark-glass `rounded-full` pills.
2. **Discriminator Accuracy:** Prompts with `<SYSTEM_MESSAGE>`, `<identity>`, or task specifications correctly render `AI Subagent Instruction`. Direct user messages correctly render `User Prompt`.
3. **Pulse Visibility:** Any in-flight conversation immediately reflects on the parent project card with the emerald pulse orb and count badge.
4. **Grouped Prompts:** Duplicate prompts group cleanly into single entries with `xN` badges, preventing clutter.
5. **Accurate Byte Truncation:** Truncation displays exact preserved byte footprint (e.g. `24.5 KB preserved`) with immediate full expansion.
6. **Export Verification:** Clicking `Export` downloads clean `.md` or `.json` containing exact prompt content.
