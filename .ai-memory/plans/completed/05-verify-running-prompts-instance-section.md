# Plan: Verify Running Prompts in Instance Section

## User Request (Verbatim)
verify running prompts can showup in the promtps section of instance, plesase verify @[.agents/skills/execute-parent-task-with-n-steps-v6]

## Architecture & Boundary
- Subsystem: Instance Card (`InstanceCard.tsx`, `InstanceCardProjects.tsx`), Prompt Tree View (`PromptTreeViewModal.tsx`, `TreePanel.tsx`, `ConversationNode.tsx`, `usePromptTreeData.ts`), and Backend Liveness Evaluator (`tree_node_liveness.rs`, `tree_node_conv.rs`).
- Goal: Systematically verify that running prompts are correctly identified by the backend, mapped to the owning instance without cross-instance bleed, displayed as glowing `[● Prompt]` and `[RUNNING]` badges in the instance card's "Active / Recent Projects" prompts section, and fully accessible in the Prompt Tree View modal with auto-selection and filtering.

## Task Breakdown & Verification Results
- ✅ **Task-01: Verify Backend Prompt Liveness & Isolation Architecture** — `[Completed]`
  - Audited `src-tauri/src/modules/repo_db/tree_node_liveness.rs` and `tree_node_conv.rs`.
  - Process verification (`is_inst_alive`) strictly gates prompt and project liveness. If the instance OS process is dead, `is_running` is forced to `false`.
  - When the process is running, genuine non-idle conversation turns from `conversation_summaries.db` (`RUNNING`, `not_fully_idle = 1`) or in-flight prompts in SQLite `active_prompts` within 45s TTL are correctly marked `is_running = true`.
  - Empty or untitled conversations with 0 words are filtered out to prevent ghost records.

- ✅ **Task-02: Verify Frontend Instance Card & Prompt Tree View Integration** — `[Completed]`
  - Audited `src/pages/instances/InstanceCard.tsx`: `hasActiveTask` computes whether the instance is running and has a genuine running conversation; renders the pulsing emerald `[● Prompt]` button in the header.
  - Audited `src/pages/instances/InstanceCardProjects.tsx`: "Active / Recent Projects" filters strictly by `isNodeOwnedByInstance`, prioritizes running projects to the top of the list, displays the `[● RUNNING]` green pill badge, and provides click-to-open deep linking.
  - Audited `src/components/instances/PromptTreeView/usePromptTreeData.ts`: `performAutoSelection` algorithm prioritizes running projects, detects active running conversations, and auto-selects the running conversation upon opening.
  - Audited `src/components/instances/PromptTreeView/TreePanel.tsx`: `Running` filter button allows immediate filtering to active running prompts.
  - Audited `src/components/instances/FleetMachinesTable.tsx` & `FleetMachineRow.tsx`: Column 5 "Running Prompts" shows `[⚡ N Running]`.
  - Audited `src/components/instances/InstanceTable.tsx` & `InstanceRowActions.tsx`: Table view "Prompts & History" menu action opens Prompt Tree.

- ✅ **Task-03: Implement & Run Standalone Automated Verification Test Suite** — `[Completed]`
  - Authored `src/pages/instances/__tests__/instanceRunningPrompts.test.ts`.
  - Ran via `node scripts/run-frontend-tests.mjs instanceRunningPrompts`: 28/28 assertions passed (exit 0).
