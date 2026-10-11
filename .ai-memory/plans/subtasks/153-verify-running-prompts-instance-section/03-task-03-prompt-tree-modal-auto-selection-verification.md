---
plan: 153-verify-running-prompts-instance-section
subtask: "03"
title: Prompt Tree Modal Auto-Selection and Deep-Linking Verification
domain: frontend-react
status: verified
evidence:
  - "usePromptTreeData verified to deep-link to initialSelectedProjectId when opened from instance card"
  - "Auto-selection algorithm verified to prioritize active running conversations across non-archived projects"
  - "Segmented Running filter verified to isolate in-flight tasks instantly in PromptTreeViewModal"
  - "Conversation guards verified to prevent ghost conversations from being auto-selected"
verified_invariants:
  - "Invariant I9 (Direct Deep-Link Precedence): Passing initialSelectedProjectId expands and focuses target project immediately"
  - "Invariant I10 (Active Conversation Auto-Selection): The newest valid running conversation is automatically selected upon opening"
  - "Invariant I11 (Non-Archived Project Prioritization): Pinned and running projects precede idle unpinned projects in tree display"
  - "Invariant I12 (Segmented Running Filter): Switching to Running filter displays only projects with active running turns"
citations:
  architecture_spec: 02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md
  root_cause_analysis: 02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md
target_files:
  - src/components/instances/prompttreeview/prompttreeviewmodal.tsx
  - src/components/instances/prompttreeview/useprompttreedata.ts
  - src/components/instances/prompttreeview/conversationguards.ts
  - src/components/instances/prompttreeview/types.ts
  - src/pages/instances/instancemodals.tsx
---

# Subtask 03: Prompt Tree Modal Auto-Selection and Deep-Linking Verification

## 1. Context & Objectives

When a user clicks on the pulsing emerald `[● Prompt]` button in the instance card header or selects an active project from the "Active / Recent Projects" list, the `PromptTreeViewModal` opens. Rather than presenting an arbitrary project or requiring the user to hunt through dozens of workspaces, the modal must immediately highlight the exact running prompt that triggered the user's attention.

Key verification objectives:
1. Verify `initialSelectedProjectId` deep-linking when triggered from instance cards.
2. Verify prioritized project sorting (pinned projects, running projects, recent activity timestamps, alphabetical).
3. Verify automatic selection of the newest active running conversation (`Boolean(c.is_running) && isValidConversation(c)`).
4. Verify the segmented `Running` filter button behavior in `PromptTreeViewModal`.
5. Verify stale group expansion and ghost conversation exclusion.

---

## 2. Verified Architectural Invariants

### Invariant I9: Direct Deep-Link Precedence
- **Rule**: If `initialSelectedProjectId` is provided (e.g. user clicked a specific project card item), the modal focuses that project immediately and bypasses global candidate searches.
- **Implementation**:
  - In `src/components/instances/prompttreeview/useprompttreedata.ts` (lines 96-128):
    ```typescript
    if (initialSelectedProjectId) {
        const target = data.find((p) => p.project_id === initialSelectedProjectId);
        if (target) {
            setExpandedProjects({ [target.project_id]: true });
            setSelectedProject(target);
            if (target.conversations && target.conversations.length > 0) {
                const runningConv = target.conversations.find(
                    (c) => Boolean(c.is_running) && isValidConversation(c)
                );
                if (runningConv) {
                    selectConversation(runningConv, target);
                    return;
                }
                const candidateConvs = target.conversations.filter(isValidConversation);
                if (candidateConvs.length > 0) {
                    const winningConv = sortByLastModifiedDesc(candidateConvs)[0];
                    selectConversation(winningConv, target);
                }
            }
            return;
        }
    }
    ```

### Invariant I10: Active Conversation Auto-Selection
- **Rule**: When no explicit project is targeted, the modal automatically searches non-archived projects for the most recent active running conversation and expands its parent project.
- **Implementation**:
  - In `src/components/instances/prompttreeview/useprompttreedata.ts` (lines 150-174):
    ```typescript
    const runningCandidates: RunningCand[] = [];
    for (const proj of prioritized) {
        for (const conv of proj.conversations) {
            if (Boolean(conv.is_running) && isValidConversation(conv)) {
                runningCandidates.push({
                    conv,
                    proj,
                    timestamp: new Date(conv.last_modified).getTime() || 0,
                });
            }
        }
    }
    if (runningCandidates.length > 0) {
        runningCandidates.sort((a, b) => b.timestamp - a.timestamp);
        const winning = runningCandidates[0];
        setExpandedProjects({ [winning.proj.project_id]: true });
        selectConversation(winning.conv, winning.proj);
        return;
    }
    ```

### Invariant I11: Non-Archived Prioritized Project Ranking
- **Rule**: Archived projects are deprioritized. Among non-archived projects, ranking enforces:
  1. Pinned projects first (`currentPinnedIds.includes(p.project_id)`).
  2. Running projects / conversations second (`Boolean(p.is_running)`).
  3. Newest conversation timestamp third (`Math.max(...convs.last_modified)`).
  4. Alphabetical by `repo_name` as fallback.
- **Implementation**:
  - In `src/components/instances/prompttreeview/useprompttreedata.ts` (lines 130-149).

### Invariant I12: Segmented Running Filter Isolation
- **Rule**: Clicking the `Running` pill filter in the modal header immediately filters the tree list, showing only projects that contain active turns or in-flight prompts.
- **Implementation**:
  - In `src/components/instances/prompttreeview/prompttreeviewmodal.tsx` (lines 750-780): The segmented pill control toggles `filterMode: 'all' | 'running' | 'archived'`. When set to `'running'`, projects without running conversations are filtered out from the sidebar.

---

## 3. Technical Evidence & Modal Flow Verification

1. **Modal Mounting via `InstanceModals.tsx`**:
   - In `src/pages/instances/instancemodals.tsx` (lines 148-160):
     ```tsx
     <PromptTreeViewModal
         isOpen={Boolean(promptTreeInstance)}
         onClose={() => setPromptTreeInstance(null)}
         instanceId={promptTreeInstance?.id || 'default'}
         instanceName={promptTreeInstance?.name || 'Default'}
         initialSelectedProjectId={promptTreeInstance?.projectId}
         instanceSeqNum={promptTreeInstance?.seqNum}
         executablePath={promptTreeInstance?.executablePath}
     />
     ```
   - Clicking from `InstanceCardProjects` passes `projectId: proj.project_id`, ensuring instant targeted focus.
   - Clicking from the header `[● Prompt]` button passes `projectId: undefined`, triggering the global auto-selection algorithm which locates the active conversation automatically.

2. **Conversation Guard Validation**:
   - `isValidConversation` checks that the conversation is not a ghost conversation (`isGhostConversation`). Ghost conversations with no user prompt are never selected as active candidates.

---

## 4. Target Files & Relative References

- `src/components/instances/prompttreeview/prompttreeviewmodal.tsx` — Main modal component containing sidebar, inspect view, and segmented filters.
- `src/components/instances/prompttreeview/useprompttreedata.ts` — Data hook implementing deep-linking, prioritization, and auto-selection algorithms.
- `src/components/instances/prompttreeview/conversationguards.ts` — Conversation filters (`isGhostConversation`, `isStaleOrEmptyConversation`).
- `src/components/instances/prompttreeview/types.ts` — TypeScript interfaces for project tree nodes and conversation nodes.
- `src/pages/instances/instancemodals.tsx` — Modal mounting container on the instance management page.
- `02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md` — Specification reference.
- `02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md` — Root cause analysis and matrix.
