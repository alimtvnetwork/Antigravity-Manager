---
plan: 113-per-instance-running-prompts-and-projects-isolation
subtask: "003"
title: Frontend Instance Card Scoping, Strict ID Isolation & Accurate Liveness Badging
domain: frontend-react
depends_on:
  - "001"
citations:
  app_spec: ../../../../02-spec/21-app/113-per-instance-running-prompts-and-projects-isolation.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../../02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md
target_files:
  - src/pages/Instances.tsx
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# Subtask 003 — Frontend Instance Card Scoping, Strict ID Isolation & Accurate Liveness Badging

## 1. Objectives & Problem Analysis

In user testing and visual telemetry (Plan 113):
- **Cross-Instance Bleed**: Instance Card #1 (Default) and Instance Card #2 (8159) mirror the exact same list of projects (`Antigravity-Manager`, `spec-builder`, `coding-guidelines`) with identical conversation turns.
- **False `[RUNNING]` Status**: Card #1 shows `spec-builder` and `coding-guidelines` as running even though they only run in Card #2 (or not at all). Card #2 shows `Antigravity-Manager` as running even though it only runs in Default profile.
- **Frontend Root Causes**:
  1. In `src/pages/Instances.tsx` (lines 926 and 1251), instance matching uses loose string comparison:
     ```tsx
     node.instance_name === inst.config.name // <-- DANGEROUS LOOSE MATCH
     ```
     This matches multiple instances if names or legacy defaults overlap.
  2. In `src/pages/Instances.tsx` (line 1284), the project running badge is calculated purely from conversation state without verifying whether the host instance process is actually running:
     ```tsx
     const isProjRunning = proj.is_running || proj.conversations?.some((c) => c.is_running || c.status === 'RUNNING');
     ```
     If the instance process is stopped, or if another instance had recent conversations for that repo path, the card still badges `[RUNNING]`.
  3. In `src/components/instances/PromptTreeViewModal.tsx` (lines 601 and 806), a hazardous fallback exists:
     ```tsx
     const relevant = instanceId
         ? data.filter(
               (p) =>
                   !p.instance_id || // Leaks unassigned nodes to any instance
                   p.instance_id === instanceId ||
                   (p.instance_id === 'default' && instanceId === 'default')
           )
         : data;
     const finalData = relevant.length > 0 ? relevant : data; // FALLBACK LEAK!
     ```
     When an instance has no projects yet (`relevant.length === 0`), it falls back to `data` (the entire global list from all instances), causing complete state leakage.

---

## 2. Target Files & Symbols

| File Path | Target Symbols / Locations | Action |
| :--- | :--- | :--- |
| `src/pages/Instances.tsx` | `fetchRunningTasks` (~L210), `hasActiveTask` (~L923), `instanceProjects` (~L1248), `isProjRunning` (~L1284) | Refactor project scoping to strict `instance_id`, eliminate loose name matching, gate `isProjRunning` by `inst.is_running`. |
| `src/components/instances/PromptTreeViewModal.tsx` | `handleRefreshSingleProject` (~L584-602), `loadTreeData` (~L790-808) | Pass `instance_id` to `get_project_conversation_tree`, remove empty fallback `finalData = relevant.length > 0 ? relevant : data`, scope `!p.instance_id` exclusively to default instance. |

---

## 3. Detailed Implementation Steps & Code Blueprint

### 3.1 `src/pages/Instances.tsx` — Strict `instance_id` Scoping in Card Filtering

1. **Remove Loose Name Matching in `hasActiveTask`**:
   In `src/pages/Instances.tsx`, update `hasActiveTask` computation around lines 923-930:
   ```tsx
   const hasActiveTask = inst.is_running && runningTreeNodes.some((node) => {
       const isInstanceMatch =
           node.instance_id === inst.config.id ||
           (inst.config.is_default && (node.instance_id === 'default' || node.instance_id === '__default__' || !node.instance_id));
       const isNodeRunning = Boolean(node.is_running) || Boolean(node.conversations?.some((c) => c.is_running || c.status === 'RUNNING'));
       return isInstanceMatch && isNodeRunning;
   });
   ```
   **Key Rule**:
   - `node.instance_name === inst.config.name` is completely deleted.
   - If `!inst.is_running`, `hasActiveTask` is immediately `false`.

