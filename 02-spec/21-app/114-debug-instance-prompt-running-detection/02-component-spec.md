# 114 Component Specification: Frontend Instance Prompt Detection & Test Suite

## 1. Executive Summary & Purpose

This specification governs the frontend state synchronization, component presentation, and integration testing architecture for **Task 114: Debug Instance Prompt Running Detection**.

### 1.1 Problem Statement
In previous iterations, the user observed severe false-positive running indicators across instances:
1. **Default Instance**: The default profile was actively executing prompts only within `Antigravity-Manager`. However, dormant projects such as `SpecBuilder` and `coding-guidelines` falsely displayed active running badges.
2. **Instance 8159 (`default-copy-8159`)**: Instance 8159 was actively executing prompts only within `coding-guidelines`. However, cloned/dormant projects such as `Antigravity-Manager` and `SpecBuilder` falsely displayed active running badges.

### 1.2 Root Cause Synthesis
- The frontend relied on a flattened tree query that merged candidate directories across all instances without enforcing instance affinity.
- Projects copied during instance cloning inherited historical conversation records containing `RUNNING` or `not_fully_idle` flags from prior sessions or other instances.
- UI mapping logic in `src/pages/Instances.tsx` evaluated `isProjRunning` using loose conditions where any historical conversation matching a repository path or receiving an un-scoped running signal triggered an affirmative `RUNNING` badge.
- When `PromptTreeViewModal.tsx` opened, instance filtering was permissive, allowing projects from foreign profiles to bleed into view when `instanceId` was ambiguous or defaulted.

### 1.3 Target Architecture
The resolution strictly establishes:
- **Affirmative Per-Instance Scoping**: The frontend and backend communicate using explicit `instance_id` parameters.
- **Process-Gated Affirmative Running State**: A project or conversation can ONLY be considered running if the underlying operating system process for that specific instance is actively alive (`is_running = true`) AND there is an affirmative in-flight execution signal (`not_fully_idle != 0` or status `CASCADE_RUN_STATUS_RUNNING`).
- **Isolation of Copied Workspaces**: Cloned `workspaceStorage` directories remain dormant and cannot inherit running flags from other profiles.
- **Defensive UI Rendering & Clean Empty States**: Elimination of erroneous glowing pulses on dormant projects and addition of contextual empty states in the prompt tree modal.

---

## 2. Frontend Component Specifications

### 2.1 `src/pages/Instances.tsx`

#### 2.1.1 `fetchRunningTasks` Execution and Polling Contract
The `fetchRunningTasks` function runs during initial mount, on a 3-second background polling cycle, and reactively upon receiving Tauri events (`prompt://dispatched`, `prompt://resumed`, `instance://switched`, `account://auto-switched`).

```typescript
const fetchRunningTasks = async () => {
    try {
        const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
            maxWords: 50,
            onlyRunning: false,
            force: false,
        });
        if (Array.isArray(data)) {
            setProjectTreeNodes(data);
            const running = data.filter(
                (node) => Boolean(node.is_running) || Boolean(node.conversations?.some((c) => Boolean(c.is_running) || c.status === 'RUNNING'))
            );
            setRunningTreeNodes(running);
        }
    } catch {
        // Silently ignore background polling error
    }
};
```

#### 2.1.2 Per-Instance Project & Running Task Mapping
In `Instances.tsx`, each instance card renders its active projects and determines overall instance task activity. The mapping logic must strictly respect instance isolation:

1. **Card Header Active Task Detection (`hasActiveTask`)**:
   An instance card indicates active task execution only if:
   - The instance itself is running (`Boolean(inst.is_running) === true`).
   - At least one node in `runningTreeNodes` matches this specific instance and contains an affirmatively running task.

   ```typescript
   const hasActiveTask = Boolean(inst.is_running) && runningTreeNodes.some((node) => {
       const isInstanceMatch = inst.config.is_default
           ? (node.instance_id === 'default' || node.instance_id === '__default__' || !node.instance_id || node.instance_id === inst.config.id)
           : node.instance_id === inst.config.id;
       const isNodeRunning = Boolean(node.is_running) || Boolean(node.conversations?.some((c) => Boolean(c.is_running) || c.status === 'RUNNING'));
       return isInstanceMatch && isNodeRunning;
   });
   ```

2. **Scoped Project Filtering (`instanceProjects`)**:
   Projects listed in the card's "Active / Recent Projects" container must strictly belong to the instance:
   - For the default instance: matches `node.instance_id === 'default'`, `node.instance_id === '__default__'`, missing `instance_id`, or `node.instance_id === inst.config.id`.
   - For non-default instances: strictly matches `node.instance_id === inst.config.id`. Projects from default or other sandboxes are excluded.

   ```typescript
   const instanceProjects = projectTreeNodes.filter((node) => {
       if (inst.config.is_default) {
           return (
               node.instance_id === 'default' ||
               node.instance_id === '__default__' ||
               !node.instance_id ||
               node.instance_id === inst.config.id
           );
       }
       return node.instance_id === inst.config.id;
   });
   ```

