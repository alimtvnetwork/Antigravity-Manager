---
plan: 153-verify-running-prompts-instance-section
subtask: "01"
title: Backend Liveness Pipeline and Multi-Gate Verification
domain: backend-rust
status: verified
evidence:
  - "Gate 0 dead-process verification enforced in is_prompt_running_for_project and evaluate_project_liveness"
  - "Gate 3 SQLite active_prompts freshness window bound to 45s sliding TTL"
  - "Gate 4 Idle Supremacy Rule enforced over stale records in conversation_summaries.db"
  - "Deep Transcript reverse-scan inspecting up to 25 lines in transcript_full.jsonl bypassing telemetry"
verified_invariants:
  - "Invariant I1 (Gate 0 Host Process Liveness): No project or conversation is marked running when host PID is dead"
  - "Invariant I2 (Gate 3 Active Prompts Freshness): Database prompt entries older than 45s are ignored"
  - "Invariant I3 (Gate 4 Idle Supremacy): not_fully_idle == 0 or explicit terminal status unconditionally forces IDLE"
  - "Invariant I4 (Deep Transcript Inspection): Prioritize transcript_full.jsonl over transcript.jsonl and extract tool_calls/thinking"
citations:
  architecture_spec: 02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md
  root_cause_analysis: 02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md
target_files:
  - src-tauri/src/modules/repo_db/liveness.rs
  - src-tauri/src/modules/repo_db/project_queries.rs
  - src-tauri/src/modules/repo_db/tree_node_liveness.rs
  - src-tauri/src/modules/repo_db/tree_node_conv.rs
  - src-tauri/src/modules/repo_db/transcript.rs
  - src-tauri/src/modules/instance.rs
---

# Subtask 01: Backend Liveness Pipeline and Multi-Gate Verification

## 1. Context & Objectives

The primary failure mode in running prompt detection is "ghost liveness", where past tasks or abandoned worker records permanently report `RUNNING` status on an Antigravity instance. This subtask verifies the backend Rust implementation of the multi-gate liveness ladder across `repo_db`, ensuring that active prompt status is verified against real operating system processes, strict TTL windows, and empirical database turns.

Key verification objectives:
1. Validate Gate 0 (`is_instance_process_running_smart`) as the foundational precondition for any running task.
2. Verify Gate 1 (In-Memory Map) and Gate 2 (Active Workers Map) scoping and live process probing.
3. Verify Gate 3 (SQLite `active_prompts`) sliding TTL window (45s) and terminal status override.
4. Verify Gate 4 (Conversation Summaries) Idle Supremacy Rule and ghost conversation filtering.
5. Verify Deep Transcript reverse-scanning for bypassing telemetry envelopes and extracting real-time tool calls and model reasoning.

---

## 2. Verified Architectural Invariants

### Invariant I1: Gate 0 Host Process Liveness
- **Rule**: If an instance has no running OS process, all associated projects and conversations are strictly forced to `IDLE`.
- **Implementation**:
  - In `src-tauri/src/modules/repo_db/liveness.rs` (lines 31-49): `crate::modules::instance::is_instance_process_running_smart(norm_inst)` queries the system process table. If `has_active_process` is `false`, it logs `INSTANCE_PROCESS_DEAD` and returns `false`.
  - In `src-tauri/src/modules/repo_db/tree_node_liveness.rs` (lines 28-33): If `!is_inst_alive`, returns `(false, "INSTANCE_PROCESS_DEAD...")`.
  - In `src-tauri/src/modules/repo_db/project_queries.rs` (lines 277-292): `live_map` entries are ignored if `!is_inst_alive`.

### Invariant I2: Gate 3 Active Prompts Freshness & Terminal Status Override
- **Rule**: SQLite records in `active_prompts` only grant running status if touched within 45 seconds (`updated_at >= now - 45`) and not superseded by a terminal state (`completed` or `failed`).
- **Implementation**:
  - In `src-tauri/src/modules/repo_db/liveness.rs` (lines 51-82, 180-211): Queries `active_prompts` checking `updated_at >= ?3` with `?3 = now - 45`. Terminal transitions in memory or the database immediately clamp `running_count` to 0.