2. **Strict Per-Instance Project List Filtering**:
   In `src/pages/Instances.tsx`, update `instanceProjects` around line 1248:
   ```tsx
   const instanceProjects = projectTreeNodes.filter((node) => {
       if (inst.config.is_default) {
           return (
               node.instance_id === inst.config.id ||
               node.instance_id === 'default' ||
               node.instance_id === '__default__' ||
               !node.instance_id
           );
       }
       return node.instance_id === inst.config.id;
   });
   ```
   - Only projects that are explicitly attributed to `inst.config.id` appear under that card.
   - Unassigned projects (`!node.instance_id`) or default aliases (`default`, `__default__`) appear strictly on the default card, never on custom cloned/secondary instances like 8159.

3. **Gating `[RUNNING]` Badging by Process Liveness**:
   In `src/pages/Instances.tsx`, update `isProjRunning` around line 1284:
   ```tsx
   // A project can only be actively running if:
   // 1. The hosting instance process is running in the OS (inst.is_running)
   // 2. The project node itself or one of its active conversations has running status
   const isProjRunning = Boolean(
       inst.is_running &&
       (proj.is_running || proj.conversations?.some((c) => c.is_running || c.status === 'RUNNING'))
   );
   ```
   - If Card #1 (Default) is running `Antigravity-Manager`, only `Antigravity-Manager` shows `[RUNNING]`.
   - If Card #2 (8159) is running `coding-guidelines`, only `coding-guidelines` shows `[RUNNING]`.
   - If an instance is stopped, none of its projects show `[RUNNING]`.

---

### 3.2 `src/components/instances/PromptTreeViewModal.tsx` — Eliminate Fallback Leak

1. **Pass `instance_id` to `get_project_conversation_tree` IPC**:
   Update line 588 and line 793 to supply `instance_id`:
   ```tsx
   const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
       instanceId: instanceId || undefined,
       maxWords: 300,
       onlyRunning: false,
       force: isForce,
   });
   ```

2. **Eliminate Unsafe Fallback**:
   In `handleRefreshSingleProject` and `loadTreeData`:
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

   // DO NOT fall back to global `data` when `relevant` is empty!
   // An empty list correctly indicates that this specific instance has no projects.
   const finalData = relevant;
   setTreeData(finalData);
   ```

---

## 4. Edge Cases & Guardrails

1. **Default Instance Aliasing**:
   Default instance IDs can appear as `"default"`, `"__default__"`, `null`, or undefined in legacy SQLite entries.
   - Guardrail: When `inst.config.is_default` is true, accept all legacy default representations.
   - When `!inst.config.is_default`, accept strictly `node.instance_id === inst.config.id`.
2. **Empty Project List on Fresh Instance**:
   When an instance is newly cloned, its `workspaceStorage` contains no projects yet.
   - Guardrail: Render "No recent projects" placeholder cleanly. Never bleed global projects from other instances.
3. **Double Click to Inspect**:
   Double-clicking an instance project continues to open `PromptTreeViewModal` with the correct `projectId` and `instanceId` scoped context.

---

## 5. Verification Commands & Acceptance Criteria

### Verification Commands
```bash
# Verify TypeScript build and bundling passes with zero errors
npm run build
```

### Acceptance Criteria
- [ ] Card #1 (Default) renders only its own workspace projects (`Antigravity-Manager`), displaying `[RUNNING]` only when active.
- [ ] Card #2 (8159) renders only its own workspace projects (`coding-guidelines`), displaying `[RUNNING]` only when active.
- [ ] No cross-instance mirroring occurs between cards.
- [ ] Stopping an instance immediately removes the `[RUNNING]` badge from all projects in that card.
- [ ] `PromptTreeViewModal` scoped to an instance shows only projects belonging to that instance, even if the list is empty.

---

## 6. Done When Checklist
- [ ] Loose string match `node.instance_name === inst.config.name` removed from `src/pages/Instances.tsx`.
- [ ] `isProjRunning` gated by `inst.is_running`.
- [ ] Dangerous fallback `finalData = relevant.length > 0 ? relevant : data` removed from `src/components/instances/PromptTreeViewModal.tsx`.
- [ ] `npm run build` succeeds cleanly.