#### 2.1.3 Elimination of False Positive `isProjRunning` Badges
Prior implementation permitted dormant projects to display the glowing `RUNNING` badge whenever `inst.is_running` was true and either `proj.is_running` was stale or any conversation had `status === 'RUNNING'`.

The strict condition for `isProjRunning` requires:
1. `inst.is_running` must be affirmative (`true`). If the IDE process is stopped, no project can be running.
2. The project must have affirmative live activity:
   - `proj.is_running === true` as computed by backend process verification.
   - OR at least one conversation in `proj.conversations` has `c.is_running === true`.
3. Stale conversations or completed turns (`status === 'IDLE'`, `status === 'COMPLETED'`, or `not_fully_idle === 0`) must never activate the badge.

```typescript
const isProjRunning = Boolean(inst.is_running) && (
    Boolean(proj.is_running) || 
    Boolean(proj.conversations?.some((c) => Boolean(c.is_running)))
);
```

Rendering specification:
- When `isProjRunning === true`:
  Render badge:
  ```tsx
  <span className="px-1 py-0.2 rounded-[4px] text-[9px] font-bold bg-cyan-500/15 text-cyan-600 dark:text-cyan-300 border border-cyan-400/30 flex items-center gap-0.5">
      <span className="w-1 h-1 rounded-full bg-cyan-500 animate-pulse" />
      RUNNING
  </span>
  ```
- When `isProjRunning === false`:
  The `RUNNING` badge is completely omitted. Only the turns count badge (`{totalTurns} turns`) is displayed.

---

### 2.2 `src/components/instances/PromptTreeViewModal.tsx`

#### 2.2.1 Scoped Backend Query and Filtering
When opened from an instance card or double-clicking a project:
1. `PromptTreeViewModal` passes `instanceId: instanceId || undefined` to Tauri command `get_project_conversation_tree`.
2. The response is filtered with strict instance matching:

```typescript
const isTargetDefault = instanceId === 'default' || instanceId === '__default__';
const relevant = instanceId
    ? data.filter((p) => {
          if (p.instance_id === instanceId) return true;
          if (isTargetDefault && (!p.instance_id || p.instance_id === 'default' || p.instance_id === '__default__')) {
              return true;
          }
          return false;
      })
    : data;
```

3. Any foreign project belonging to another instance (e.g., `default-copy-8159` data when inspecting `default`, or `default` data when inspecting `default-copy-8159`) is completely excluded from the tree model.

#### 2.2.2 Clean Empty States
`PromptTreeViewModal.tsx` must render clean, informative empty states rather than empty blank panes or unhandled null states:

1. **No Projects Found for Instance**:
   When `relevant.length === 0`:
   ```tsx
   <div className="py-16 text-center space-y-3 px-6">
       <div className="w-12 h-12 rounded-full bg-slate-100 dark:bg-[#0c2438] flex items-center justify-center mx-auto text-slate-400">
           <Folder className="w-6 h-6 text-slate-400" />
       </div>
       <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
           No Projects Found in this Profile
       </div>
       <p className="text-xs text-slate-500 dark:text-slate-400 max-w-sm mx-auto">
           No workspace storage or conversation records were found for instance "{instanceName || instanceId}". Launch this instance and open a workspace to start logging prompts.
       </p>
   </div>
   ```

2. **Search Query Zero Matches**:
   When active filter yields no matches:
   ```tsx
   <div className="py-12 text-center text-xs text-slate-400 space-y-2">
       <div>No conversations match "{searchQuery}"</div>
       <button
           onClick={() => setSearchQuery('')}
           className="text-blue-500 hover:underline text-[11px]"
       >
           Clear search query
       </button>
   </div>
   ```

3. **Empty Right Preview Pane**:
   When no conversation is selected or the active project has zero conversations:
   ```tsx
   <div className="flex-1 flex flex-col items-center justify-center text-slate-400 text-xs p-6 space-y-2">
       <MessageSquare className="w-8 h-8 text-slate-300 dark:text-slate-600" />
       <span>Select a conversation from the left hierarchy to inspect prompt history</span>
   </div>
   ```

---

## 3. Integration Testing Specification

### 3.1 Test Suite Location
File: `src-tauri/tests/per_instance_prompt_liveness_test.rs`
Execution: Standard cargo test or ignored e2e:
```bash
cargo test --test per_instance_prompt_liveness_test -- --nocapture
```

### 3.2 Fixture Architecture & Isolation
The integration test suite creates a temporary sandbox filesystem using `tempfile::TempDir`:
1. **Mock Home & Instance Storage**:
   - `temp_dir/home/.gemini/antigravity/` (Default instance Gemini storage)
   - `temp_dir/instances/default-copy-8159/home/.gemini/antigravity/` (8159 instance storage)
