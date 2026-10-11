# Completed Plan: 07-verify-running-prompts-instance-section

## User Request (Verbatim)
`verify running prompts can showup in the promtps section of instance, plesase verify @[.agents/skills/execute-parent-task-with-n-steps-v6]`

## Canonical Specifications
- Architecture Spec: `02-spec/21-app/07-verify-running-prompts-instance-section/01-architecture-spec.md`
- Component Spec: `02-spec/21-app/07-verify-running-prompts-instance-section/02-component-spec.md`

## Verification Summary
Both backend and frontend pipelines were comprehensively verified for tracking, sandboxing, and rendering running prompts in the instance section:

1. **Backend Liveness & Instance Sandboxing (`src-tauri/src/modules/repo_db/`)**:
   - `get_project_conversation_tree` / `compute_project_conversation_tree` partitions active prompts by composite tuple `(instance_id, normalized_repo_path)`, preventing cross-instance prompt bleeding between default and cloned profiles.
   - Operating system process gating (`is_inst_alive`) strictly forces prompts and projects to `IDLE` if the instance process is terminated.
   - Empirical confirmation (`has_confirmed_running = has_active_worker || has_non_idle_summary`) with adaptive TTL (<=45s, or <=300s with active worker PID) guarantees that ghost/stale tasks never stay `RUNNING`.
   - FIFO query guarantees (`ORDER BY created_at ASC, id ASC`) in `project_queries.rs`.

2. **Frontend Instance Section & Prompt Tree Rendering (`src/pages/instances/` & `src/components/instances/PromptTreeView/`)**:
   - `InstanceCard.tsx`: Displays pulsing green `Prompt` pill when `hasActiveTask` is true.
   - `InstanceCardProjects.tsx`: Hoists running projects to the top with a green `RUNNING` badge and pulsing dot alongside turn counts; clicking deep-links to `PromptTreeViewModal`.
   - `PromptTreeViewModal.tsx` & `TreePanel.tsx`: Interactive `Running` filter pill; auto-selects the newest active running conversation on open via `performAutoSelection` in `usePromptTreeData.ts`.
   - `ConversationNode.tsx`: Renders pulsing green `RUNNING` badge with 4-tier origin badges.

3. **Automated Verification Suite**:
   - Ran `node scripts/run-frontend-tests.mjs instanceRunningPrompts` -> 20/20 test assertions passed (Test Group 1: Ownership; Test Group 2: Sorting; Test Group 3: Active Task; Test Group 4: Ghost filtering).

## Work Items
| Task-ID | Title | Status | Evidence |
|---|---|---|---|
| Task-01 | Verify Backend Running Prompts Engine | DONE | Verified composite partitioning, process gating, and empirical liveness in `tree_nodes.rs` and `tree_node_conv.rs` |
| Task-02 | Verify Frontend Instance Prompts Section | DONE | Verified `InstanceCardProjects.tsx`, `TreePanel.tsx`, `ConversationNode.tsx`, and passed unit tests (20/20) |
