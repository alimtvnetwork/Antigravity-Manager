# Specification: Backend Fleet Synchronization & Prompt Dispatch Root Cause Analysis (RCA)

- **Slug**: `ui-and-functionality-enhancements`
- **Module**: `Backend Systems, Fleet Remote Sync & Prompt Queue Engine`
- **Specification Path**: `02-spec/21-app/ui-and-functionality-enhancements/02-backend-fleet-and-rca-spec.md`
- **Parent Plan**: `.ai-memory/plans/ui-and-functionality-enhancements.md`
- **Master Ledger**: `02-spec/21-app/ui-and-functionality-enhancements/00-master-audit-ledger.md`
- **Related UI Spec**: `02-spec/21-app/ui-and-functionality-enhancements/01-architecture-and-ui-spec.md`
- **Benchmark Branch**: `backup/ui-and-functionality-enhancements-benchmark`
- **Active Working Branch**: `main`
- **Target Release**: `v4.207.0` (Stable Minor Release)
- **Status**: `[APPROVED & IMPLEMENTATION READY]`

---

## 1. Executive Summary & Architectural Scope

This specification establishes two foundational backend capabilities for Antigravity-Manager:
1. **Exhaustive 4-Part Root Cause Analysis (RCA)**: Deconstructs the historical failure modes in prompt dispatching and FIFO queue advancement (`send_prompt_now` and `enqueue_prompt`), documents the surgical code resolutions in `repo_db/agy.rs` and `commands/instance.rs`, and codifies non-negotiable runtime invariants.
2. **Fleet Remote Information Connectivity Engine**: Defines the multi-machine discovery, instance inventory, and account quota synchronization protocol using pure JSON payloads (`FleetSyncPayload` schema `agm-fleet-sync-v1.0` and `[AGM IN-USE]` lease claims) across a hybrid 3-tier transport architecture (Supabase PostgREST, Email IMAP/SMTP envelopes, and Telegram bot commands).

```
+----------------------------------------------------------------------------------------------------+
|                                    Antigravity Manager Gateway                                     |
|                                                                                                    |
|    +--------------------------+        +--------------------------+        +---------------------+ |
|    |   Prompt Dispatch Engine |        |   FIFO Queue Engine      |        | Fleet Sync Engine   | |
|    |   - spawn_or_replace_agy |        |   - drain_next_queued    |        | - JSON Telemetry    | |
|    |   - Live CDP Fallback    |        |   - ORDER BY created_at  |        | - 3-Tier Transports | |
|    |   - Poison-safe Mutex    |        |   - Monitor Thread Exit  |        | - Account Lease     | |
|    +------------+-------------+        +------------+-------------+        +----------+----------+ |
+-----------------|-----------------------------------|---------------------------------|------------+
                  |                                   |                                 |
                  v                                   v                                 v
        [ Antigravity IDE ]                [ SQLite active_prompts ]           [ Decentralized Fleet ]
        (CDP Composer DOM)                 (Split DB: repodb.sqlite)           (Supabase / Email / TG)
```

---

## 2. Root Cause Analysis (RCA): Prompt Sending & FIFO Enqueueing Fixes

### 2.1 Failure Taxonomy & Problem Statement

Historically, users experienced intermittent failures when sending or enqueueing prompts from the AGM GUI (specifically within `PromptTreeView` and `DetailHeader`). Prompts would occasionally lock up the UI, report success while remaining invisible in the IDE, fail to advance the queue, or reject subsequent prompt attempts with misleading errors.

Analysis across user telemetry, logs, and transcripts identified **six distinct failure modes (F1 – F6)**:

