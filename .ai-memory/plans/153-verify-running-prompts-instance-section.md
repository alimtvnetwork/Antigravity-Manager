# Master Plan: Verification of Running Prompts in Instance Section

- **Slug**: `153-verify-running-prompts-instance-section`
- **Date**: 2026-10-11
- **Lead Architect**: Md. Alim Ul Karim (@aukgit)
- **Status**: COMPLETED

---

## 1. Executive Summary & Verification Context

User requested empirical verification that active running prompts accurately appear in the prompts section of an Antigravity instance:
`<USER_REQUEST> verify running prompts can showup in the promtps section of instance, plesase verify @[.agents/skills/execute-parent-task-with-n-steps-v6] </USER_REQUEST>`

This verification covers the full-stack architecture:
1. **Backend Liveness & Dead-Process Gating**:
   - `tree_node_liveness.rs` and `tree_node_conv.rs` check if the instance process is alive (`is_inst_alive`).
   - If the instance process is dead on the host OS, running state is strictly forced to `false` (`INSTANCE_PROCESS_DEAD`).
   - If the instance is alive, active conversation turns from `conversation_summaries.db` (`RUNNING` / `not_fully_idle = 1`) or active SQLite prompts (`active_prompts`) within the sliding TTL window (45s) are flagged `is_running = true`.
   - Deep transcript inspection reverse-scans up to 25 lines in `transcript_full.jsonl` bypassing trailing telemetry noise.
2. **Per-Instance Ownership Isolation**:
   - `isNodeOwnedByInstance` maps projects and conversation nodes strictly to the owning instance configuration (`instConfig.id` or default aliases `default`/`__default__`), preventing cross-instance state bleed.
3. **Instance Card Prompts Badging & Projects Section**:
   - `useInstanceCardData` calculates `hasActiveTask`. When `true`, a pulsing emerald `[● Prompt]` button is displayed in the card header.
   - `InstanceCardProjects` renders the "Active / Recent Projects" section, sorts running projects to the top, and renders the green `[● RUNNING]` pill badge.
   - Slot 1 of the action bar capsule renders the `[Layers]` prompt tree trigger with a cyan pulse dot.
4. **Prompt Tree View Modal (`PromptTreeViewModal`)**:
   - Opening the modal passes `instanceId` and optional `initialSelectedProjectId`.
   - `usePromptTreeData` automatically prioritizes non-archived projects, detects running conversations, and auto-selects the active running conversation.
   - The `Running` segmented filter button allows instant filtering to active running prompts.
5. **Headless CLI Parity**:
   - `agm observe default`, `agm prompts tree`, and `agm running-prompts ls` reflect active prompt counts, liveness status, and queue states in exact parity with the GUI.

---

## 2. Work Breakdown & Subtask Allocation

| Subtask ID | Title | Owner | Target Files | Status |
| :--- | :--- | :---: | :--- | :---: |
| **Subtask-01** | Backend Liveness & Instance Sandboxing Verification | Worker 01 | `src-tauri/src/modules/repo_db/tree_node_liveness.rs`, `src-tauri/src/modules/repo_db/tree_node_conv.rs` | COMPLETED |
| **Subtask-02** | Frontend Instance Card & Active Projects Section Verification | Worker 02 | `src/pages/instances/InstanceCard.tsx`, `src/pages/instances/InstanceCardProjects.tsx`, `src/pages/instances/instanceCardData.ts` | COMPLETED |
| **Subtask-03** | Prompt Tree Deep-Linking & Auto-Selection Engine Verification | Worker 02 | `src/components/instances/PromptTreeView/usePromptTreeData.ts`, `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | COMPLETED |
| **Subtask-04** | CLI Parity & Automated Verification Test Suite | Worker 01 | `src/pages/instances/__tests__/instanceRunningPrompts.test.ts`, `src-tauri/src/bin/agm/` | COMPLETED |

---

## 3. Strict Rules & Quality Invariants

- Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- Strictly relative Git paths with lowercase hygiene.
- Mandatory Multi-Agent Partitioning: `A = 2`, `H = 2` via `invoke_subagent`.
- Zero disruption to host IDE processes during verification.
- Attribution strictly to `@aukgit` (`(Thanks to @aukgit)`).
