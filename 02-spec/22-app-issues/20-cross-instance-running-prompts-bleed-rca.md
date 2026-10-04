# 20 — Cross-Instance Project and Running Prompt State Bleed RCA

## Executive Summary

In Antigravity Manager versions prior to Plan 113, visual telemetry and multi-instance deployments revealed critical state bleeding and false-positive liveness indicators across separate IDE profiles. Specifically:
- **Card Mirroring**: Instance Card #1 (Default Profile) and Instance Card #2 (8159 Profile) erroneously mirrored the exact same list of projects (`Antigravity-Manager`, `spec-builder`, `coding-guidelines`) with identical turn counts and conversation trees.
- **False Liveness Bleed**:
  - Instance Card #1 (Default Profile) showed `spec-builder` and `coding-guidelines` as `[RUNNING]` even though Card #1 was only running `Antigravity-Manager`.
  - Instance Card #2 (8159 Profile) showed `Antigravity-Manager` and `spec-builder` as `[RUNNING]` even though Card #2 was only running `coding-guidelines`.
  - When an instance was stopped, its projects frequently remained badged with `[RUNNING]` indicators.

This document presents a comprehensive, grounded 4-part Root Cause Analysis (RCA) detailing the exact architectural flaws across backend discovery and frontend rendering, the definitive architectural remedies implemented, and non-negotiable rules for all future AI agents and contributors.

---

## Part 1: Failure Symptoms & Visual Telemetry Analysis

### 1.1 Observed Symptoms (User Screenshots & Visual Verification)

Visual telemetry captured across multi-profile setups (`Sequence #1: Default Profile` and `Sequence #2: 8159 Profile`) revealed severe cross-contamination:

1. **Card #1 (Default Profile)**:
   - Ground Truth in OS: The user opened only `Antigravity-Manager` in the default Antigravity IDE instance.
   - UI Display: Displayed `Antigravity-Manager` (`12 turns`, `[RUNNING]`), `spec-builder` (`8 turns`, `[RUNNING]`), and `coding-guidelines` (`15 turns`, `[RUNNING]`).
   - Failure: Card #1 falsely displayed `spec-builder` and `coding-guidelines` as actively running tasks.

2. **Card #2 (8159 Profile)**:
   - Ground Truth in OS: The user opened only `coding-guidelines` in the dedicated 8159 instance (`--user-data-dir .../8159`).
   - UI Display: Displayed `Antigravity-Manager` (`12 turns`, `[RUNNING]`), `spec-builder` (`8 turns`, `[RUNNING]`), and `coding-guidelines` (`15 turns`, `[RUNNING]`).
   - Failure: Card #2 mirrored the projects of Card #1 and claimed `Antigravity-Manager` and `spec-builder` were actively running inside the 8159 profile.

3. **Prompt Tree Modal Scope Contamination**:
   - Opening the Prompt Tree View Modal from Card #2 displayed all global projects from the default instance rather than displaying strictly the projects hosted in the 8159 workspace.

4. **Ghost Running Badges on Stopped Instances**:
   - If an instance process was killed or gracefully stopped, project conversation items that had recent file modification timestamps within the last 10 minutes retained the `[RUNNING]` badge and header task pulsing dot.

---

## Part 2: Deep-Dive Root Causes (4 Core Flaws)

A rigorous codebase audit uncovered four distinct root architectural defects spanning both the Rust backend discovery pipeline and the React frontend state filtering.

### Flaw 1: Global Candidate Directory Aggregation Without Instance Boundary Isolation (Backend)

In `src-tauri/src/modules/repo_db.rs`, the core discovery routine `compute_project_conversation_tree` gathered workspace storage directories globally:
- It scanned standard paths (`get_workspace_storage_dirs()`) aggregating all candidate directories across the system (default `%APPDATA%/Antigravity/User/workspaceStorage`, global `.gemini`, and all custom instance data directories).
- However, when constructing the project conversation tree, it scanned all projects together into a single global `Vec<AgmProjectTreeNode>` and cached it globally (`prompt_tree_cache`) without scoping queries to the requested `instance_id`.
- Every project discovered from any instance's `workspaceStorage` was grouped together and returned in the global list, with loose or missing instance tagging on conversations.

### Flaw 2: Un-Gated 10-Minute Recency Heuristic Overriding Process Liveness (Backend)