| Failure ID | Failure Mode | Observable Symptom | Impact Level |
|---|---|---|---|
| **F1** | **Lingering Worker Lockout** | Subsequent `Send` attempts failed with *"agy is already running for this workspace (pid XXXX); prompt was not sent again"*, even though the previous turn had completed or hung. | `CRITICAL` |
| **F2** | **Poisoned Mutex Cascades** | In-memory worker maps (`ACTIVE_AGY_WORKERS`) panicked during `.lock().unwrap()` if any prior thread panicked while holding the guard, permanently crashing future dispatch calls. | `CRITICAL` |
| **F3** | **Uncaught Task Panics in Runtime** | `tokio::task::spawn_blocking` invocations aborted silently upon panic, failing to map errors into `AppError` and leaving frontend IPC callers hanging indefinitely without UI feedback. | `HIGH` |
| **F4** | **Missing Live CDP Fallback** | Headless `agy` CLI processes launched into background execution, but the visual Antigravity IDE Electron/Chromium composer remained blank, leaving the user unaware of prompt ingestion. | `HIGH` |
| **F5** | **Missing FIFO Queue Drain Hook** | Enqueued prompts (`status = 'queued'`) remained indefinitely dormant in `active_prompts` SQLite table because no automated daemon or exit hook triggered queue drain upon active turn completion. | `CRITICAL` |
| **F6** | **UI Button Coupling & Conflation** | `Send` and `Enqueue` actions were rendered as separate, uncoordinated buttons without unified split-action state, causing unintended concurrent dispatches and race conditions. | `MEDIUM` |

---

### 2.2 Deep Code-Level Root Causes

#### 1. Stale PID Retention in Worker Map (`src-tauri/src/modules/repo_db/agy.rs`)
In the legacy codebase, `get_active_agy_workers()` stored child process IDs in a static `HashMap<String, u32>` keyed by `"{instance_id}:{repo_path}"`. When a child process terminated unexpectedly or completed without explicitly clearing its map entry, the PID remained registered. Because the code unconditionally checked `workers.get(&ws_key)` and rejected new dispatches when present without checking `sys.refresh_processes_specifics`, the workspace was permanently locked out until full application restart.

#### 2. Bare Mutex Locking (`.lock().unwrap()`)
The global worker map `ACTIVE_AGY_WORKERS` used `std::sync::Mutex`. Standard library mutexes become "poisoned" if a thread panics while holding the lock. Legacy code used bare `.lock().unwrap()`. A single panic in an audit hook or logging call poisoned the mutex, causing all subsequent prompt dispatch attempts across all workspaces to panic immediately with `PoisonError`.

#### 3. Absence of `catch_unwind` in Async Bridge (`src-tauri/src/commands/instance.rs`)
Tauri command handlers `send_prompt_now` and `enqueue_prompt` delegated synchronous database and process operations to `tokio::task::spawn_blocking`. Because Rust panics inside `spawn_blocking` do not automatically convert to string errors returned to JavaScript, any panic inside the worker thread triggered a JoinError (`task failed to execute to completion`), bypassing user error dialogs and violating the **No Silent Errors** house rule.

#### 4. Disconnect Between CLI Process and IDE Composer
The Antigravity desktop IDE uses an Electron/Chromium shell. Submitting a prompt via `agy -p <text>` starts a standalone CLI session in the background. If the user had the GUI IDE open, the CLI worker executed isolated from the IDE renderer, so the chat history did not update in real-time, misleading users into believing the prompt was lost.

#### 5. Lack of Process Exit Hook for Queue Advancement
When `enqueue_prompt_for_instance` saved a prompt into `active_prompts` with `status = 'queued'`, it relied on manual polling. If a prompt was already executing, the queued prompt sat idle. When the executing `agy` process terminated, the child wait loop in `std::thread::spawn` recorded completion in SQLite but failed to inspect or dispatch pending queued prompts for that workspace.

---

### 2.3 Comprehensive Surgical Resolutions

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Prompt Dispatch Resolution Architecture                       │
└─────────────────────────────────────────────────────────────────────────────────────────┘

 User Action: Click "Send" (Explicit Turn)
       │
       ▼
 [ run_caught_prompt_task ] ─── std::panic::catch_unwind ───► Captures Panics -> AppError
       │
       ▼
 [ spawn_or_replace_prompt_via_agy(replace=true) ]
       │
       ├─► Poison-Resilient Mutex Guard: match get_active_agy_workers().lock() { ... }
       ├─► Process Liveness Probe: sys.refresh_processes_specifics([existing_pid])
       │     └─► If alive & replace=true: proc.kill() (Politely terminates lingering worker)
       ├─► Spawn agy CLI Worker: std::process::Command with CREATE_NO_WINDOW
       ├─► Live CDP Fallback: submit_prompt_via_cdp_for_instance(prompt.instance_id, clean_prompt)
       │
       ▼
 [ agy-monitor-<id> Thread ]
       │
       ├─► child.wait()
       ├─► Remove PID from ACTIVE_AGY_WORKERS
       ├─► Mark active_prompts status = 'completed'
       │
       ▼
 [ drain_next_queued_prompt_for_workspace ]
       │
       ├─► Query: SELECT * FROM active_prompts WHERE status = 'queued'
       │          ORDER BY created_at ASC, id ASC LIMIT 1
       │
       └─► Re-dispatch next prompt automatically!
