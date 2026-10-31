# 4-Part Root Cause Analysis: Running & Enqueued Prompt Detection in Multi-Instance Environments

## 1. Problem Statement & Symptoms

In multi-instance Antigravity environments (such as running primary `default` alongside secondary sandboxed profiles like `lara-pub-6067`):
1. **Enqueued Prompts Missing from UI & Tree**: When users enqueue prompts to an actively executing IDE instance via Antigravity Manager, the enqueued prompts failed to appear in the Prompt Tree, the conversation detail header, or the queued prompts counter.
2. **Missing 4 Enqueued Prompts**: During long execution sessions, 4 enqueued prompts dispatched to a secondary instance (`lara-pub-6067`) were invisible in the UI, and were subsequently either dropped or rendered as completed turns instead of active queue items.
3. **Running State Discrepancy & Premature Idle Override**: Live instances executing multi-step agent tasks were misclassified as `IDLE` or `INSTANCE_PROCESS_DEAD`, causing active running indicators and queued turns to be suppressed when the `only_running` filter was toggled in the UI.

---

## 2. Root Cause Analysis (5 Compounding Defect Mechanisms)

### Defect A: Missing `'enqueued_ide'` Status in Backend Queue Guards
- When `enqueue_prompt_for_instance` (`src-tauri/src/modules/repo_db/agy.rs:1138`) injects a prompt into a running IDE via CDP with `cancel_running_first: false`, it records the prompt in `active_prompts` with `status = "enqueued_ide"`.
- However, in `src-tauri/src/modules/repo_db/tree_node_conv.rs:141`:
  ```rust
  fn is_queued_or_pending_status(status: &str) -> bool {
      const QUEUED_STATUSES: &[&str] = &["queued", "queue", "pending"];
      status.equals_any_ignore_case(QUEUED_STATUSES)
  }
  ```
  `"enqueued_ide"` and `"dispatched"` were completely omitted.
- Consequently:
  - In `tree_node_conv.rs:614`, enqueued prompts were dropped by ghost conversation pruning.
  - In `tree_node_conv.rs:697`, `is_ap_queued` evaluated to `false`.
  - In `tree_node_conv.rs:705`, `should_retain_ap` evaluated to `false` when `only_running` was toggled, dropping the prompts entirely.
  - In `tree_node_conv.rs:772`, the status defaulted to `"COMPLETED"` with `is_queued: false`.
  - In `src-tauri/src/modules/repo_db/queue.rs:43`, `count_enqueued_prompts` checked only `status IN ('backed_up', 'queued', 'pending', 'in_progress')`, returning count 0.
  - In `src-tauri/src/modules/repo_db/project_queries.rs:54` and line 212, queries ignored `'enqueued_ide'`.

### Defect B: Subagent Tab Focus & DOM Harvester Selector Fragility
- `EXTRACT_QUEUED_MESSAGES_JS` in `src-tauri/src/modules/instance/cdp_onboarding.rs` relied on a strict tree walker matching exact string `val === 'Queued Messages'`.
- Whenever the IDE is focused on an internal subagent tab (`tab=subagent_convo__...`) or renders dynamic counts (`Queued Messages (4)` or `Queued messages`), the walker failed to match and returned `[]`.
- No fallback existed to inspect React Fiber (`queuedMessages`, `queuedTurns`) or container query selectors.

### Defect C: Asymmetric Liveness Evaluation Between Backend Modules
- In `tree_collect.rs:151`, effective liveness recognized any live Antigravity process (`is_owning_inst_alive || is_any_antigravity_alive`).
- But in `tree_nodes.rs:45`, secondary instances checked only `is_instance_running(&inst.id, &inst.data_dir, inst.pid)`.
- If `inst.pid` in `instances.json` was stale after an IDE restart, `evaluate_project_liveness` returned `INSTANCE_PROCESS_DEAD`.
- In `tree_nodes.rs:140`, if `proj_is_running` was false, every conversation was forcibly overridden to `c.is_running = false; c.status = "IDLE"`.

