# Subtask 02: Supabase Stack Trace & Error Logging Hardening

**Plan Slug:** `supabase-connection-and-ui-ux-enhancement`  
**Subtask File:** `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/02-stack-trace-and-error-logging.md`  
**Associated Spec:** `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-supabase-and-secrets-spec.md`  
**Status:** READY  
**Author:** Worker 01 (Supabase, Secrets & Image Storage Architect)  

---

## 1. Context & Objectives

In accordance with the project's non-negotiable architectural invariant:
> **No Silent Errors (house rule, 2026-10-10)**: Every fallible operation must route its failure into the error module. Swallowing an error — discarding it where no one can ever see it — is forbidden. In Rust (`src-tauri/src/`), errors flow into `AppError` and are logged via `tracing` (file + console + Tauri log bridge). Never leave a bare `let _ = <fallible>`, `.ok()`, or `.unwrap_or_default()` on a `Result` without routing the `Err`. For genuinely best-effort operations, `crate::error::record_ignored(...)` plus a one-line `// Justification:` comment is strictly required.

This subtask conducts a targeted, surgical eradication of silent error swallowing across four key Supabase Rust modules:
1. `src-tauri/src/modules/supabase_client/client.rs`
2. `src-tauri/src/modules/supabase_sync/query.rs`
3. `src-tauri/src/modules/supabase_sync/fleet.rs`
4. `src-tauri/src/modules/supabase_command_queue.rs`

---

## 2. Exhaustive Audit of Swallowed Errors

### 2.1 `src-tauri/src/modules/supabase_sync/query.rs`
- **Location (Lines 57-60)**:
  ```rust
  let profiles_val = client
      .select("instance_profiles", "select=*")
      .await
      .unwrap_or(serde_json::Value::Array(Vec::new()));
  ```
  *Defect*: If the PostgREST query fails due to authentication expiration or network drop, the failure is discarded silently.
- **Location (Lines 113-116)**:
  ```rust
  let leases_val = client
      .select("workspace_leases", "select=*")
      .await
      .unwrap_or(serde_json::Value::Array(Vec::new()));
  ```
  *Defect*: Lease query errors are swallowed, causing the UI to report zero active leases even during an active network partition.

### 2.2 `src-tauri/src/modules/supabase_sync/fleet.rs`
- **Location (Lines 152-165)**:
  ```rust
  if let Some(ref c) = client {
      if let Ok(val) = c.select("nodes", "select=*&order=last_heartbeat_at.desc").await {
          raw_nodes = serde_json::from_value(val).unwrap_or_default();
      }
      if let Ok(val) = c.select("instance_profiles", "select=*").await {
          raw_profiles = serde_json::from_value(val).unwrap_or_default();
      }
      if let Ok(val) = c.select("workspace_leases", "select=*").await {
          raw_leases = serde_json::from_value(val).unwrap_or_default();
      }
  }
  ```
  *Defect*: Three consecutive `if let Ok` statements and three `.unwrap_or_default()` calls silently conceal database query and JSON deserialization errors.

### 2.3 `src-tauri/src/modules/supabase_command_queue.rs`
- **Location (Lines 48-65)**:
  ```rust
  for ep in endpoints {
      let client = match SupabaseClient::new(&ep) {
          Ok(c) => c,
          Err(_) => continue,
      };
      ...
      if let Ok(val) = client.select("command_queue", &query).await {
          if let Ok(cmds) = serde_json::from_value::<Vec<InboundDbCommand>>(val) {
              return (Some(client), cmds);
          }
      }
  }
  ```
  *Defect*: `Err(_) => continue` skips broken endpoints without logging, and nested `if let Ok` calls swallow failed query responses and schema deserialization errors.

---

## 3. Step-by-Step Remediation Plan

### Step 1: Query Error Logging in `query.rs`
Replace bare `.unwrap_or` with explicit `match` blocks capturing stack traces:
```rust
let profiles_val = match client.select("instance_profiles", "select=*").await {
    Ok(val) => val,
    Err(e) => {
        let bt = std::backtrace::Backtrace::capture();
        tracing::warn!(
            target: "supabase::query",
            "[FleetQuery] Failed to fetch instance_profiles from {}: {}\nBacktrace:\n{:?}",
            client.base_url(), e, bt
        );
        serde_json::Value::Array(Vec::new())
    }
};

let leases_val = match client.select("workspace_leases", "select=*").await {
    Ok(val) => val,
    Err(e) => {
        let bt = std::backtrace::Backtrace::capture();
        tracing::warn!(
            target: "supabase::query",
            "[FleetQuery] Failed to fetch workspace_leases from {}: {}\nBacktrace:\n{:?}",
            client.base_url(), e, bt
        );
        serde_json::Value::Array(Vec::new())
    }
};
```