```

#### Resolution Details:
1. **Polite Process Termination & Replacement**:
   In `src-tauri/src/modules/repo_db/agy.rs`, `spawn_or_replace_prompt_via_agy(prompt: &ActivePrompt, replace_existing: bool)` was refactored:
   - When `replace_existing` is `true` (explicit user Send/Rerun), it queries `sysinfo::System` for the recorded PID.
   - If the process is still running, it executes `proc.kill()` to terminate the lingering headless worker and purges the stale map entry.
   - If the process is no longer running (stale PID), it automatically removes the dead PID from the worker map and proceeds cleanly.

2. **Poison-Resilient Mutex Guards**:
   All accesses to `get_active_agy_workers()` now use poison-safe pattern matching:
   ```rust
   let mut workers = match get_active_agy_workers().lock() {
       Ok(guard) => guard,
       Err(poisoned) => poisoned.into_inner(),
   };
   ```
   This guarantees that even if a thread panics, subsequent operations reclaim the inner state without panicking.

3. **Panic-Trapping Async Wrapper (`run_caught_prompt_task`)**:
   In `src-tauri/src/commands/instance.rs`, all prompt operations are wrapped in `run_caught_prompt_task`:
   ```rust
   fn run_caught_prompt_task<F>(label: &str, task: F) -> Result<ActivePrompt, String>
   where
       F: FnOnce() -> Result<ActivePrompt, String> + Send,
   {
       let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
       match caught {
           Ok(result) => result,
           Err(payload) => Err(prompt_task_error(label, payload)),
       }
   }
   ```
   Panics are trapped, logged with high-resolution backtraces via `tracing::error!`, and routed as clean `Err(String)` to the frontend.

4. **Chrome DevTools Protocol (CDP) DOM Fallback**:
   When launching a prompt, `spawn_or_replace_prompt_via_agy` simultaneously calls:
   ```rust
   let _ = crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(
       &prompt.instance_id,
       &clean_prompt,
   );
   ```
   This hooks into the running Antigravity IDE's Chromium debug port (CDP), selects the active conversation composer textarea, injects the prompt text, and simulates the Enter keypress. This guarantees immediate visual reflection in the open GUI.

5. **Automated FIFO Queue Advancement Hook**:
   The background child monitor thread (`agy-monitor-<prompt_id>`) terminates with an unconditional queue advancement call:
   ```rust
   drain_next_queued_prompt_for_workspace(&inst_id_clone, &repo_path_clone);
   ```
   This function executes a strict FIFO query:
   ```sql
   SELECT id, instance_id, repo_path, prompt_content, session_id 
   FROM active_prompts 
   WHERE instance_id = ?1 AND repo_path = ?2 AND status = 'queued'
   ORDER BY created_at ASC, id ASC 
   LIMIT 1;
   ```
   If a queued prompt is found, it transitions status to `'running'` and immediately dispatches it via `spawn_or_replace_prompt_via_agy`.

6. **DetailHeader Split Action Group**:
   The UI button layout was separated into a primary `Send` split button with a secondary dropdown containing `Enqueue Prompt`, eliminating concurrent trigger hazards.

---

### 2.4 Preventive Invariants & Future Guardrails

- **[INVARIANT-FIFO-QUEUE]**: All queue queries against `active_prompts`, `prompt_backups`, and SQLite databases **MUST** specify `ORDER BY created_at ASC, id ASC`. LIFO order (`ORDER BY updated_at DESC`) is strictly forbidden for dispatch loops.
- **[INVARIANT-POISON-SAFE]**: Every Mutex locking operation on global worker state must utilize `.unwrap_or_else(|p| p.into_inner())` or `match lock() { Ok(g) => g, Err(p) => p.into_inner() }`. Bare `.unwrap()` is prohibited.
- **[INVARIANT-NO-SILENT-ERRORS]**: Infallible assertions and silent swallowed errors are forbidden. Fallible operations must route through `AppError` or `crate::error::record_ignored(..., "description")` with a mandatory `// Justification:` comment.
- **[INVARIANT-DUAL-DISPATCH]**: Headless CLI execution must always be accompanied by live CDP DOM submission when the target instance is actively running with an open window.

