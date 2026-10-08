# Component and CLI Specification: Prompt Dispatch, Process Verification & Tag Compaction

- **Spec ID:** `02-spec/21-app/147-smart-instance-process-cache-and-prompt-dispatch/02-component-and-cli-spec.md`
- **Milestone:** Task 147 — Smart Instance Process Cache & Prompt Dispatch
- **Target Subtasks:** Subtask 02 (UI & Running Detection Hardening), Subtask 03 (CLI & E2E Testing)
- **Status:** `APPROVED`
- **Author:** Author 02 (UI Component, CLI Parity & E2E Testing Specialist)

---

## 1. Executive Summary & Architectural Overview

This engineering specification establishes the end-to-end component design, CLI verb contracts, and automated verification suites for **Task 147**. The implementation addresses three fundamental pain points identified in real-world usage:

1. **Unwanted IDE Relaunch on Prompt Send/Enqueue:** Sending or queueing a prompt previously triggered unconditional process spawn logic (`focus_or_launch_instance_with_workspace`), causing window flickering, duplicate instance spawning, or crashing already running Antigravity IDE instances. A smart process cache coupled with active PID liveness verification intercepts all dispatch routes to send directly to running instances without relaunching.
2. **Visual Clutter in Prompt Tree View:** Excessive square bracket tagging (`[AGM:P001 | GM:#1]`, `[AGM:C001 | GM:<cid>]`), redundant uppercase role badges (`USER`, `SUBAGENT`, `SYSTEM`, `TOOL`) cluttering every row, and disconnected action buttons violating the design system's segmented capsule guidelines.
3. **Pervasive False-Positive "RUNNING" Statuses:** Idle or completed conversations displaying persistent green `● RUNNING` badges due to un-synchronized database states and superficial status queries.

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                 DISPATCH & UI ARCHITECTURE                             │
│                                                                                        │
│   GUI / CLI / REST Dispatch Request                                                    │
│       │                                                                                │
│       ▼                                                                                │
│   ┌──────────────────────────────────────────────────────────────┐                     │
│   │ Smart Instance Process Cache & Double-Check Liveness Engine  │                     │
│   └──────────────────────────────┬───────────────────────────────┘                     │
│                                  │                                                     │
│                  ┌───────────────┴───────────────┐                                     │
│                  ▼                               ▼                                     │
│         [PID Alive & Running]            [PID Stale / Dead]                            │
│                  │                               │                                     │
│         Direct IPC / Resume Task         Double-Check Process Table                    │
│         (NO Process Termination,                 │                                     │
│          NO Window Relaunch)             ┌───────┴───────┐                             │
│                  │                       ▼               ▼                             │
│                  ▼                  [Found Active]   [Confirmed Dead]                  │
│         Focus Running Window        Update Cache     Clean Launch & Register           │
│                  │                       │               │                             │
│                  └───────────────────────┼───────────────┘                             │
│                                          ▼                                             │
│                           FIFO Prompt Queue Storage                                    │
│                           (ORDER BY created_at ASC)                                    │
│                                          │                                             │
│                                          ▼                                             │
│                           Compacted Prompt Tree View                                   │
│                           (P001 · #1, C001 · <cid>, 4-Tier Icons,                      │
│                            Segmented Dark-Glass Capsules)                              │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Component Specification: Compacted Prompt Tree View & Inspector

Target File: `src/components/instances/PromptTreeViewModal.tsx`

### 2.1 Compacted Dual Sequence Badges
The current tree renders noisy bracketed tokens that distract from prompt hierarchy:
- **Legacy Format:** `[AGM:P001 | GM:#1]` and `[AGM:C001 | GM:3f8e12a0]`
- **Compacted Format:** `P001 · #1` and `C001 · 3f8e12a0`