### Defect D: Stale In-Flight Sweep & Fallback Project Discovery
- In `tree_phases.rs:transition_stale_inflight_prompts`, active prompts were previously swept too aggressively, turning active/enqueued items into `'completed'`.
- In `tree_phases.rs:discover_fallback_projects:213`, active prompt synthesis queried `WHERE status IN ('running', 'in_flight', 'dispatched', 'queued')`, omitting `'enqueued_ide'` and `'pending'`.
- In `tree_nodes.rs:175`, `only_running` filtered out projects where `has_any_running_conv` was false, even when `has_any_queued_turn` was true.

### Defect E: Frontend Status Classification Omission
- In `src/components/instances/PromptTreeView/usePromptTreeData.ts:221`, `isQueuedTurn` only checked `Boolean(p.is_queued) || String(p.status).toUpperCase() === 'QUEUED'`.
- Turns arriving with `status = 'enqueued_ide'`, `'dispatched'`, or `'pending'` were not recognized as queued, wiping the optimistic queued turn and clearing the queue badge.

---

## 3. Surgical Implementation & Resolution

1. **Rust Status Parity (`src-tauri/src/modules/repo_db/tree_node_conv.rs`)**:
   - Expanded `is_queued_or_pending_status` to include `"enqueued_ide"`, `"dispatched"`, and `"in_progress"`.
   - Added `find_pids_for_data_dir` fallback in `is_owning_instance_alive` for secondary instances.
   - Added unit test assertions verifying `"enqueued_ide"`, `"dispatched"`, and `"pending"` status recognition.
2. **Unified Multi-Instance Liveness & Queue Retention (`src-tauri/src/modules/repo_db/tree_nodes.rs`)**:
   - Unified `is_inst_alive` by reusing `super::tree_node_conv::is_owning_instance_alive(&proj.instance_id, registry)`.
   - Updated `only_running` filter in `tree_nodes.rs` to retain projects having `c.is_queued` or turns with `p.is_queued`.
3. **Queue Counter & Query Harmonization (`src-tauri/src/modules/repo_db/queue.rs` & `project_queries.rs`)**:
   - Updated `count_enqueued_prompts` to include `'enqueued_ide'` and `'dispatched'`.
   - Updated `list_all_prompts` and `get_live_project_execution_info` to query and prioritize `'enqueued_ide'`.
   - Updated `discover_fallback_projects` in `tree_phases.rs` to synthesize projects for `'enqueued_ide'`.
4. **Enhanced CDP Queued Message Harvester (`src-tauri/src/modules/instance/cdp_onboarding.rs`)**:
   - Upgraded `EXTRACT_QUEUED_MESSAGES_JS` to search case-insensitively for queue cards, scan dedicated test-ids (`[data-testid*="queued"]`, `[aria-label*="queued" i]`), and inspect React Fiber props (`queuedMessages`, `queuedTurns`).
5. **Frontend Queue Classification (`src/components/instances/PromptTreeView/usePromptTreeData.ts`)**:
   - Updated `isQueuedTurn` to recognize `ENQUEUED_IDE`, `DISPATCHED`, `PENDING`, and `is_queued: true`.

---

## 4. Verification & Invariants Preserved

- **FIFO Queue Ordering Preserved**: All queue queries maintain `ORDER BY created_at ASC, id ASC`.
- **Zero Silent Errors**: All fallible operations mapped to `AppError` or recorded via `crate::error::record_ignored` with justification comments.
- **Strictly Relative Paths**: All references adhere to repository-relative paths (`02-spec/...`, `src-tauri/...`, `src/...`).
- **Pre-Flight Gates Validated**:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: 100% PASS.
  - `npm run build`: 100% PASS (Vite production bundle compiled cleanly in 25s).