2. **Mock SQLite DBs**:
   Creates `conversation_summaries.db` in each profile with tables matching schema:
   ```sql
   CREATE TABLE conversation_summaries (
       conversation_id TEXT PRIMARY KEY,
       title TEXT,
       preview TEXT,
       status TEXT,
       not_fully_idle INTEGER,
       workspace_uris TEXT,
       last_modified_time TEXT
   );
   ```
3. **Mock Process Prober**:
   Injects or controls process liveness check so that `is_antigravity_running` or `is_instance_running` returns deterministically for test scenarios.

---

### 3.3 Test Case Specifications

#### 3.3.1 Test Case 1: Default Instance Running Antigravity-Manager Only
- **Context**: Default instance process is ALIVE.
- **Fixtures in Default DB**:
  - `conv-agm-1`: workspace `d:/work/Antigravity-Manager`, `not_fully_idle = 1`, `status = "CASCADE_RUN_STATUS_RUNNING"`.
  - `conv-spec-1`: workspace `d:/work/SpecBuilder`, `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`.
  - `conv-cg-1`: workspace `d:/work/coding-guidelines`, `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`.
- **Assertions**:
  1. `get_project_conversation_tree_cached(Some("default"), 50, false, true)` returns 3 projects.
  2. Project `Antigravity-Manager` has `is_running == true`.
  3. Project `SpecBuilder` has `is_running == false`.
  4. Project `coding-guidelines` has `is_running == false`.
  5. Filtering with `only_running = true` returns ONLY `Antigravity-Manager`.

#### 3.3.2 Test Case 2: Instance 8159 Running coding-guidelines Only
- **Context**: Instance 8159 process is ALIVE; Default instance process is IDLE/DEAD.
- **Fixtures in 8159 DB**:
  - `conv-8159-cg`: workspace `d:/work/coding-guidelines`, `not_fully_idle = 1`, `status = "CASCADE_RUN_STATUS_RUNNING"`.
  - `conv-8159-agm`: workspace `d:/work/Antigravity-Manager`, `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`.
  - `conv-8159-spec`: workspace `d:/work/SpecBuilder`, `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`.
- **Assertions**:
  1. `get_project_conversation_tree_cached(Some("default-copy-8159"), 50, false, true)` returns projects for 8159.
  2. Project `coding-guidelines` has `is_running == true`.
  3. Project `Antigravity-Manager` has `is_running == false`.
  4. Project `SpecBuilder` has `is_running == false`.
  5. None of Default instance's conversations bleed into 8159 tree nodes.

#### 3.3.3 Test Case 3: Copied/Dormant `workspaceStorage` Folders Do Not Activate False `is_running`
- **Context**: Instance 8159 cloned Default instance's entire workspace directory, including historical state. Default instance has an actively running conversation in `Antigravity-Manager`.
- **Fixtures**:
  - In Default DB: `conv-agm-live`: `Antigravity-Manager`, `not_fully_idle = 1`, `status = "CASCADE_RUN_STATUS_RUNNING"`.
  - In 8159 DB: Copied record `conv-agm-dormant`: `Antigravity-Manager`, `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`.
- **Assertions**:
  1. When evaluating 8159, `Antigravity-Manager` MUST report `is_running == false`.
  2. The running state of `Antigravity-Manager` on the default instance MUST NOT propagate to 8159.
  3. Keying of conversations MUST be qualified by `(instance_id, normalized_repo_path)`, never by `normalized_repo_path` alone.

#### 3.3.4 Test Case 4: Process Termination Forces `is_running = false`
- **Context**: The database records indicate `not_fully_idle = 1` and `status = "RUNNING"` (e.g. IDE was forcefully killed, crashed, or closed while running).
- **Condition**: Process probe verifies the instance process is NOT running (`is_inst_alive == false`).
- **Assertions**:
  1. Project tree node MUST report `is_running == false`.
  2. Every conversation node in `conv_nodes` MUST report `is_running == false` and status `IDLE` (or original status with `is_running = false`).
  3. Audit log line emitted with `INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle`.
  4. UI `isProjRunning` evaluates strictly to `false` because `inst.is_running` is false.

---

## 4. Structured Audit Logging Verification

Every liveness evaluation must emit a machine-parseable structured log entry to `app.log`:
```text
[PROMPT_LIVENESS_PROBE][PROJECT] instance_id="{id}" project="{name}" is_running={bool} rationale="{rationale}"
```

Allowed rationales:
- `INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle`
- `ACTIVE_IN_FLIGHT_TASKS: active prompt or non-idle conversation detected -> marked running`
- `IDLE: process alive but no in-flight tasks or active conversations -> marked idle`

These log lines provide unambiguous operational telemetry for debugging and ensure automated root cause auditing without relying on guesswork.