---

## 3. Fleet Remote Information Connectivity Specification

### 3.1 Hybrid 3-Tier Architecture

To achieve multi-machine fleet connectivity across diverse network topologies (LAN, WAN, headless servers, NAT/firewalls), Antigravity-Manager employs a **hybrid 3-tier synchronization layer**:

```
                                  +-----------------------+
                                  |   Remote AGM Fleet    |
                                  +-----------+-----------+
                                              |
                     +------------------------+------------------------+
                     |                        |                        |
                     v                        v                        v
             [ Tier 1: Supabase ]     [ Tier 2: Email IMAP ]   [ Tier 3: Telegram Bot ]
             • Realtime PostgREST     • Encrypted JSON Mail    • Bidirectional Webhook
             • Direct DB Cluster      • Firewall Bypassing     • Mobile Notification
             • Central Lease Table    • [AGM IN-USE] Claims    • Remote /fleet & /switch
                     |                        |                        |
                     +------------------------+------------------------+
                                              |
                                              v
                                  +-----------------------+
                                  | Local fleet_sync.db   |
                                  | (WAL Mode SQLite)     |
                                  +-----------------------+
```

1. **Tier 1 — Supabase PostgREST (Central Database Hub)**:
   - Primary high-speed synchronization plane for deployments with internet access and configured Supabase credentials.
   - Synchronizes `nodes`, `instances`, and `workspace_leases` tables.
   - Provides sub-second query latency and global lease arbitration.
2. **Tier 2 — Email IMAP/SMTP JSON Envelopes (Decentralized Tunnel)**:
   - Resilient asynchronous fallback that operates across strict corporate firewalls, NATs, and VPN boundaries without open inbound ports.
   - Uses standardized email subject headers (e.g. `[AGM IN-USE | <node> | <ip>] <email> (1h lease)`) and pure JSON body payloads.
   - Inbound daemon polls IMAP, extracts JSON payloads, and ingests them via `ingest_fleet_sync_payload`.
3. **Tier 3 — Telegram Inbound Daemon (Operational Remote Control)**:
   - Interactive remote control and telemetry broadcast via Telegram bot webhooks/long-polling.
   - Commands `/fleet`, `/whoami`, `/nodes`, `/switch <email>`, and `/open <machine>` allow remote operators to inspect fleet state and trigger cross-machine account rotations.

---

### 3.2 Pure JSON Telemetry Schemas

All multi-machine communication is strictly standardized into machine-readable JSON envelopes. Markdown, HTML, and unstructured plaintext formats are forbidden in programmatic sync routes.

#### 3.2.1 Primary Fleet Synchronization Payload (`agm-fleet-sync-v1.0`)
Defined in `src-tauri/src/modules/fleet_sync.rs` (`FleetSyncPayload`):

```json
{
  "$schema": "agm-fleet-sync-v1.0",
  "message_type": "heartbeat",
  "timestamp": 1729051200,
  "node": {
    "node_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "node_alias": "Node-Workstation-Alpha",
    "local_ip": "192.168.1.105",
    "os_info": "windows x86_64",
    "agm_version": "4.207.0",
    "uptime_seconds": 86400,
    "is_online": true,
    "last_heartbeat_at": 1729051200,
    "sync_transport": "email"
  },
  "instances": [
    {
      "instance_id": "inst-worker-1",
      "name": "Production Pipeline 1",
      "is_running": true,
      "pid": 14220,
      "bound_account_id": "acc-gemini-pro-01",
      "bound_email": "dev-ops-01@domain.com",
      "data_dir": "C:\\Users\\Admin\\.antigravity_tools\\instances\\inst-worker-1",
      "executable_path": "C:\\Program Files\\Antigravity\\antigravity.exe",
      "memory_mb": 420.5
    }
  ],
  "accounts": [
    {
      "id": "acc-gemini-pro-01",
      "email": "dev-ops-01@domain.com",
      "is_active": true,
      "quota_4h": 0.82,
      "quota_weekly": 0.45,
      "cooldown_until": null
    }
  ],
  "workspaces": [
    {
      "project_id": "proj-agm-repo",
      "repo_name": "Antigravity-Manager",
      "repo_path": "d:\\work\\Antigravity-Manager",
      "is_running": true,
      "running_prompts_count": 1,
      "active_prompt_snippet": "Executing UI and functionality enhancements Wave 2..."
    }
  ]
}
```

