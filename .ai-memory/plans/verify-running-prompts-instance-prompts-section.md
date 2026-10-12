# [Plan] Verify Running Prompts in Instance Prompts Section

## User Request (Verbatim)
verify running prompts can showup in the promtps section of instance, plesase verify @[.agents/skills/execute-parent-task-with-n-steps-v6]

## Executive Summary
This plan addresses the root causes of why running prompts with active loading spinners in Google Antigravity IDE failed to appear in the Prompts / Prompt Tree View section of AGM (often showing "No Projects Found in this Profile" or "Running: 0").

Through empirical transcript analysis and reverse-engineering of Antigravity's internal storage:
1. In `src-tauri/src/modules/repo_db/transcript.rs`, individual tool execution steps (`status == "DONE"`) were treated as terminal turn completions, incorrectly flipping `is_terminal_done = true` and `effective_recent_active = false` mid-turn.
2. In `tree_collect.rs` and `liveness.rs`, substring matching on `status.contains("IDLE")` was overriding `not_fully_idle == 1`. In Google Antigravity, while parent callers await subagent tasks or tool calls, the parent status is `CASCADE_RUN_STATUS_IDLE` but `not_fully_idle = 1`. AGM was incorrectly silencing active turns.
3. Timestamp thresholds were clamped to a rigid 60-second cutoff instead of the 10-minute (600s) adaptive window needed for reasoning/thinking models (Claude 3.7 Thinking, Gemini 2.5 Pro).
4. `tree_node_liveness.rs` was falling back to `IDLE_EMPTY_WORKSPACE` if conversation summaries lagged behind real-time disk transcripts.

## Decomposition & Subtasks

| Subtask ID | Title | Owner | Target Files | Status |
| :--- | :--- | :--- | :--- | :--- |
| `Task-01` | Running Prompts Candidate Directory & Storage Resolution | Worker 01 | `src-tauri/src/modules/repo_db/gemini_dirs.rs`, `src-tauri/src/modules/repo_db/tree_collect.rs` | PENDING |
| `Task-02` | Transcript Turn Liveness & Not Fully Idle Status Evaluation | Worker 02 | `src-tauri/src/modules/repo_db/transcript.rs`, `src-tauri/src/modules/repo_db/liveness.rs`, `src-tauri/src/modules/repo_db/tree_node_liveness.rs` | PENDING |
| `Task-03` | Prompt Tree View UI Running State Rendering & Filter Hardening | Worker 01 | `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, `src/pages/instances/InstanceCardProjects.tsx` | PENDING |
| `Task-04` | Running Prompts Verification Test Harness & Telemetry Proof | Worker 02 | `scripts/test-running-prompts-e2e.ps1` | PENDING |

## Work Breakdown & Wave Allocation

- **Wave 1 (Backend Engine Fixes)**:
  - **Worker 01**: `Task-01` — Expand candidate directories in `gemini_dirs.rs`, ensure workspace storage resolution does not drop valid project paths, and refine `tree_collect.rs` candidate directory scanning.
  - **Worker 02**: `Task-02` — Fix `transcript.rs` reverse scanner to differentiate step completion from turn completion, honor `not_fully_idle > 0` over idle status strings in `tree_collect.rs`, `liveness.rs`, and `tree_node_liveness.rs`, and corroborate file `mtime`.

- **Wave 2 (UI Rendering & Verification)**:
  - **Worker 01**: `Task-03` — Polish `PromptTreeViewModal.tsx` and `InstanceCardProjects.tsx` to ensure active running prompt indicators (pulsing green badges, live turn counts, tool call summaries) render accurately and filters default sensibly.
  - **Worker 02**: `Task-04` — Author and execute `scripts/test-running-prompts-e2e.ps1` validating running prompts detection across CLI (`agm tree`, `agm prompts tree`) and IPC.

## Verification & Acceptance Criteria
1. Active Antigravity projects with in-flight prompts appear in `agm prompts tree` / `agm tree` with `is_running = true`.
2. Intermediate tool completion steps do not cause `is_terminal_done = true`.
3. `not_fully_idle = 1` remains authoritative over `CASCADE_RUN_STATUS_IDLE`.
4. UI displays green pulsing `RUNNING` badge and active turn count.
5. `scripts/test-running-prompts-e2e.ps1` executes successfully with exit 0.
