# Subtask 02: Backend Loops, Supabase Backoff & Quota Refresh Intervals

**Application:** Antigravity Manager (AGM)  
**Task Slug:** `antigravity-manager-agm-codebase-review-and-refactoring`  
**Subtask ID:** `02-backend-loops-supabase-backoff-and-quota-intervals`  
**Status:** Ready / Implemented  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md)

---

## 1. Problem Statement & Scope

1. **Poller Loop CPU Churn:** Background loops in CLI commands `fpug` (finish prompts until green) and `sug` (shutdown until green) allowed low sleep durations clamped to 5s (`interval_sec.max(5)`), leading to excessive CPU wakeups.
2. **Missing Workspace Infinite Spin:** If an enqueued prompt pointed to a deleted or non-existent `repo_path`, `queue.rs` skipped it without updating its status, leaving it to be re-evaluated every 5 seconds indefinitely.
3. **Supabase Heartbeat Resilience:** Network interruptions or misconfigured endpoints triggered aggressive retries rather than sliding backoffs.
4. **Quota Rate-Limiting:** Frequent account quota checks risked hitting Google OAuth rate limits without adaptive caching.

---

## 2. Technical Implementation Plan

### 2.1 Poller Loop Floor in `src-tauri/src/bin/agm/green_cmds.rs`
- In `cmd_finish_prompts_until_green`:
  ```rust
  // Enforce strict >= 30s floor on poller sleep
  std::thread::sleep(Duration::from_secs(interval_sec.max(30)));
  ```
- In `cmd_shutdown_until_green`:
  ```rust
  // Enforce strict >= 30s floor on poller sleep
  std::thread::sleep(Duration::from_secs(interval_sec.max(30)));
  ```

### 2.2 Missing Workspace Directory Handling in `src-tauri/src/modules/repo_db/queue.rs`
- In `check_and_dispatch_enqueued_prompts`:
  ```rust
  let ws_dir = PathBuf::from(&prompt.repo_path);
  if !ws_dir.exists() {
      crate::modules::logger::log_warn(&format!(
          "[PromptQueueScheduler] workspace directory does not exist: '{}' for prompt '{}'; marking as failed to prevent infinite retry loop",
          prompt.repo_path, prompt.id
      ));
      // Mark missing workspace prompt failed to break infinite retry loop
      crate::error::record_ignored(
          conn.execute(
              "UPDATE active_prompts SET status = 'failed', updated_at = ?1 WHERE id = ?2",
              rusqlite::params![now, &prompt.id],
          ),
          "mark missing workspace prompt failed",
      );
      if let Ok(mut map) = get_memory_prompts_map().lock() {
          if let Some(p) = map.get_mut(&prompt.id) {
              p.status = "failed".to_string();
              p.updated_at = now;
          }
      }
      continue;
  }
  ```

### 2.3 Supabase 300s Backoff in `src-tauri/src/modules/supabase_sync/fleet.rs`
- In `start_sync_worker`:
  ```rust
  let config = match load_config() {
      Ok(c) => c,
      Err(e) => {
          tracing::warn!(
              target: "supabase_sync",
              "Failed to load config in background sync worker: {e}; backing off for 300 seconds"
          );
          tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
          continue;
      }
  };

  if !config.is_sync_enabled {
      continue;
  }

  for ep in &config.endpoints {
      if !ep.is_enabled {
          continue;
      }
      if ep.role == "root" {
          if let Err(e) = send_heartbeat(ep, &config.node_alias).await {
              tracing::warn!(
                  target: "supabase_sync",
                  "Failed to send heartbeat to Supabase endpoint '{}': {e}; backing off for 300 seconds",
                  ep.url
              );
              tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
          }
      }
  }
  ```

### 2.4 Adaptive Account Quota L1 Cache in `src-tauri/src/modules/account/quota_fetch.rs`
- In `store_l1_quota_cache`:
  ```rust
  let percentage = detected_percentage.unwrap_or(100.0);
  let ttl_seconds = if percentage < 20.0 {
      120 // 2 minutes for low quota (<20%)
  } else if percentage < 50.0 {
      180 // 3 minutes for moderate quota (<50%)
  } else {
      300 // 5 minutes for full / healthy quota (>=50%)
  };
  ```
- Untouched / inactive accounts inherit a default 300s to 600s TTL.

---

## 3. Invariants & Positive Booleans

- `is_sync_enabled`: Positive boolean determining whether fleet sync daemon should run.
- `is_enabled`: Positive boolean indicating active Supabase endpoint state.
- `is_workspace_present`: Positive check ensuring directory exists on disk before dispatch.

---

## 4. Verification & Testing

```bash
cd src-tauri && cargo test -p antigravity-tools-lib repo_db
cd src-tauri && cargo test -p antigravity-tools-lib account::quota_fetch
cd src-tauri && cargo test -p antigravity-tools-lib supabase_sync
cd src-tauri && cargo check --bin agm
```

### Verification Criteria
- [x] Background CLI commands `agm fpug` and `agm sug` enforce sleep >= 30s.
- [x] Missing workspace directory logs warning and marks prompt `status = 'failed'`.
- [x] Supabase heartbeat error waits 300s before retrying.
- [x] L1 quota cache TTL responds dynamically to account remaining quota percentage.
