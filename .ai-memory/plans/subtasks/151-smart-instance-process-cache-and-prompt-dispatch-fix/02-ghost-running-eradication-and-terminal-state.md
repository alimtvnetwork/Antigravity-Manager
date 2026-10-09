# Subtask Implementation Plan: Ghost Running Eradication & Terminal State Detection

> **Subtask ID:** `02-ghost-running-eradication-and-terminal-state`  
> **Parent Slug:** `151-smart-instance-process-cache-and-prompt-dispatch-fix`  
> **Target Files:** `src-tauri/src/modules/repo_db.rs`  
> **Status:** `[READY]`  
> **Author:** @aukgit  
> **Protocol:** `execute-parent-task-with-n-steps-v6`  

---

## 1. Context & Objectives

Users report that finished conversations and idle projects frequently display a pulsing green "RUNNING" state in the Prompt Tree View and Instance Cards long after work has ceased. Even after an IDE instance has been closed for minutes, AGM continues to report it as actively running tasks.

### Core Objectives:
1. **Deep Reverse Transcript Scan**: Expand reverse scanning in `inspect_conversation_transcript` from `take(5)` to `take(25)` lines, comprehensively filtering out trailing telemetry noise (`TOKEN_USAGE`, `TELEMETRY`, `HEARTBEAT`, `PROGRESS`, `METRICS`, `SYSTEM_LOG`).
2. **Broadened Terminal State Recognition**: Recognize `MODEL` and `PLANNER_RESPONSE` turn completions when open tool calls are empty, even if the explicit `"status": "DONE"` field is omitted or blank (standard Antigravity transcript format).
3. **OS-Bound Project Liveness Guard**: Guard the fallback project discovery loop in `compute_project_conversation_tree` with an explicit `is_owning_inst_alive` verification, preventing closed IDE instances with stale SQLite markers from reporting `is_running = true`.
4. **Hardened Active Prompt TTLs**: Tighten unconfirmed active prompt expiration from 600 seconds to **45 seconds** unless an active living OS worker process is verified.

---

## 2. Step-by-Step Implementation Details

### Step 1: Deep Reverse Transcript Inspection (`take(25)`) with Telemetry Filtering

**File:** `src-tauri/src/modules/repo_db.rs` (lines 4471–4503 & 4638–4671)

#### Current Problem:
- The parser only scans the last 5 lines (`lines.iter().rev().take(5)`) for `is_completed_or_waiting`, and the last 10 lines (`take(10)`) for `is_terminal_done`.
- Antigravity appends multiple trailing metadata lines upon completion: `TOKEN_USAGE`, `TELEMETRY`, `HEARTBEAT`, `PROGRESS`, and `METRICS`.
- When more than 5 telemetry lines are appended, the reverse scan terminates without evaluating any actual message lines, leaving `is_completed_or_waiting` as `false` and triggering a false-positive running state.

#### Proposed Implementation:
```rust
// In inspect_conversation_transcript (src-tauri/src/modules/repo_db.rs):

// 1. Deep scan for turn completion or user waiting state (expanded to 25 lines)
let mut is_completed_or_waiting = false;
for line in lines.iter().rev().take(25) {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
        let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
        
        // Comprehensive telemetry and progress noise filter
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
            
            // Empty status without open tool calls signals completed model response
            if !has_tool_calls && (s_status.is_empty() || s_status.eq_ignore_ascii_case("done") || s_status.eq_ignore_ascii_case("completed")) {
                is_completed_or_waiting = true;
                break;
            }
        }
    }
}
```

---

### Step 2: Broadened Terminal State Recognition in `is_terminal_done`

**File:** `src-tauri/src/modules/repo_db.rs` (lines 4638–4671)

#### Current Problem:
`is_terminal_done` previously required:
```rust
if (s_status.eq_ignore_ascii_case("DONE") || s_status.eq_ignore_ascii_case("COMPLETED")) && !has_open_tools {
    is_terminal_done = true;
}
```
In many Antigravity transcript logs, the final assistant message has `type: "PLANNER_RESPONSE"` and `source: "MODEL"`, but the `status` field is completely omitted or null. Because `s_status` was `"running"` by default, `is_terminal_done` remained `false`.

#### Proposed Implementation:
```rust
let mut is_terminal_done = false;

// Deep scan: inspect up to 25 lines in reverse order to bypass telemetry noise
for line in lines.iter().rev().take(25) {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
        let s_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let s_source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");

        // Bypass pure telemetry / token usage metadata
        if s_type == "TOKEN_USAGE" 
            || s_type == "TELEMETRY" 
            || s_type == "HEARTBEAT"
            || s_type == "PROGRESS"
            || s_type == "METRICS"
            || s_type == "SYSTEM_LOG"
        {
            continue;
        }

        let s_status = val
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Inspect for terminal completion
        if s_type == "PLANNER_RESPONSE" || s_source == "MODEL" {
            let tool_calls = val.get("tool_calls").and_then(|tc| tc.as_array());
            let has_open_tools = tool_calls.map(|tc| !tc.is_empty()).unwrap_or(false);
            
            let is_explicit_done = s_status.eq_ignore_ascii_case("DONE")
                || s_status.eq_ignore_ascii_case("COMPLETED")
                || s_status.eq_ignore_ascii_case("IDLE");
            
            // Turn is terminal if explicitly marked done OR if model finished without open tool calls
            if !has_open_tools && (is_explicit_done || s_status.is_empty()) {
                is_terminal_done = true;
                break;
            }
        }
        
        // If a USER_INPUT was reached and it is waiting for response, break scan
        if s_type == "USER_INPUT" || s_source == "USER_EXPLICIT" {
            break;
        }
    }
}
```

