# Master Audit Ledger: Verification of Running Prompts in Instance Section

- **Slug**: `153-verify-running-prompts-instance-section`
- **Created**: 2026-10-11
- **Status**: COMPLETED
- **Lead Architect**: Md. Alim Ul Karim (@aukgit)

---

## User Request (Verbatim)

```text
<USER_REQUEST> verify running prompts can showup in the promtps section of instance, plesase verify @[.agents/skills/execute-parent-task-with-n-steps-v6] </USER_REQUEST>
```

---

## Discrete Deliverables & Verification Tracking

| Task ID | Deliverable Description | Status | Completion Timestamp | Target Files & Evidence |
| :--- | :--- | :---: | :---: | :--- |
| **Task-01** | **Backend Liveness & Instance Sandboxing Verification**: Verify composite partitioning by `(instance_id, normalized_repo_path)`, OS process gating (`is_inst_alive`), TTL-bounded running status (<=45s / <=300s with active worker PID), and FIFO query ordering (`ORDER BY created_at ASC, id ASC`). | COMPLETED | 2026-10-11T11:05:00Z | `src-tauri/src/modules/repo_db/tree_nodes.rs`, `src-tauri/src/modules/repo_db/tree_node_liveness.rs`, `src-tauri/src/modules/repo_db/tree_node_conv.rs`<br>• Verified dead-process cutoff strictly resets `is_running` to `false` when instance PID is dead.<br>• Verified zero cross-instance prompt bleeding between default and cloned profiles. |
| **Task-02** | **Frontend Instance Card & Projects Section UI Verification**: Verify `hasActiveTask` header indicator, running project hoisting and green badges in `InstanceCardProjects`, Slot 1 `[Layers]` action capsule button with pulse dot, and ownership isolation. | COMPLETED | 2026-10-11T11:10:00Z | `src/pages/instances/instancecard.tsx`, `src/pages/instances/instancecardprojects.tsx`, `src/pages/instances/instancecardactions.tsx`, `src/pages/instances/instancecarddata.ts`<br>• Verified `InstanceCard.tsx` (lines 123-133) pulsing emerald `[● Prompt]` button.<br>• Verified `InstanceCardProjects.tsx` (lines 14-23) sorting running projects to top, and lines 72-76 `[● RUNNING]` badge.<br>• Verified `InstanceCardActions.tsx` (lines 144-160) Slot 1 `[Layers]` button with cyan pulse dot.<br>• Verified `instanceCardData.ts` (lines 76-87) `hasActiveTask` and `isNodeOwnedByInstance` ownership isolation. |
| **Task-03** | **Prompt Tree Deep-Linking & Auto-Selection Engine Verification**: Verify `PromptTreeViewModal` auto-expansion and auto-selection of running conversations on open, deep-linking with `initialSelectedProjectId`, and `Running` segmented filter pill. | COMPLETED | 2026-10-11T11:14:00Z | `src/components/instances/prompttreeviewmodal.tsx`, `src/components/instances/prompttreeview/useprompttreedata.ts`, `src/components/instances/prompttreeview/treepanel.tsx`<br>• Verified `usePromptTreeData.ts` (lines 88-209) `performAutoSelection` algorithm finding newest running candidate, expanding owning project, and selecting conversation.<br>• Verified `Running` filter pill in `TreePanel.tsx` isolating active running prompts.<br>• Verified 4-tier prompt classification badges (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`). |
| **Task-04** | **Automated Test Verification Suite Execution**: Verify end-to-end prompt liveness, project sorting, ownership scoping, and ghost filtration in automated test harness. | COMPLETED | 2026-10-11T11:17:00Z | `src/pages/instances/__tests__/instancerunningprompts.test.ts`, `scripts/run-frontend-tests.mjs`<br>• Executed `node scripts/run-frontend-tests.mjs instanceRunningPrompts` with 20/20 assertions passing.<br>• Verified all 4 test groups: (1) Ownership Isolation, (2) Project Sorting, (3) Active Task Calculation, (4) Ghost Conversation Filtering. |

---

## Non-Negotiable Operational Invariants & Verification Gates

1. **Strict FIFO Queue Processing Invariant**:
   All prompt restore, dispatch, and queue processing queries across SQLite databases enforce strict FIFO ordering (`ORDER BY created_at ASC, id ASC`).

2. **Per-Instance Ownership Isolation**:
   Projects and conversation trees are strictly bounded to the owning instance (`instConfig.id` or default aliases `default`/`__default__`). Cloned profile prompts never bleed into default instances or sibling clones.

3. **Process Vitality & Dead-Process Gating**:
   A prompt is never displayed as `RUNNING` if the owning instance process is terminated or absent from the operating system process table (`is_inst_alive == false`).

4. **Ghost & Empty Turn Filtration**:
   Conversations with 0 words and empty previews, or `isGhostConversation` matches, are strictly excluded from triggering `hasActiveTask` or being selected as running candidates.

5. **Cross-Platform & Headless Parity**:
   All UI states and data structures maintain full parity between native desktop Tauri runtime and headless/web REST client environments.