#### Formatting Specification:
```typescript
/**
 * Formats compacted dual sequence badges without outer brackets or redundant prefixes.
 * Project Level:      P001 · #1
 * Conversation Level: C001 · 3f8e12a0
 */
export function formatCompactedDualBadge(
    seqCode: string | undefined,
    defaultSeq: string,
    gmCode: string | undefined,
    defaultGm: string
): string {
    const primary = (seqCode || defaultSeq).replace(/^AGM:/i, '').trim();
    let secondary = (gmCode || defaultGm).replace(/^GM:/i, '').trim();
    if (secondary.startsWith('#')) {
        secondary = secondary.substring(1);
    }
    return `${primary} · #${secondary}`;
}

export function formatCompactedConvBadge(
    seqCode: string | undefined,
    defaultSeq: string,
    gmShortId: string | undefined
): string {
    const primary = (seqCode || defaultSeq).replace(/^AGM:/i, '').trim();
    const secondary = (gmShortId || '').replace(/^GM:/i, '').trim();
    return secondary ? `${primary} · ${secondary}` : primary;
}
```

#### Visual Styling:
- Enclosed in an inline monospace container: `font-mono text-[9.5px] font-medium tracking-tight px-1.5 py-0.5 rounded-[4px]`.
- Subtle dark-glass styling in dark mode: `bg-slate-200/80 dark:bg-[#15334d]/80 text-slate-700 dark:text-cyan-300 border border-slate-300/50 dark:border-[#1e4468]/60`.
- Eliminated all enclosing brackets (`[`, `]`).

---

### 2.2 Tree Item Row Tag Compaction & Role Iconography
To eliminate row clutter while maintaining strict origin classification, explicit uppercase text badges (`USER`, `SUBAGENT`, `SYSTEM`, `TOOL`) are removed from row list items. The 4-tier taxonomy is preserved via distinct iconography and theme-aligned chromatic styling:

| Origin Tier | Row Icon | Icon Color (Light / Dark) | Semantic Role |
| :--- | :--- | :--- | :--- |
| `USER_PROMPT` | `User` (14px) | `text-sky-600` / `text-sky-400` | Human operator prompt |
| `SUBAGENT_INSTRUCTION` | `Bot` (14px) | `text-purple-600` / `text-purple-400` | Autonomous delegated subagent |
| `SYSTEM_MESSAGE` | `Terminal` (14px) | `text-zinc-600` / `text-zinc-400` | System injection / daemon hook |
| `TOOL_OUTPUT` | `Wrench` (14px) | `text-amber-600` / `text-amber-400` | Tool call output / MCP result |

#### Tree Item Row Layout Blueprint:
```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│ [Icon] [C001 · 3f8e12a0]  Fix smart process cache logic...    [3 stp] [● RUNNING 0:42]│
└───────────────────────────────────────────────────────────────────────────────────────┘
```
- **Left Group:**
  1. Origin Icon (14x14px, colored by tier).
  2. Compacted Dual Sequence Badge (`C001 · 3f8e12a0`).
  3. Truncated conversation title / prompt snippet (11px, flex-1 truncate).
  4. Repeat Badge (`x2`, `x3`) rendered only if repeated execution detected.
- **Right Group:**
  1. Step counter pill (`3 stp`).
  2. Status Indicator:
     - Running: Emerald pulse badge (`● RUNNING mm:ss`).
     - Queued: Amber badge (`⏳ QUEUED`).
     - Completed/Idle: Clean without trailing status tag.

---

### 2.3 Segmented Dark-Glass Capsules for Action & Inspector Bars
In accordance with `AGENTS.md` UI standards, adjacent toolbar buttons and control triggers must be unified into continuous segmented pill capsules rather than isolated circular buttons.

#### Design Invariants:
- Shared outer border: `rounded-full border border-slate-200/80 dark:border-[#15334d]/90`.
- Segmented dividers: `divide-x divide-slate-200/70 dark:divide-[#15334d]/80`.
- Dark-glass backdrop: `bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md shadow-2xs`.
- Transparent embedded `<select>` menus: `bg-transparent border-0 focus:ring-0 focus:outline-none text-xs font-semibold cursor-pointer`.

#### Header Controls:
Split into two distinct capsules:
```tsx
{/* Capsule 1: Data Operations */}
<div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80">
  <button onClick={handleBackup} className="px-3 py-1.5 text-xs font-semibold hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-l-full">
    <Download className="w-3.5 h-3.5 text-indigo-500" /> Backup
  </button>
  <button onClick={handleRestore} className="px-3 py-1.5 text-xs font-semibold hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90">
    <Upload className="w-3.5 h-3.5 text-emerald-500" /> Restore
  </button>
  <button onClick={handleRefresh} className="px-3 py-1.5 text-xs font-semibold hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90">
    <RotateCw className="w-3.5 h-3.5 text-cyan-500" /> Refresh
  </button>
  <div className="px-2.5 py-1 text-xs">
    <select value={syncInterval} onChange={handleSyncChange} className="bg-transparent border-0 focus:ring-0 focus:outline-none">
      <option value="off">Sync: Off</option>
      <option value="15s">Sync: 15s</option>
      <option value="30s">Sync: 30s</option>
    </select>
  </div>
</div>

{/* Capsule 2: Window Controls */}
<div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80">
  <button onClick={toggleFullscreen} className="px-2.5 py-1.5 text-xs hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-l-full">
    {isFullscreen ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
  </button>
  <button onClick={onClose} className="px-2.5 py-1.5 text-xs hover:bg-rose-500/20 text-slate-600 dark:text-slate-300 hover:text-rose-600 dark:hover:text-rose-400 rounded-r-full">
    <X className="w-3.5 h-3.5" />
  </button>
</div>
```

---

### 2.4 Live In-Flight Execution HUD Banner
When the selected prompt is actively executing, the inspector displays a prominent, low-profile status banner:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ ● RUNNING (01:24) · Step 4 · gemini-3.8-flash-high       [ ↗ Focus Antigravity Window ]│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

#### Specifications:
- **Visuals:** Emerald accent border with subtle gradient backdrop (`bg-emerald-500/10 dark:bg-emerald-950/30 border-emerald-500/30`).
- **Telemetry:** Active PID indicator, live ticker (`mm:ss`), current step count, active model designation.
- **Focus Action:** "Focus Antigravity Window" button triggers `focus_instance_window(instance_id)` via Tauri invoke, bringing the live running window to the foreground without restarting or sending conflicting keystrokes.

---

## 3. CLI Specification: Antigravity-Manager Native Terminal CLI (`agm`)

Target Files: `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/cli.rs`

The native CLI supports comprehensive parity with GUI operations. All prompt commands conform to the standard `CliEnvelope<T>` JSON envelope when invoked with `--json`.

```json
{
  "status": "success",
  "command": "prompt send",
  "instance_id": "default",
  "timestamp": 1728456789,
  "data": { ... }
}
```

### 3.1 Command Contract Matrix

| Command Verb | Arguments | Flags | Description |
| :--- | :--- | :--- | :--- |
| `agm prompt send` | `<instance_id> <prompt_text>` | `--repo`, `--model`, `--prefix`, `--suffix`, `--json` | Direct dispatch to running IDE instance; verifies PID before fallback launch |
| `agm prompt queue` | `<instance_id> <prompt_text>` | `--repo`, `--model`, `--fifo`, `--json` | Enqueue prompt into FIFO backlog with status `queued` |
| `agm prompt running` | `[<instance_id>]` | `--instance`, `--all`, `--json` | Query active running prompts with hardened false-positive filtering |
| `agm instance status` | `<instance_id>` | `--json`, `--verbose` | Deep status check returning cached PID, OS liveness, memory, and queue stats |

---

### 3.2 Verb: `agm prompt send`

#### Usage:
```bash
agm prompt send <instance_id> "<prompt_text>" [options]
```

#### Operational Workflow:
1. Parse `instance_id` and prompt text from CLI arguments.
2. Query `instance_processes` SQLite cache to obtain cached PID.
3. Perform OS process table verification:
   - Check if cached PID exists and matches `Antigravity` executable name and instance `data_dir`.
4. If process is **ALIVE**:
   - Write task payload to `<repo_path>/.antigravity_resume_task.json`.
   - Update SQLite `active_prompts` with status `in_flight`.
   - Forward via local REST `/prompts/dispatch` or IPC window trigger.
   - **CRITICAL:** Do NOT invoke `launch_instance` or terminate existing process.
5. If process is **NOT ALIVE**:
   - Double-check system process table to prevent false cache miss.
   - If confirmed completely dead, spawn IDE instance with correct `--user-data-dir`.
   - Record new child PID in `instance_processes`.
   - Dispatch prompt payload.

#### Output Example (`--json`):
```json
{
  "status": "success",
  "command": "prompt send",
  "instance_id": "profile-alpha",
  "timestamp": 1728457000,
  "data": {
    "prompt_id": "prompt-a1b2c3d4",
    "instance_id": "profile-alpha",
    "repo_path": "d:/work/Antigravity-Manager",
    "status": "in_flight",
    "cached_pid": 14280,
    "process_action": "cache_hit_dispatched"
  }
}
```

---

### 3.3 Verb: `agm prompt queue`

#### Usage:
```bash
agm prompt queue <instance_id> "<prompt_text>" [options]
```

#### Operational Workflow:
1. Parse target instance ID and payload.
2. Insert prompt into `active_prompts` table with:
   - `status = 'queued'`
   - `created_at = Utc::now().timestamp()`
   - `updated_at = Utc::now().timestamp()`
3. Ensure FIFO queue ordering invariant: all dispatch retrieval queries enforce `ORDER BY created_at ASC, id ASC`.
4. Emit notification to instance event hub.

#### Output Example (`--json`):
```json
{
  "status": "success",
  "command": "prompt queue",
  "instance_id": "profile-alpha",
  "timestamp": 1728457100,
  "data": {
    "prompt_id": "prompt-e5f6g7h8",
    "instance_id": "profile-alpha",
    "status": "queued",
    "queue_position": 1,
    "created_at": 1728457100
  }
}
```

---

### 3.4 Verb: `agm prompt running`

#### Usage:
```bash
agm prompt running [--instance <instance_id>] [options]
```

#### False-Positive Elimination Workflow:
1. Fetch all prompts from `active_prompts` marked `status IN ('running', 'in_flight', 'dispatched')`.
2. For each candidate prompt:
   - Check associated instance PID: if instance process is dead, mark prompt as `aborted` or `completed`.
   - Inspect conversation `transcript_full.jsonl` (reverse-scan last 10 lines):
     * If final records indicate completion (`TOKEN_USAGE`, idle state, user turn awaiting input), mark prompt as `completed`.
     * If active tool call without output or thinking block in-flight, preserve `running` status.
3. Return only genuine running prompts.

---

### 3.5 Verb: `agm instance status`

#### Usage:
```bash
agm instance status <instance_id> [options]
```

#### Output Example (`--json`):
```json
{
  "status": "success",
  "command": "instance status",
  "instance_id": "default",
  "timestamp": 1728457200,
  "data": {
    "id": "default",
    "name": "Default Instance",
    "status": "running",
    "pid": 14280,
    "os_process_verified": true,
    "uptime_seconds": 3840,
    "data_dir": "C:/Users/Administrator/AppData/Roaming/Antigravity",
    "active_prompts_count": 1,
    "queued_prompts_count": 0
  }
}
```

---

## 4. End-to-End Test Suite Specification

Target Script: `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`

An automated Python test harness validates the 5 core test cases. The test runner operates safely by inspecting and shielding all existing IDE processes before execution.

```
┌────────────────────────────────────────────────────────────────────────┐
│                      E2E VERIFICATION SUITE MATRIX                     │
├──────┬──────────────────────────────────────────┬──────────────────────┤
│ Case │ Target Invariant                         │ Verification Method  │
├──────┼──────────────────────────────────────────┼──────────────────────┤
│ TC01 │ Smart Process Cache Hit & No Relaunch    │ PID Preservation     │
│ TC02 │ Process Death Double-Check & Relaunch    │ Controlled Kill Check│
│ TC03 │ Prompt Enqueue IPC & FIFO Storage        │ SQLite Schema Audit  │
│ TC04 │ False-Positive Running Elimination       │ Transcript Parsing   │
│ TC05 │ UI Tag Compaction Verification           │ Regex Pattern Audit  │
└──────┴──────────────────────────────────────────┴──────────────────────┘
```

### Test Case 1: Smart Process Cache Hits & Relaunch Prevention
- **Objective:** Verify that sending a prompt to an active instance dispatches to the running process without killing or recreating it.
- **Preconditions:** Instance is launched with PID `P1`.
- **Execution:**
  1. Record initial PID `P1` via `agm instance status <id> --json`.
  2. Invoke `agm prompt send <id> "Hello autonomous prompt" --json`.
  3. Query `agm instance status <id> --json` again to record post-dispatch PID `P2`.
- **Assertion:**
  - `P1 == P2` (PID strictly preserved).
  - No new IDE processes spawned.
  - Resume task written to target workspace.

### Test Case 2: Process Death Double-Check & Graceful Relaunch
- **Objective:** Verify that relaunch occurs ONLY when the cached PID is genuinely dead in the OS process table.
- **Preconditions:** Instance is tracked in cache with dummy or terminated PID `999999`.
- **Execution:**
  1. Register stale PID in cache.
  2. Invoke `agm prompt send <id> "Prompt after restart" --json`.
  3. Confirm process detector queries OS table, detects dead PID, and triggers launch sequence.
- **Assertion:**
  - Double-check logic confirms PID death before launching.
  - New running PID `P3` is recorded in cache.

### Test Case 3: Prompt Enqueue IPC & FIFO Storage
- **Objective:** Verify prompt enqueuing stores records in FIFO order without triggering IDE launches.
- **Execution:**
  1. Enqueue prompt 1: `agm prompt queue <id> "Step 1" --json`.
  2. Enqueue prompt 2: `agm prompt queue <id> "Step 2" --json`.
  3. Query SQLite `active_prompts` where `status = 'queued'`.
- **Assertion:**
  - Both records exist with status `queued`.
  - Ordered strictly by `created_at ASC, id ASC`.
  - Zero IDE launch events triggered during enqueuing.

### Test Case 4: False Positive Running Elimination
- **Objective:** Verify that completed or idle conversations are never reported as `RUNNING`.
- **Execution:**
  1. Create synthetic completed transcript file with terminal user turn.
  2. Run `agm prompt running --instance <id> --json`.
- **Assertion:**
  - Completed conversation is excluded from running list.
  - Result list contains only genuinely in-flight prompts.

### Test Case 5: UI Tag Compaction Verification
- **Objective:** Validate that UI components do not contain uncompacted brackets or raw uppercase role badges.
- **Execution:**
  - Scan `src/components/instances/PromptTreeViewModal.tsx` using regex analysis:
    * Ensure `formatDualBadge` does not produce `[AGM:... | GM:...]`.
    * Ensure row list items do not contain `<span ...>USER</span>` or similar uppercase role badges.
    * Ensure segmented capsules implement `rounded-full` with shared borders.
- **Assertion:**
  - All 3 UI invariants pass with zero violations.

---

## 5. Implementation Roadmap & Guardrails

1. **Path Discipline:** Strictly utilize relative paths across all files and test assertions.
2. **Search Guardrail:** Total prohibition on `rg`, `ripgrep`, `grep`, `git grep`. Exclusively use GitMap (`gitmap find`, `gitmap aum search`).
3. **Safety Protection:** Test harness must never kill developer or user IDE instances outside the designated test sandbox instance.
4. **Attribution:** All release manifests and changelogs must strictly attribute `@aukgit`.
