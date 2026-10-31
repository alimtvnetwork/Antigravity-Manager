# Specification: Instance-Scoped Prompt Scoping & Enqueued Prompts Pipeline

## 1. Executive Summary
This specification defines the architectural and implementation requirements for:
1. Eliminating cross-instance prompt bleeding (where an active prompt on instance 2 erroneously renders on instance 1).
2. Establishing a reliable enqueued prompts tracking and visualization pipeline.

## 2. Topic 1: Instance-Scoped Running Prompt Association

### 2.1 Root Cause & Invariant
- **Root Cause**:
  1. Stale `'dispatched'` rows in SQLite table `active_prompts` were never retired by `transition_stale_inflight_prompts` (which only checked `'running'` and `'in_flight'`).
  2. `build_active_prompt_conv_nodes` in `tree_node_conv.rs` matched `is_inst_alive && ap_status_lower.contains("dispatched")` without checking row age (TTL), causing old prompts to be marked active forever whenever instance 1's process was running.
  3. `instance_seq_num` on conversation nodes was hardcoded to `None`, breaking sequence-based instance matching.
- **Specification**:
  - `src-tauri/src/modules/repo_db/tree_phases.rs`: Expand `transition_stale_inflight_prompts` to transition `'dispatched'` rows older than 120 seconds to `'completed'`.
  - `src-tauri/src/modules/repo_db/tree_node_conv.rs`: Apply a 10-minute maximum freshness TTL for any `'dispatched'` or `'running'` prompt before setting `target_conv.is_running = true`.
  - Forward `proj.instance_seq_num` directly to conversation nodes.
  - In `src/pages/instances/instancePageUtils.ts`: Enforce strict `instance_id` matching before falling back to default alias mapping.

## 3. Topic 2: Enqueued Prompts Pipeline

### 3.1 Tracking & State Preservation
- In `src-tauri/src/modules/repo_db/transcript.rs`: Multi-turn extraction MUST NOT erase queued turns when an instance process pauses. Prompt turns with `step_index` beyond the last completed assistant response MUST retain status `QUEUED` and `is_queued = true`.
- In `src/components/instances/PromptTreeView/ConversationNode.tsx`:
  - When a conversation has enqueued turns (`enqueuedTurnsCount > 0`), initialize `isExpanded` to `true` by default so enqueued turns are immediately visible.
  - Always render the amber badge `⚡ {enqueuedTurnsCount} queued` alongside the conversation title.
- In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
  - Render an explicit queue status chip: `⚡ {enqueuedCount} Prompts Enqueued (FIFO)`.