#### 3.2.2 Distributed Account In-Use Lease Payload (`[AGM IN-USE]`)
Defined in `src-tauri/src/modules/notification_hub/email_alerts.rs` (`send_account_in_use_lease_alert`):

```json
{
  "event": "account_in_use_lease",
  "version": "v4.207.0",
  "claimed_account": "active-user@company.com",
  "previous_account": "standby-user@company.com",
  "node_name": "Node-Workstation-Alpha",
  "node_ip": "192.168.1.105",
  "instance_id": "default",
  "claimed_at_utc": "2026-10-16T04:00:00Z",
  "claimed_at_unix": 1729051200,
  "lease_expires_at_utc": "2026-10-16T05:00:00Z",
  "lease_expires_at_unix": 1729054800,
  "lease_duration_seconds": 3600,
  "target_quota_4h": 0.95,
  "target_quota_weekly": 0.60,
  "previous_quota_4h": 0.12,
  "previous_quota_weekly": 0.15,
  "trigger_mode": "Auto-Switch",
  "reason": "Quota exhaustion threshold (98%) breached on previous account",
  "active_projects": ["Antigravity-Manager", "Backend-Gateway"],
  "prompts_backed_up": 2,
  "prompts_restored": 2
}
```

---

### 3.3 Multi-Machine Discovery & Collision Shielding

#### 1. Local Database Persistence (`fleet_sync.db`)
Incoming JSON payloads are parsed and committed in an atomic SQLite transaction:
- `FleetNodes`: Updates hardware info, IP, version, uptime, and last heartbeat timestamp.
- `FleetInstances`: Replaces the instance registry for the source node ID.
- `FleetWorkspaces`: Tracks active repositories and active prompt snippets per node.
- `FleetSummaryView`: Aggregates active nodes, total instances, running instances, and active workspaces in a unified view.

#### 2. Liveness TTL Evaluation
A node is considered online if and only if:
$$\text{is\_online} = \text{true} \quad \land \quad (\text{now}() - \text{last\_heartbeat\_at}) < 300\text{ seconds}$$
If no heartbeat is received for over 5 minutes (300s), the node is marked offline in UI status boards.

#### 3. Collision-Shielded Account Rotation & Fast-Forwarding
To prevent two nodes from concurrently auto-switching to the same high-quota account:
1. Before switching accounts, the scoring algorithm in `auto_switch` queries active leases from both Supabase and `fleet_sync.db`.
2. Any account with an active lease where $\text{now}() < \text{lease\_expires\_at\_unix}$ held by another node is penalized with an infinite cooldown weight ($+\infty$).
3. When Node Alpha successfully acquires an account, it broadcasts the `account_in_use_lease` JSON payload across Email and Supabase.
4. Peer nodes ingest the claim within their polling cycle and immediately exclude the account from their candidate pool.

---

## 4. Verification & Testing Matrix

| Test ID | Area | Verification Command / Procedure | Expected Outcome |
|---|---|---|---|
| **TEST-RCA-01** | Poison Mutex Resilience | Unit test injecting panic inside mock worker map lock | Poisoned mutex auto-recovers via `into_inner()` without panic. |
| **TEST-RCA-02** | Lingering Process Kill | Spawn dummy worker PID, invoke `spawn_or_replace_prompt_via_agy(replace=true)` | Process killed, PID cleared, new worker spawns cleanly. |
| **TEST-RCA-03** | FIFO Queue Drain | Enqueue 2 prompts in `active_prompts`, let first exit | Monitor thread triggers `drain_next_queued_prompt_for_workspace` and executes prompt 2. |
| **TEST-FLEET-01** | JSON Schema Ingestion | Ingest sample `FleetSyncPayload` via `ingest_fleet_sync_payload` | `FleetNodes` and `FleetInstances` updated in `fleet_sync.db`. |
| **TEST-FLEET-02** | Lease Collision Shield | Simulate active lease on `test@org.com`, trigger candidate scoring | Candidate scoring excludes `test@org.com` until lease expiry. |
| **TEST-PREFLIGHT** | Pre-Flight Gates | `cargo fmt -- --check`, `cargo clippy`, `npm run build` | All pass with zero errors and zero warnings. |