In `src-tauri/src/modules/repo_db.rs`, conversation status determination relied heavily on timestamp heuristics:
```rust
// FLAW: Blind recency heuristic
let is_recency_active = age_seconds < 600; // 10 minutes
if is_recency_active || status == "RUNNING" {
    conv.is_running = true;
}
```
- A conversation updated within 600 seconds was unconditionally marked `is_running = true`, completely ignoring whether any OS process (PID) was actually alive for that instance!
- If an instance was shut down 2 minutes ago, or if an external editor touched files, the system declared the task `[RUNNING]`.
- Crucially, the discovery logic never checked `is_instance_running(&inst.id, &inst.data_dir, inst.pid)` before assigning `conv.is_running = true` and `node.is_running = true`.

### Flaw 3: Loose Frontend Name Matching in Card Component (Frontend)

In `src/pages/Instances.tsx`:
```tsx
// FLAW: Loose instance_name string matching
const hasActiveTask = runningTreeNodes.some((node) => {
    const isInstanceMatch =
        node.instance_id === inst.config.id ||
        node.instance_name === inst.config.name || // <-- DANGEROUS LOOSE MATCH
        (inst.config.is_default && (node.instance_id === 'default' || node.instance_id === '__default__' || !node.instance_id));
    ...
});

const instanceProjects = projectTreeNodes.filter((node) => {
    return (
        node.instance_id === inst.config.id ||
        node.instance_name === inst.config.name || // <-- DANGEROUS LOOSE MATCH
        ...
    );
});
```
- Because instance names were often defaulted, duplicated, or empty strings (`"" === ""`), loose string equality `node.instance_name === inst.config.name` matched projects across unintended instance cards.
- Furthermore, `isProjRunning` was evaluated purely from conversation states without gating on the host instance's running state:
  ```tsx
  // FLAW: Ungated project running state
  const isProjRunning = proj.is_running || proj.conversations?.some((c) => c.is_running || c.status === 'RUNNING');
  ```
  If `inst.is_running` was `false`, `isProjRunning` still evaluated to `true`, rendering pulsing green/cyan badges on dead instances!

### Flaw 4: Empty Array Fallback Trap in Prompt Tree Modal (Frontend)

In `src/components/instances/PromptTreeViewModal.tsx` (in both `handleRefreshSingleProject` and `loadTree`):
```tsx
// FLAW: Empty array fallback leak
const relevant = instanceId
    ? data.filter((p) =>
          !p.instance_id ||
          p.instance_id === instanceId ||
          (p.instance_id === 'default' && instanceId === 'default')
      )
    : data;
const finalData = relevant.length > 0 ? relevant : data; // <-- CRITICAL LEAK!
setTreeData(finalData);
```
- When a user opened `PromptTreeViewModal` for a newly created or scoped instance (such as 8159) that had no projects yet (`relevant.length === 0`), the ternary operator fell back to `data` (the entire global list from all instances)!
- In addition, `!p.instance_id` allowed unassigned/legacy projects to leak into secondary/cloned instances rather than confining unassigned nodes strictly to the default instance.
- As a consequence, opening the modal for an isolated instance instantly leaked all projects from the default profile.

---

## Part 3: Comprehensive Architecture Remedies

To permanently eliminate cross-instance bleeding and false liveness, four coordinated remedies were engineered across the full stack.

### 3.1 Parametric & Partitioned Backend Discovery (`get_project_conversation_tree`)

1. **IPC Signature Enhancement**:
   Updated `get_project_conversation_tree` to accept `instance_id: Option<String>`.
2. **Workspace Storage Scoping**:
   When `instance_id` is specified:
   - AGM resolves the instance's explicit `data_dir`.
   - Workspace directories are restricted strictly to that instance's `data_dir/User/workspaceStorage`.
   - Discovered projects and conversations are tagged with that instance's exact ID.
3. **Partitioned In-Memory and SQLite Caching**:
   Cache keys in `prompt_tree_cache` are partitioned by instance: `tree:{instance_id}:{max_words}:{only_running}`. Changes in one instance never invalidate or contaminate cached trees of another instance.

### 3.2 OS Process (PID) Verification Gate for Prompt Liveness