### Invariant I3: Gate 4 Idle Supremacy & Ghost Conversation Filtering
- **Rule**: If `not_fully_idle == 0` or status contains `IDLE`, `COMPLETED`, `FAILED`, or `CANCELLED`, the conversation is unconditionally IDLE. Ghost conversations (untitled with 0 words and empty preview) are filtered out.
- **Implementation**:
  - In `src-tauri/src/modules/repo_db/liveness.rs` (lines 264-288): Filters out ghost conversations and applies explicit idle checks:
    ```rust
    let is_idle_count = not_fully_idle == 0;
    let has_idle_status = status.contains("IDLE")
        || status.contains("COMPLETED")
        || status.contains("FAILED")
        || status.contains("CANCELLED");
    if is_idle_count || has_idle_status {
        continue;
    }
    ```
  - An active turn requires `not_fully_idle > 0 && status.contains("RUNNING")` with `now - conv_time <= 60` (adaptively extended up to 600s during active model reasoning).

### Invariant I4: Deep Transcript Reverse-Scan & Telemetry Bypass
- **Rule**: Prioritize `transcript_full.jsonl` over `transcript.jsonl`. Reverse-scan up to 25 lines to bypass trailing telemetry records (`TOKEN_USAGE`, `HEARTBEAT`, `TELEMETRY`, `PROGRESS`, `METRICS`, `SYSTEM_LOG`), extracting in-flight `tool_calls` and `thinking` blocks.
- **Implementation**:
  - In `src-tauri/src/modules/repo_db/transcript.rs` (lines 196-220, 264-306, 429-545): `resolve_transcript_path` selects `transcript_full.jsonl`. Reverse loop skips metadata envelopes and populates `latest_step_summary`, `execution_results`, and `latest_response` from real tool calls and thinking snippets.

---

## 3. Technical Evidence & Audit Findings

1. **Process Table Verification**: Audited `src-tauri/src/modules/instance.rs`. The process detection function `is_instance_process_running_smart` checks running processes matching the instance user data directory (`--user-data-dir`) and resolves live PIDs accurately.
2. **Conversation Summaries Ingestion**: Audited `src-tauri/src/modules/repo_db/liveness.rs`. Ingestion uses read-only SQLite flags with immutable mode fallback, ensuring zero database lock contention on active IDE sessions.
3. **Structured Audit Trail**: Verified that all gate evaluations emit structured telemetry via `log_instance_prompt_audit` with clear action codes (`Gate0:HostProcessLiveness`, `Gate1:MemoryPromptsMap`, `Gate2:ActiveAgyWorkers`, `Gate3:ActivePromptsSQLite`, `Gate4:ConversationSummariesLiveTurn`), providing instant traceability.

---

## 4. Target Files & Relative References

- `src-tauri/src/modules/repo_db/liveness.rs` — Primary 5-gate liveness checker function (`is_prompt_running_for_project`).
- `src-tauri/src/modules/repo_db/project_queries.rs` — Live execution status queries and summaries database scanner (`get_live_project_execution_info`).
- `src-tauri/src/modules/repo_db/tree_node_liveness.rs` — Project-level tree node liveness evaluation (`evaluate_project_liveness`).
- `src-tauri/src/modules/repo_db/tree_node_conv.rs` — Conversation node generation and running flag binding (`build_transcript_conv_nodes`).
- `src-tauri/src/modules/repo_db/transcript.rs` — Deep transcript tail reading and reverse-scanning (`inspect_conversation_transcript`).
- `src-tauri/src/modules/instance.rs` — Instance process inspection and PID liveness validation (`is_instance_process_running_smart`).
- `02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md` — Architectural specification.
- `02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md` — Root cause analysis and matrix.
