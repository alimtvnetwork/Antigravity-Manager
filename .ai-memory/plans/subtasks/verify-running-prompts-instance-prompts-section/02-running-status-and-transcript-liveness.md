---
plan: verify-running-prompts-instance-prompts-section
subtask: "02"
title: Running Status and Transcript Liveness Multi-Gate Refactoring
domain: backend-rust
status: pending
citations:
  component_spec: 02-spec/21-app/verify-running-prompts-instance-prompts-section/02-component-spec.md
target_files:
  - src-tauri/src/modules/repo_db/transcript.rs
  - src-tauri/src/modules/repo_db/liveness.rs
  - src-tauri/src/modules/repo_db/tree_node_liveness.rs
  - src-tauri/src/modules/repo_db/tree_collect.rs
---

# Subtask 02: Running Status and Transcript Liveness Multi-Gate Refactoring

## 1. Context & Objectives

During autonomous agent turns and extended reasoning model runs (Claude 3.7 Thinking, Gemini 2.5 Pro), active sessions in Antigravity frequently flip to `IDLE` prematurely. This causes running prompts in the instance card header, active projects list, and `agm prompts tree` CLI to drop out before the agent completes its task.

The objectives of this subtask are:
1. **Refactor Transcript Reverse Scanner (`transcript.rs`)**:
   - Differentiate step-level completion (e.g. tool execution steps `type == "GENERIC"`, `type == "TOOL_OUTPUT"`, or step `status: "DONE"`) from overall assistant turn completion (`PLANNER_RESPONSE` / `MODEL` with zero tool calls and terminal status).
   - Prevent individual tool step outputs from causing `is_completed_or_waiting = true`.
   - Incorporate physical transcript file `mtime` as an authoritative real-time heartbeat corroboration signal.
   - Expand `is_recent_active` evaluation window to 600s (10 minutes) for turns without terminal completion.
2. **Refactor Gate 4 in Project Liveness Checker (`liveness.rs`)**:
   - Establish `not_fully_idle > 0` as authoritative evidence of an active turn regardless of stale `status` strings such as `CASCADE_RUN_STATUS_IDLE`.
   - Expand the timestamp freshness window to 600s (10 minutes) when `not_fully_idle > 0`.
3. **Refactor Project Tree Node Liveness (`tree_node_liveness.rs`)**:
   - Replace the rigid 45-second window (`now - conv_ts <= 45`) on active conversation nodes with an adaptive 600-second window when `c.is_running` is verified.
4. **Align Conversation Collection (`tree_collect.rs`)**:
   - Harmonize `is_conv_running` determination with the authoritative `not_fully_idle > 0` rule and the updated transcript scanner inspection results.

---

## 2. Verified Architectural Invariants

- **Invariant I1 (Step-Level vs. Turn-Level Distinction)**: A tool execution output step (`GENERIC`, `TOOL_OUTPUT`, `TOOL_RESPONSE`, or containing `tool_call_id`) indicating `DONE` signifies only that the specific tool finished executing. It MUST NOT mark the conversation turn as terminal or completed.
- **Invariant I2 (`not_fully_idle > 0` Turn Supremacy)**: When `conversation_summaries.db` records `not_fully_idle > 0`, this numerical counter is empirical evidence of active work. Stale status strings (`CASCADE_RUN_STATUS_IDLE` or `IDLE`) MUST NOT override `not_fully_idle > 0`.
- **Invariant I3 (Adaptive 600s Thinking Window)**: For active turns with `not_fully_idle > 0` or non-terminal transcripts, the timestamp cutoff is expanded to 600s (10 minutes) to prevent premature timeouts during deep model reasoning and long tool invocations.
- **Invariant I4 (Transcript `mtime` Heartbeat)**: The modification timestamp (`mtime`) of `transcript_full.jsonl` / `transcript.jsonl` provides continuous real-time verification of agent execution even when SQLite summary records have not yet flushed to disk.

---

## 3. Concrete File Diff Specifications

### 3.1 `src-tauri/src/modules/repo_db/transcript.rs`

**Location**: Lines 264–310 in `src-tauri/src/modules/repo_db/transcript.rs`

#### Before:
```rust
    // Check if the conversation turn has completed or is waiting for user
    let mut is_completed_or_waiting = false;
    for line in lines.iter().rev().take(25) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if s_type == "TOKEN_USAGE"
                || s_type == "TELEMETRY"
                || s_type == "HEARTBEAT"
                || s_type == "PROGRESS"
                || s_type == "METRICS"
                || s_type == "SYSTEM_LOG"
            {
                continue;
            }
            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");
            if s_status.eq_ignore_ascii_case("done")
                || s_status.eq_ignore_ascii_case("completed")
                || s_status.eq_ignore_ascii_case("error")
                || s_status.eq_ignore_ascii_case("idle")
            {
                is_completed_or_waiting = true;
                break;
            }
            if s_type == "PLANNER_RESPONSE"
                || val.get("source").and_then(|s| s.as_str()) == Some("MODEL")
            {
                let has_tool_calls = val
                    .get("tool_calls")
                    .and_then(|t| t.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if !has_tool_calls
                    && (s_status.is_empty()
                        || s_status.eq_ignore_ascii_case("done")
                        || s_status.eq_ignore_ascii_case("completed")
                        || s_status.eq_ignore_ascii_case("idle"))
                {
                    is_completed_or_waiting = true;
                    break;
                }
            }
        }
    }

    let is_recent_active =
        !is_completed_or_waiting && mtime_epoch > 0 && (now_epoch - mtime_epoch <= 60);
```

