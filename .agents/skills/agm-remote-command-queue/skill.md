---
name: agm-remote-command-queue
description: Specialized skill for managing the Supabase PostgreSQL secondary command queue, multi-endpoint cascading failover, headless remote execution, sliding timeouts, and execution telemetry in Antigravity-Manager.
---

# AGM Remote Command Queue & Secondary Cascade Architecture

This skill provides architectural guidance, schema contracts, failover ladders, and operational commands for the Supabase Remote Command Queue subsystem in Antigravity-Manager (`src-tauri/src/modules/supabase_command_queue.rs`).

---

## 1. Subsystem Architecture Overview

The Remote Command Queue allows headless cluster nodes to receive, execute, and report prompt instructions and administrative tasks without requiring direct inbound open ports or public IPs.

```
+------------------------------------------------------------------------------------+
|                         Supabase Secondary Endpoints                               |
|        Priority 1 (Secondary A) --------> Priority 2 (Secondary B)                |
+------------------------------------------+-----------------------------------------+
                                           |
                                           v
+------------------------------------------------------------------------------------+
|                                Command Poller Loop                                 |
|  - Queries `command_queue` where `status = 'pending'` and `target_node_id` matches  |
|  - Selects oldest pending commands (`order=created_at.asc&limit=5`)               |
|  - Cascades across secondary endpoints if primary secondary fails                  |
+------------------------------------------------------------------------------------+
                                           |
                                           v
+------------------------------------------------------------------------------------+
|                             Execution & State Engine                               |
|  1. Claim command: transition status `pending` -> `executing` (`started_at = now`) |
|  2. Execute locally via `CommandExtWrapper` with stdout/stderr capture             |
|  3. Complete command: transition `executing` -> `completed` / `failed`             |
|  4. Upload output logs and execution metrics back to Supabase                      |
+------------------------------------------------------------------------------------+
```

---

## 2. Inbound Command Schema & State Machine

### DTO Schema (`InboundDbCommand`)
```rust
pub struct InboundDbCommand {
    pub id: String,
    pub target_node_id: String,      // Exact node ID or wildcard '*'
    pub source: String,              // 'cli', 'telegram', 'web', 'mesh'
    pub command_text: String,        // Command string to execute
    pub status: String,              // 'pending' | 'executing' | 'completed' | 'failed'
    pub created_at: i64,             // Unix timestamp in seconds
    pub started_at: i64,
    pub completed_at: i64,
}
```

### State Machine Lifecycle
- **Pending (`pending`)**: The command has been posted by an operator or remote node.
- **Executing (`executing`)**: A node with matching `target_node_id` or wildcard `*` claims the command, writing `started_at = Utc::now().timestamp()`.
- **Completed (`completed`)**: Execution finished successfully (`exit code 0`). Output is captured in `result_output`.
- **Failed (`failed`)**: Execution exited with non-zero code or timed out. Error details are captured in `error_output`.

---

## 3. Cascading Multi-Endpoint Failover

In [`src-tauri/src/modules/supabase_command_queue.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/supabase_command_queue.rs):
- `get_secondary_endpoints(config)`: Retrieves all enabled secondary database endpoints sorted by `priority` ascending.
- `fetch_pending_commands(config, node_id)`: Probes secondary endpoints in priority order. If Endpoint 1 is unreachable or times out, it gracefully cascades to Endpoint 2.
- `update_command_status`: Writes state updates back through the same client that successfully claimed the task.

---

## 4. Maintenance Checklist & Critical Invariants

1. **Target Specificity & Wildcard Handling**:
   Nodes must query `target_node_id=eq.<my_node_id>|target_node_id=eq.*`. Wildcard commands are picked up by the first available node; once transitioned to `executing`, no other node may execute it.
2. **Execution Sandboxing & Privilege Hardening**:
   Commands dispatched via the remote queue run through `CommandExtWrapper`. Dangerous commands affecting the host workstation IDE must be intercepted by safety guards.
3. **Execution Timeout Guard**:
   Commands running longer than the configured execution limit (default 300 seconds) must be terminated to prevent worker starvation.