### Step 2: Deserialization and Query Visibility in `fleet.rs`
Replace `if let Ok` and `.unwrap_or_default()` with exhaustive error reporting:
```rust
if let Some(ref c) = client {
    match c.select("nodes", "select=*&order=last_heartbeat_at.desc").await {
        Ok(val) => {
            match serde_json::from_value::<Vec<PostgrestNodeRow>>(val) {
                Ok(nodes) => raw_nodes = nodes,
                Err(e) => {
                    let bt = std::backtrace::Backtrace::capture();
                    tracing::warn!(
                        target: "supabase::fleet",
                        "[FleetSync] JSON deserialization failed for nodes table: {}\nBacktrace:\n{:?}",
                        e, bt
                    );
                }
            }
        }
        Err(e) => {
            let bt = std::backtrace::Backtrace::capture();
            tracing::warn!(
                target: "supabase::fleet",
                "[FleetSync] PostgREST query failed for nodes on {}: {}\nBacktrace:\n{:?}",
                c.base_url(), e, bt
            );
        }
    }
    // Repeat symmetrical pattern for instance_profiles and workspace_leases
}
```

### Step 3: Cascading Failover Logging in `supabase_command_queue.rs`
Ensure every skipped secondary endpoint logs its error before advancing to the next endpoint:
```rust
for ep in endpoints {
    let client = match SupabaseClient::new(&ep) {
        Ok(c) => c,
        Err(e) => {
            let bt = std::backtrace::Backtrace::capture();
            tracing::warn!(
                target: "supabase::command_queue",
                "[CommandQueue] Client initialization failed for secondary endpoint {} ({}): {}\nBacktrace:\n{:?}",
                ep.id, ep.url, e, bt
            );
            continue;
        }
    };

    let query = format!(
        "status=eq.pending&(target_node_id=eq.{}|target_node_id=eq.*)&order=created_at.asc&limit=5",
        node_id
    );

    match client.select("command_queue", &query).await {
        Ok(val) => match serde_json::from_value::<Vec<InboundDbCommand>>(val) {
            Ok(cmds) => return (Some(client), cmds),
            Err(e) => {
                let bt = std::backtrace::Backtrace::capture();
                tracing::warn!(
                    target: "supabase::command_queue",
                    "[CommandQueue] JSON deserialization failed on endpoint {}: {}\nBacktrace:\n{:?}",
                    ep.id, e, bt
                );
            }
        },
        Err(e) => {
            let bt = std::backtrace::Backtrace::capture();
            tracing::warn!(
                target: "supabase::command_queue",
                "[CommandQueue] PostgREST select failed on endpoint {}: {}\nBacktrace:\n{:?}",
                ep.id, e, bt
            );
        }
    }
}
```

### Step 4: Verification of Justified Ignored Operations
For background telemetry or update calls where failure must not derail execution (e.g. marking a command as running before executing it), enforce `crate::error::record_ignored(...)` with an explicit `// Justification:` comment:
```rust
// Justification: best-effort command status update; execution proceeds even if remote state sync lags
crate::error::record_ignored(
    client.update("command_queue", &query, run_payload).await,
    "update command status to running",
);
```

### Step 5: Unit & Compiler Verification
- Run `cargo fmt -- --check` across `src-tauri`.
- Run `cargo clippy --all-targets --all-features` to ensure zero lint warnings.
- Run targeted tests for `supabase_client` and `supabase_sync`.

---

## 4. Acceptance Criteria & Quality Gates

- [ ] **AC-1 (Zero Bare Unwraps)**: Zero occurrences of unmonitored `.unwrap_or(...)` on remote PostgREST queries in `query.rs` and `fleet.rs`.
- [ ] **AC-2 (Backtrace Capture)**: All query and deserialization error paths capture and log `std::backtrace::Backtrace`.
- [ ] **AC-3 (Failover Visibility)**: `supabase_command_queue.rs` logs informative diagnostics for skipped endpoints before trying fallbacks.
- [ ] **AC-4 (Justification Comments)**: All best-effort operations use `crate::error::record_ignored` accompanied by a clear `// Justification:` comment.
- [ ] **AC-5 (Gate Cleanliness)**: `cargo clippy` and `cargo fmt` pass without errors.