#### After:
```rust
    // Check if the conversation turn has completed or is waiting for user.
    // Invariant: Differentiate step-level tool completion from turn-level assistant completion.
    let mut is_completed_or_waiting = false;
    for line in lines.iter().rev().take(25) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");
            if s_type == "TOKEN_USAGE"
                || s_type == "TELEMETRY"
                || s_type == "HEARTBEAT"
                || s_type == "PROGRESS"
                || s_type == "METRICS"
                || s_type == "SYSTEM_LOG"
            {
                continue;
            }

            // Step-level outputs (tool execution results) report status: "DONE" on the step itself.
            // These indicate tool completion, NOT turn completion. Skip them.
            let is_tool_step = s_type == "GENERIC"
                || s_type == "TOOL_OUTPUT"
                || s_type == "TOOL_RESPONSE"
                || s_source == "TOOL"
                || val.get("tool_call_id").is_some();
            if is_tool_step {
                continue;
            }

            let s_status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");

            // Explicit turn-level terminal completion: Planner/Model response with ZERO tool calls
            if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
                let has_tool_calls = val
                    .get("tool_calls")
                    .and_then(|t| t.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if !has_tool_calls
                    && (s_status.is_empty()
                        || s_status.eq_ignore_ascii_case("done")
                        || s_status.eq_ignore_ascii_case("completed")
                        || s_status.eq_ignore_ascii_case("idle")
                        || s_status.eq_ignore_ascii_case("error"))
                {
                    is_completed_or_waiting = true;
                    break;
                }
            } else if s_type == "USER_INPUT" || s_source == "USER_EXPLICIT" {
                // If the most recent meaningful event is user input awaiting AI, turn is active (not completed)
                break;
            } else if s_status.eq_ignore_ascii_case("completed")
                || s_status.eq_ignore_ascii_case("error")
                || s_status.eq_ignore_ascii_case("idle")
            {
                is_completed_or_waiting = true;
                break;
            }
        }
    }

    // Adaptive 10-Minute Thinking Window: Expand timestamp window to 600s (10 minutes) when mtime is fresh
    let is_mtime_fresh = mtime_epoch > 0 && (now_epoch - mtime_epoch <= 600);
    let is_recent_active = !is_completed_or_waiting && is_mtime_fresh;
```

---

### 3.2 `src-tauri/src/modules/repo_db/liveness.rs`

**Location**: Lines 277–305 in `src-tauri/src/modules/repo_db/liveness.rs`

#### Before:
```rust
                        // Strict idle supremacy rule:
                        // if not_fully_idle == 0 or status contains "IDLE", "COMPLETED", "FAILED", "CANCELLED", it is unconditionally IDLE.
                        let is_idle_count = not_fully_idle == 0;
                        let has_idle_status = status.contains("IDLE")
                            || status.contains("COMPLETED")
                            || status.contains("FAILED")
                            || status.contains("CANCELLED");
                        let is_explicit_idle = is_idle_count || has_idle_status;

                        if is_explicit_idle {
                            continue;
                        }

                        // Adaptive 10-Minute Thinking Window: Expand the timestamp cutoff from rigid 120s to 600s (10 minutes)
                        // for active turns with not_fully_idle > 0 && status.contains("RUNNING"), ensuring reasoning/thinking models
                        // (Claude 3.7 Thinking, Gemini 2.5 Pro) are not prematurely flipped to IDLE mid-generation!
                        let conv_time = parse_flexible_timestamp(&_last_time_str);
                        let is_recent = conv_time > 0 && (now - conv_time <= 60);

                        if !is_recent {
                            continue;
                        }

                        let is_conv_running = not_fully_idle > 0 && status.contains("RUNNING");
```

#### After:
```rust
                        // Authoritative not_fully_idle supremacy rule:
                        // If not_fully_idle > 0, this is definitive empirical proof of active in-flight execution.
                        // Stale text status strings (e.g. CASCADE_RUN_STATUS_IDLE) must not override not_fully_idle > 0.
                        let is_zero_idle = not_fully_idle == 0;
                        let has_terminal_status = status.contains("COMPLETED")
                            || status.contains("FAILED")
                            || status.contains("CANCELLED");
                        let is_explicit_idle = is_zero_idle && (status.contains("IDLE") || has_terminal_status);

                        if is_explicit_idle {
                            continue;
                        }

                        // Adaptive 10-Minute Thinking Window: Expand timestamp cutoff to 600s (10 minutes)
                        // for active turns with not_fully_idle > 0, ensuring reasoning/thinking models
                        // (Claude 3.7 Thinking, Gemini 2.5 Pro) are never prematurely flipped to IDLE mid-generation.
                        let conv_time = parse_flexible_timestamp(&_last_time_str);
                        let max_allowed_elapsed = if not_fully_idle > 0 { 600 } else { 60 };
                        let is_recent = conv_time > 0 && (now - conv_time <= max_allowed_elapsed);

                        if !is_recent {
                            continue;
                        }

                        let is_conv_running = not_fully_idle > 0;
```