---

### Step 3: Strict OS Liveness Guard in Fallback Projects Discovery

**File:** `src-tauri/src/modules/repo_db.rs` (lines 4964–5038)

#### Current Problem:
When `candidate_dirs_fb` iterates over `conversation_summaries.db`, it extracts projects and determines running state using:
```rust
let is_running = status.contains("RUNNING") && not_fully_idle > 0 && is_recent;
```
It **never checks if the instance owning that directory is actually running on the OS**. If the IDE exited 30 seconds ago, the database still reports `RUNNING`, causing the fallback project to be added with `is_running = true`.

#### Proposed Implementation:
```rust
for (owning_inst_id, base) in &candidate_dirs_fb {
    // 1. Verify if the owning instance process is actually alive on the OS
    let is_owning_inst_alive = if owning_inst_id == "default" || owning_inst_id == "__default__" {
        crate::modules::process::is_antigravity_running(None) || {
            let def_dir = crate::modules::instance::get_default_antigravity_data_dir();
            !crate::modules::instance::find_pids_for_data_dir(&def_dir.to_string_lossy(), true).is_empty()
        }
    } else if let Some(inst) = registry.instances.iter().find(|i| i.id == *owning_inst_id || i.name == *owning_inst_id) {
        let (is_running, _, _) = crate::modules::instance::is_instance_process_running_smart(&inst.id);
        is_running
    } else {
        false
    };

    let summaries_db = base.join("conversation_summaries.db");
    if !summaries_db.exists() {
        continue;
    }
    // ...
    // Guard is_running with OS liveness check:
    let is_running = is_owning_inst_alive && status.contains("RUNNING") && not_fully_idle > 0 && is_recent;
    // ...
}
```

---

### Step 4: Hardening Active Prompt TTLs to 45 Seconds

**File:** `src-tauri/src/modules/repo_db.rs` (lines 4887–4896 & 5632–5637)

#### Current Problem:
- Stale prompts in `active_prompts` were only transitioned to `completed` after 300 seconds (`now - updated_at > 300`).
- During tree node construction, unconfirmed active prompts were allowed up to 600 seconds (`now - ap.updated_at <= 600`).
- This caused dead prompts to linger in the tree view for 5–10 minutes.

#### Proposed Implementation:
1. **Automated SQLite Stale Prompt Cleanup**:
   ```rust
   // Automatically transition unconfirmed in-flight prompts (> 45s without update) to completed
   if let Ok(conn) = connect_db() {
       let _ = conn.execute(
           "UPDATE active_prompts 
            SET status = 'completed', updated_at = ?1 
            WHERE status IN ('running', 'in_flight', 'dispatched') 
              AND (?1 - updated_at > 45)",
           rusqlite::params![now],
       );
   }
   ```
2. **Strict Active Prompt Tree Liveness Evaluation**:
   ```rust
   // Enforce 45s TTL unless confirmed active OS worker process is alive
   let max_allowed_ttl = if has_active_worker { 300 } else { 45 };
   let is_run = is_inst_alive
       && (ap.status == "running" || ap.status == "in_flight" || ap.status == "dispatched")
       && (now - ap.updated_at <= max_allowed_ttl)
       && has_confirmed_running;
   ```

---

## 3. Verification & Testing Checklist

- [ ] **25-Line Reverse Scan Test**: Feed a transcript ending with 12 trailing `TOKEN_USAGE` and `TELEMETRY` records into `inspect_conversation_transcript`; verify `is_terminal_done` accurately resolves to `true`.
- [ ] **Omitted Status Recognition Test**: Feed a transcript where the last line is `{"type": "PLANNER_RESPONSE", "source": "MODEL", "content": "Done"}` without a `status` field; verify `is_terminal_done` is `true`.
- [ ] **Closed IDE Fallback Project Test**: Simulate a closed IDE with a recent SQLite timestamp; verify `compute_project_conversation_tree` flags `is_running = false`.
- [ ] **45s Active Prompt TTL Test**: Insert an active prompt with `updated_at = now - 50`; verify it is filtered out of running nodes and transitioned to `completed`.

---

## 4. Risk Assessment & Invariants

| Invariant | Description |
| :--- | :--- |
| **Strict OS Parity** | A project or prompt can never be marked running if its owning instance process is dead on the host OS. |
| **Telemetry Immunity** | Trailing telemetry metadata bursts must never obscure legitimate assistant turn completions. |
| **FIFO Ordering** | All database queries continue to use `ORDER BY created_at ASC, id ASC`. |