1. **Liveness Gating Rule**:
   A conversation or project node is evaluated as `is_running: true` **if and only if**:
   - The owning instance has an active OS process running (`find_pids_for_data_dir` or verified live PID via `sysinfo`), AND
   - The conversation status is explicitly `"RUNNING"` or has an active uncompleted prompt within the active execution window.
2. **Timestamp Recency Downgraded**:
   File modification recency (`age < 600`) is demoted from an unconditional "RUNNING" flag to a secondary heuristic that is valid **only while the parent instance PID is alive**. If the instance process is dead, `is_running` is strictly `false`.

### 3.3 Strict Frontend Instance ID Scoping & Process Gating (`Instances.tsx`)

1. **Elimination of Loose Name Matching**:
   Removed `node.instance_name === inst.config.name` across all card filters.
2. **Strict Per-Instance Filtering**:
   - Default Instance Card: Matches `node.instance_id === inst.config.id || node.instance_id === 'default' || node.instance_id === '__default__' || !node.instance_id`.
   - Cloned / Secondary Instance Cards (e.g. 8159): Matches STRICTLY `node.instance_id === inst.config.id`.
3. **Host Process Gating for Badges**:
   ```tsx
   const isProjRunning = Boolean(inst.is_running) && Boolean(
       proj.is_running || proj.conversations?.some((c) => c.is_running || c.status === 'RUNNING')
   );
   const hasActiveTask = Boolean(inst.is_running) && runningTreeNodes.some((node) => {
       const isInstanceMatch = inst.config.is_default
           ? (node.instance_id === 'default' || node.instance_id === '__default__' || !node.instance_id || node.instance_id === inst.config.id)
           : node.instance_id === inst.config.id;
       const isNodeRunning = Boolean(node.is_running) || Boolean(node.conversations?.some((c) => c.is_running || c.status === 'RUNNING'));
       return isInstanceMatch && isNodeRunning;
   });
   ```
   If an instance is stopped in the OS (`!inst.is_running`), its card will **never** display `[RUNNING]` badges or prompt activity indicators.

### 3.4 Elimination of the Empty Fallback Leak (`PromptTreeViewModal.tsx`)

1. **Elimination of `relevant.length > 0 ? relevant : data`**:
   The hazardous fallback was replaced with:
   ```tsx
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
   const finalData = relevant;
   setTreeData(finalData);
   ```
   If an instance has zero projects, `treeData` is legitimately `[]`, displaying the clean empty state ("No projects or conversations found.") without leaking global data.
2. **IPC Parameter Forwarding**:
   Both `handleRefreshSingleProject` and `loadTree` now forward `instanceId: instanceId || undefined` directly to `get_project_conversation_tree`.

---

## Part 4: Future AI Directives & Non-Negotiable Rules

All future AI agents, maintainers, and contributors modifying Antigravity Manager MUST adhere strictly to the following non-negotiable architectural mandates:

### Rule 1: NEVER Use Loose Name Matching for Instance Scoping
- **Forbidden**: Matching instances via `name`, `label`, or display title strings (e.g. `node.instance_name === inst.name`). Names are user-editable, non-unique, and prone to collisions.
- **Mandatory**: Use canonical IDs (`inst.config.id`, `"default"`, or explicit data directory paths).

### Rule 2: NEVER Fall Back to Global Collections on Empty Filter Results
- **Forbidden**: `const final = scoped.length > 0 ? scoped : globalAll;`
- **Rationale**: An empty list is a completely valid state representing an isolated or freshly initialized instance. Falling back to the global collection destroys data boundaries and causes silent security/privacy state leaks. Always render the authentic empty state.

### Rule 3: NEVER Declare an Entity "RUNNING" Without Active OS Process Confirmation
- **Forbidden**: Tagging an instance, project, conversation, or task as `[RUNNING]` solely because a file timestamp is recent (`age < 600`) or because a database record has status `"RUNNING"`.
- **Mandatory**: Process liveness is the source of truth. If the parent OS process (PID) is dead, the task is dead. Gate all liveness badges with affirmative process verification (`is_running` / `is_instance_running`).

### Rule 4: Unassigned Nodes Belong Exclusively to the Default Profile
- **Forbidden**: Allowing `!node.instance_id` to match secondary or cloned instances.
- **Mandatory**: Legacy, unassigned, or null `instance_id` nodes must map strictly to the Default profile (`inst.config.is_default`). Cloned instances (e.g., 8159) must match ONLY their explicit, registered `instance_id`.