---

### 3.3 `src-tauri/src/modules/repo_db/tree_node_liveness.rs`

**Location**: Lines 19–26 in `src-tauri/src/modules/repo_db/tree_node_liveness.rs`

#### Before:
```rust
    let has_active_conv = conv_nodes.iter().any(|c| {
        if !c.is_running {
            return false;
        }
        let conv_ts = parse_flexible_timestamp(&c.last_modified);
        conv_ts > 0 && (now - conv_ts <= 45)
    });
```

#### After:
```rust
    // Adaptive 10-Minute Thinking Window: For conversations already evaluated as running,
    // allow up to 600s elapsed window to accommodate extended reasoning and heavy tool execution.
    let has_active_conv = conv_nodes.iter().any(|c| {
        if !c.is_running {
            return false;
        }
        let conv_ts = parse_flexible_timestamp(&c.last_modified);
        conv_ts > 0 && (now - conv_ts <= 600)
    });
```

---

### 3.4 `src-tauri/src/modules/repo_db/tree_collect.rs`

**Location**: Lines 156–177 in `src-tauri/src/modules/repo_db/tree_collect.rs`

#### Before:
```rust
                            let now = chrono::Utc::now().timestamp();
                            let conv_ts = parse_flexible_timestamp(&last_time_str);
                            let is_recent = conv_ts > 0 && (now - conv_ts <= 60);

                            let is_idle_count = not_fully_idle == 0;
                            let has_idle_status = status.contains("IDLE")
                                || status.contains("COMPLETED")
                                || status.contains("FAILED")
                                || status.contains("CANCELLED");
                            let is_explicit_idle = is_idle_count || has_idle_status;

                            let inspection = inspect_conversation_transcript(base, &cid);

                            let is_conv_running = if is_explicit_idle || inspection.is_terminal_done {
                                false
                            } else if is_owning_inst_alive && not_fully_idle > 0 && status.contains("RUNNING") && is_recent {
                                true
                            } else if is_owning_inst_alive && inspection.is_recent_active && !has_idle_status {
                                true
                            } else {
                                false
                            };
```

#### After:
```rust
                            let now = chrono::Utc::now().timestamp();
                            let conv_ts = parse_flexible_timestamp(&last_time_str);
                            let max_allowed_elapsed = if not_fully_idle > 0 { 600 } else { 60 };
                            let is_recent = conv_ts > 0 && (now - conv_ts <= max_allowed_elapsed);

                            let is_zero_idle = not_fully_idle == 0;
                            let has_terminal_status = status.contains("COMPLETED")
                                || status.contains("FAILED")
                                || status.contains("CANCELLED");
                            let is_explicit_idle = is_zero_idle && (status.contains("IDLE") || has_terminal_status);

                            let inspection = inspect_conversation_transcript(base, &cid);

                            let is_conv_running = if is_explicit_idle || inspection.is_terminal_done {
                                false
                            } else if is_owning_inst_alive && not_fully_idle > 0 && is_recent {
                                true
                            } else if is_owning_inst_alive && inspection.is_recent_active && !has_terminal_status {
                                true
                            } else {
                                false
                            };
```

---

## 4. Acceptance Criteria

| Check | Assertion | Status |
| :--- | :--- | :--- |
| `has_turn_level_differentiation` | Tool execution steps (`GENERIC`, `TOOL_OUTPUT`, `status: "DONE"`) do not trigger `is_completed_or_waiting = true` | Required |
| `is_not_fully_idle_authoritative` | `not_fully_idle > 0` marks the turn running even if status text contains `CASCADE_RUN_STATUS_IDLE` | Required |
| `has_600s_thinking_window` | Active turns with `not_fully_idle > 0` remain running for up to 600 seconds without premature timeout | Required |
| `is_mtime_heartbeat_corroborated` | Transcript file write modification time (`mtime`) is checked and validates ongoing turn execution | Required |
| `has_tree_liveness_alignment` | `tree_node_liveness.rs` and `tree_collect.rs` apply the 600s window and respect `is_conv_running` | Required |
| `has_zero_compilation_errors` | `cargo clippy --all-targets --all-features` compiles with 0 errors and 0 warnings | Required |

---

## 5. Targeted Verification Commands

To verify these changes upon implementation:

1. **Rust Formatting Gate**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
2. **Rust Clippy Gate**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
3. **Unit Tests for Repo DB Module**:
   ```bash
   cd src-tauri && cargo test modules::repo_db
   ```
4. **CLI Prompts Tree Inspection**:
   ```bash
   src-tauri/target/debug/agm.exe prompts tree --json
   src-tauri/target/debug/agm.exe tree
   ```
